-- =============================================================
-- FamilyGuard — one-shot setup (schema + RLS).
-- Paste this whole file into the Supabase SQL editor and Run.
-- Optionally also run retention.sql for scheduled data cleanup.
-- =============================================================

-- ============================================================================
-- FamilyGuard — Postgres schema (Supabase)
-- ============================================================================
-- Run order: schema.sql -> policies.sql -> (optional) seed.sql
-- All family data is protected by RLS in policies.sql. Run BOTH files.
--
-- Conventions:
--   * UUID primary keys (gen_random_uuid()).
--   * created_at / updated_at timestamptz on mutable tables.
--   * Parents are Supabase Auth users (auth.users). Devices are NOT auth users.
-- ============================================================================

create extension if not exists pgcrypto;      -- gen_random_uuid(), digest()

-- ---------------------------------------------------------------------------
-- updated_at helper
-- ---------------------------------------------------------------------------
create or replace function set_updated_at()
returns trigger language plpgsql as $$
begin
  new.updated_at = now();
  return new;
end; $$;

-- ---------------------------------------------------------------------------
-- families: a household / account
-- ---------------------------------------------------------------------------
create table if not exists families (
  id          uuid primary key default gen_random_uuid(),
  name        text not null default 'My Family',
  created_at  timestamptz not null default now(),
  updated_at  timestamptz not null default now()
);
create trigger trg_families_updated before update on families
  for each row execute function set_updated_at();

-- ---------------------------------------------------------------------------
-- family_members: links auth.users to a family with a role
--   role: 'owner' (created the family) | 'guardian' (invited co-parent)
-- ---------------------------------------------------------------------------
create table if not exists family_members (
  id          uuid primary key default gen_random_uuid(),
  family_id   uuid not null references families(id) on delete cascade,
  user_id     uuid not null references auth.users(id) on delete cascade,
  role        text not null default 'guardian' check (role in ('owner','guardian')),
  created_at  timestamptz not null default now(),
  unique (family_id, user_id)
);
create index if not exists idx_family_members_user on family_members(user_id);
create index if not exists idx_family_members_family on family_members(family_id);

-- ---------------------------------------------------------------------------
-- children: a child profile within a family
--   birth_year drives age-appropriate defaults (no full DOB needed -> minimize data)
-- ---------------------------------------------------------------------------
create table if not exists children (
  id          uuid primary key default gen_random_uuid(),
  family_id   uuid not null references families(id) on delete cascade,
  name        text not null,
  birth_year  int  check (birth_year between 1990 and extract(year from now())::int),
  avatar      text,
  created_at  timestamptz not null default now(),
  updated_at  timestamptz not null default now()
);
create index if not exists idx_children_family on children(family_id);
create trigger trg_children_updated before update on children
  for each row execute function set_updated_at();

-- ---------------------------------------------------------------------------
-- devices: a physical Android device bound to a child
--   device JWTs are scoped to devices.id; revoked_at disables a device server-side.
-- ---------------------------------------------------------------------------
create table if not exists devices (
  id              uuid primary key default gen_random_uuid(),
  child_id        uuid not null references children(id) on delete cascade,
  label           text not null default 'Phone',
  platform        text not null default 'android',
  model           text,
  os_version      text,
  -- management posture achieved on-device:
  management_mode text not null default 'unprovisioned'
                    check (management_mode in ('unprovisioned','device_admin','device_owner')),
  push_token      text,                    -- FCM token (wake signal only)
  last_seen_at    timestamptz,
  battery_pct     int,
  foreground_app  text,                    -- last reported foreground package (for live status)
  revoked_at      timestamptz,             -- set to unlink a device
  created_at      timestamptz not null default now(),
  updated_at      timestamptz not null default now()
);
create index if not exists idx_devices_child on devices(child_id);
create trigger trg_devices_updated before update on devices
  for each row execute function set_updated_at();

