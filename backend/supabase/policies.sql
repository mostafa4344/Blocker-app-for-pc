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