-- ---------------------------------------------------------------------------
-- enrollment_tokens: short-lived, single-use tokens that bind a device to a child
--   Only the SHA-256 hash is stored; the raw token lives only in the QR code.
-- ---------------------------------------------------------------------------
create table if not exists enrollment_tokens (
  id          uuid primary key default gen_random_uuid(),
  child_id    uuid not null references children(id) on delete cascade,
  token_hash  bytea not null unique,       -- digest(raw_token,'sha256')
  expires_at  timestamptz not null,
  consumed_at timestamptz,
  device_id   uuid references devices(id),  -- set when consumed
  created_at  timestamptz not null default now()
);
create index if not exists idx_enroll_child on enrollment_tokens(child_id);

-- ---------------------------------------------------------------------------
-- policies: the active rule set for a child
--   Stable fields as columns; evolving per-app/web rules in jsonb `rules`.
--   Enforce one active policy per child with a partial unique index.
-- ---------------------------------------------------------------------------
create table if not exists policies (
  id                 uuid primary key default gen_random_uuid(),
  child_id           uuid not null references children(id) on delete cascade,
  is_active          boolean not null default true,
  daily_limit_min    int,                  -- total screen-time minutes/day (null = no limit)
  bedtime_start      time,                 -- e.g. 21:00 local
  bedtime_end        time,                 -- e.g. 07:00 local
  timezone           text not null default 'UTC',
  location_enabled   boolean not null default false,
  web_filter_level   text not null default 'moderate'
                       check (web_filter_level in ('off','moderate','strict')),
  rules              jsonb not null default '{}'::jsonb,  -- {apps:[{pkg,action,limitMin}], webCategories:[...]}
  version            bigint not null default 1,           -- bumped on each change; device compares
  created_at         timestamptz not null default now(),
  updated_at         timestamptz not null default now()
);
create unique index if not exists uniq_active_policy_per_child
  on policies(child_id) where is_active;
create index if not exists idx_policies_child on policies(child_id);
create trigger trg_policies_updated before update on policies
  for each row execute function set_updated_at();

-- bump version + enqueue a sync command whenever an active policy changes
create or replace function on_policy_change()
returns trigger language plpgsql as $$
begin
  new.version := coalesce(old.version,0) + 1;
  return new;
end; $$;
create trigger trg_policies_version before update on policies
  for each row when (new.is_active) execute function on_policy_change();

-- ---------------------------------------------------------------------------
-- device_commands: parent -> device command queue
-- ---------------------------------------------------------------------------
create table if not exists device_commands (
  id          uuid primary key default gen_random_uuid(),
  device_id   uuid not null references devices(id) on delete cascade,
  kind        text not null check (kind in
                ('policy_updated','lock_now','unlock','focus_now','sync','unlink','locate_now')),
  payload     jsonb not null default '{}'::jsonb,
  created_at  timestamptz not null default now(),
  delivered_at timestamptz,
  acked_at     timestamptz
);
create index if not exists idx_cmds_device_undelivered
  on device_commands(device_id) where acked_at is null;

-- ---------------------------------------------------------------------------
-- telemetry: usage_events, heartbeats, locations (device -> backend)
-- ---------------------------------------------------------------------------
-- Daily per-app usage. One row per (device, package, day). The device reports
-- today's cumulative foreground seconds each sync; the backend UPSERTs onto the
-- day's row, so repeated syncs overwrite (not add) — no double counting.
create table if not exists usage_events (
  id            bigint generated always as identity primary key,
  device_id     uuid not null references devices(id) on delete cascade,
  package_name  text not null,
  foreground_s  int  not null default 0,   -- cumulative foreground seconds for `day`
  day           date not null default (now() at time zone 'utc')::date,
  occurred_at   timestamptz not null default now(),  -- last update time
  created_at    timestamptz not null default now(),
  unique (device_id, package_name, day)
);
create index if not exists idx_usage_device_day on usage_events(device_id, day desc);

create table if not exists heartbeats (
  id            bigint generated always as identity primary key,
  device_id     uuid not null references devices(id) on delete cascade,
  battery_pct   int,
  foreground_app text,
  policy_version bigint,                    -- which policy the device currently has
  created_at    timestamptz not null default now()
);
create index if not exists idx_heartbeats_device_time on heartbeats(device_id, created_at desc);

create table if not exists locations (
  id            bigint generated always as identity primary key,
  device_id     uuid not null references devices(id) on delete cascade,
  lat           double precision not null,
  lon           double precision not null,
  accuracy_m    double precision,
  occurred_at   timestamptz not null,
  created_at    timestamptz not null default now()
);
create index if not exists idx_locations_device_time on locations(device_id, occurred_at desc);

-- ---------------------------------------------------------------------------
-- Convenience: does the current auth user belong to the family owning a child?
-- Used by RLS policies. SECURITY DEFINER so it can read family_members.
-- ---------------------------------------------------------------------------
create or replace function auth_user_owns_child(p_child uuid)
returns boolean language sql stable security definer set search_path = public as $$
  select exists (
    select 1
    from children c
    join family_members fm on fm.family_id = c.family_id
    where c.id = p_child and fm.user_id = auth.uid()
  );
$$;

create or replace function auth_user_owns_device(p_device uuid)
returns boolean language sql stable security definer set search_path = public as $$
  select exists (
    select 1
    from devices d
    join children c on c.id = d.child_id
    join family_members fm on fm.family_id = c.family_id
    where d.id = p_device and fm.user_id = auth.uid()
  );
$$;


-- ========================= RLS =============================
-- ============================================================================
-- FamilyGuard — Row Level Security (RLS)
-- ============================================================================
-- Run AFTER schema.sql.
--
-- SECURITY-CRITICAL FILE. The parent web app talks to Postgres directly, so
-- these policies are the boundary that stops one family reading another's data.
--
-- Model:
--   * Parents = auth.users. A parent may touch a row iff it chains:
--        row -> child -> family -> family_members(user_id = auth.uid())
--     (families/children/members use the same chain; see helper functions).
--   * Devices DO NOT use these policies. Devices never connect as a Postgres
--     role with RLS; they call Edge Functions (service role) that re-derive the
--     device_id from the verified device JWT. Telemetry tables therefore have
--     RLS that allows parents to READ their own devices' rows, and NO anon/auth
--     INSERT — inserts happen via service role in Edge Functions only.
--
-- TEST CHECKLIST (write these as automated tests before launch):
--   [ ] Parent A cannot SELECT Parent B's families/children/devices/policies/telemetry.
--   [ ] Parent A cannot UPDATE/DELETE Parent B's rows.
--   [ ] A parent can do full CRUD within their own family.
--   [ ] anon role can do nothing on these tables.
--   [ ] A non-member of a family sees zero rows for it.
-- ============================================================================

alter table families          enable row level security;
alter table family_members    enable row level security;
alter table children          enable row level security;
alter table devices           enable row level security;
alter table enrollment_tokens enable row level security;
alter table policies          enable row level security;
alter table device_commands   enable row level security;
alter table usage_events      enable row level security;
alter table heartbeats        enable row level security;
alter table locations         enable row level security;

-- ---------------------------------------------------------------------------
-- families: member can read; owner can update/delete; any authed user can
-- create a family (they become the owner via a trigger-free app-side insert
-- into family_members, or via the create-family RPC — see note).
-- ---------------------------------------------------------------------------
create policy families_select on families for select
  using (exists (select 1 from family_members fm
                 where fm.family_id = families.id and fm.user_id = auth.uid()));

create policy families_insert on families for insert
  with check (true);  -- creation is allowed; membership is added immediately after (see create_family RPC)

create policy families_update on families for update
  using (exists (select 1 from family_members fm
                 where fm.family_id = families.id and fm.user_id = auth.uid()
                   and fm.role = 'owner'));

create policy families_delete on families for delete
  using (exists (select 1 from family_members fm
                 where fm.family_id = families.id and fm.user_id = auth.uid()
                   and fm.role = 'owner'));

-- ---------------------------------------------------------------------------
-- family_members: a user can see members of families they belong to; owners manage members.
-- NOTE: policies must avoid infinite recursion. We reference family_members
-- directly with a self-contained subquery on the same table via a security-definer helper.
-- ---------------------------------------------------------------------------
create or replace function auth_user_in_family(p_family uuid)
returns boolean language sql stable security definer set search_path = public as $$
  select exists (select 1 from family_members fm
                 where fm.family_id = p_family and fm.user_id = auth.uid());
$$;

create or replace function auth_user_is_owner(p_family uuid)
returns boolean language sql stable security definer set search_path = public as $$
  select exists (select 1 from family_members fm
                 where fm.family_id = p_family and fm.user_id = auth.uid() and fm.role = 'owner');
$$;

create policy fm_select on family_members for select
  using (auth_user_in_family(family_id));

-- allow a user to insert their OWN membership (used by create_family RPC path),
-- and allow owners to add others.
create policy fm_insert on family_members for insert
  with check (user_id = auth.uid() or auth_user_is_owner(family_id));

create policy fm_update on family_members for update
  using (auth_user_is_owner(family_id));

create policy fm_delete on family_members for delete
  using (auth_user_is_owner(family_id));

-- ---------------------------------------------------------------------------
-- children
-- ---------------------------------------------------------------------------
create policy children_select on children for select using (auth_user_in_family(family_id));
create policy children_insert on children for insert with check (auth_user_in_family(family_id));
create policy children_update on children for update using (auth_user_in_family(family_id));
create policy children_delete on children for delete using (auth_user_in_family(family_id));

-- ---------------------------------------------------------------------------
-- devices  (parents manage; device status writes come via Edge Functions/service role)
-- ---------------------------------------------------------------------------
create policy devices_select on devices for select using (auth_user_owns_child(child_id));
create policy devices_insert on devices for insert with check (auth_user_owns_child(child_id));
create policy devices_update on devices for update using (auth_user_owns_child(child_id));
create policy devices_delete on devices for delete using (auth_user_owns_child(child_id));

-- ---------------------------------------------------------------------------
-- enrollment_tokens (parents create/see their own; consumption is service-role)
-- ---------------------------------------------------------------------------
create policy enroll_select on enrollment_tokens for select using (auth_user_owns_child(child_id));
create policy enroll_insert on enrollment_tokens for insert with check (auth_user_owns_child(child_id));
create policy enroll_delete on enrollment_tokens for delete using (auth_user_owns_child(child_id));

-- ---------------------------------------------------------------------------
-- policies
-- ---------------------------------------------------------------------------
create policy policies_select on policies for select using (auth_user_owns_child(child_id));
create policy policies_insert on policies for insert with check (auth_user_owns_child(child_id));
create policy policies_update on policies for update using (auth_user_owns_child(child_id));
create policy policies_delete on policies for delete using (auth_user_owns_child(child_id));

-- ---------------------------------------------------------------------------
-- device_commands (parent enqueues; device acks via Edge Function/service role)
-- ---------------------------------------------------------------------------
create policy cmd_select on device_commands for select using (auth_user_owns_device(device_id));
create policy cmd_insert on device_commands for insert with check (auth_user_owns_device(device_id));

-- ---------------------------------------------------------------------------
-- telemetry: parents may READ their own devices' rows. No auth INSERT:
-- all telemetry is written by Edge Functions using the service role (which
-- bypasses RLS), after verifying the device JWT.
-- ---------------------------------------------------------------------------
create policy usage_select     on usage_events for select using (auth_user_owns_device(device_id));
create policy heartbeats_select on heartbeats   for select using (auth_user_owns_device(device_id));
create policy locations_select on locations     for select using (auth_user_owns_device(device_id));

-- ---------------------------------------------------------------------------
-- create_family RPC: atomically create a family and add the caller as owner.
-- Keeps the "families_insert with_check(true)" policy safe in practice because
-- the web app uses this RPC instead of raw inserts.
-- ---------------------------------------------------------------------------
create or replace function create_family(p_name text default 'My Family')
returns uuid language plpgsql security definer set search_path = public as $$
declare v_family uuid;
begin
  if auth.uid() is null then
    raise exception 'not authenticated';
  end if;
  insert into families(name) values (coalesce(p_name,'My Family')) returning id into v_family;
  insert into family_members(family_id, user_id, role) values (v_family, auth.uid(), 'owner');
  return v_family;
end; $$;

revoke all on function create_family(text) from public;
grant execute on function create_family(text) to authenticated;
