-- (2026-10-25) Automatic app updates + anonymous health reports.
-- Needs 20261025010000_preset_lists.sql and 20261025020000_presets_version.sql first.
-- Safe to run more than once.
--
-- HOW TO APPLY: Supabase dashboard -> SQL Editor -> paste this file -> Run.

-- ---------------------------------------------------------------------------
-- 1. Releases: the app installs the newest enabled release automatically.
--    Rows are created by the publish script (docs/AUTO-UPDATE.md). The app
--    only installs a file whose Ed25519 signature matches the public key built
--    into it, so a wrong/forged row can't install anything.
-- ---------------------------------------------------------------------------
create table if not exists public.app_releases (
  version     text primary key check (version ~ '^[0-9]+(\.[0-9]+){1,3}$'),
  url         text not null check (url like 'https://%'),
  sha256      text not null check (sha256 ~ '^[0-9a-f]{64}$'),
  size        bigint not null check (size > 0),
  signature   text not null check (signature ~ '^[0-9a-f]{128}$'),
  -- Share of devices (0-100) that get it now: start small, then raise.
  rollout     int not null default 0 check (rollout between 0 and 100),
  enabled     boolean not null default true,
  notes       text not null default '',
  created_at  timestamptz not null default now()
);
alter table public.app_releases enable row level security;
revoke all on table public.app_releases from anon, authenticated;

-- Newest enabled release (compares versions numerically: 1.10.0 > 1.9.0).
create or replace function public.latest_release()
returns json language sql stable security definer set search_path = public as $$
  select json_build_object('version', version, 'url', url, 'sha256', sha256,
                           'size', size, 'signature', signature, 'rollout', rollout)
  from public.app_releases
  where enabled and rollout > 0
  order by string_to_array(version, '.')::int[] desc
  limit 1
$$;
revoke all on function public.latest_release() from public;

-- ---------------------------------------------------------------------------
-- 2. Health: last time each device checked in, its app version and a small
--    anonymous status object (no browsing data, no personal data).
-- ---------------------------------------------------------------------------
alter table public.accounts add column if not exists last_seen   timestamptz;
alter table public.accounts add column if not exists app_version text;
alter table public.accounts add column if not exists health      jsonb;
create index if not exists accounts_last_seen_idx on public.accounts (last_seen);

-- Small admin-editable settings sent to every app (no app update needed):
--   purchase_url  web page of the "Get Premium" button; the app appends ?id=<device id>
create table if not exists public.app_config (
  key   text primary key,
  value text not null default ''
);
alter table public.app_config enable row level security;
revoke all on table public.app_config from anon, authenticated;
insert into public.app_config (key, value) values ('purchase_url', '') on conflict (key) do nothing;

create or replace function public.app_config_value(p_key text)
returns text language sql stable security definer set search_path = public as $$
  select nullif(value, '') from public.app_config where key = p_key
$$;
revoke all on function public.app_config_value(text) from public;

-- Every license answer carries the preset version, the latest release, the
-- purchase page and a flag telling the app it may send health reports.
create or replace function public.license_json(a public.accounts)
returns json language sql stable security definer set search_path = public as $$
  select json_build_object(
    'plan', public.effective_plan(a),
    'trial_started', a.trial_started,
    'trial_days', a.trial_days,
    'premium_until', a.premium_until,
    'presets_version', public.presets_version(),
    'latest_release', public.latest_release(),
    'purchase_url', public.app_config_value('purchase_url'),
    'accepts_health', true
  )
$$;
revoke all on function public.license_json(public.accounts) from public;

-- get_license now also takes the app version and health report (both
-- optional, so older apps that send only p_install_id keep working).
-- The row is written at most about once a day per device unless something
-- changed, to keep database load low with millions of devices.
drop function if exists public.get_license(text);
create or replace function public.get_license(
  p_install_id text, p_version text default null, p_health jsonb default null
) returns json language plpgsql security definer set search_path = public as $$
declare a public.accounts;
begin
  select * into a from public.accounts where install_id = p_install_id;
  if not found then raise exception 'not registered' using errcode = 'P0002'; end if;
  if p_health is not null and length(p_health::text) > 4000 then
    p_health := jsonb_build_object('error', 'report too large');
  end if;
  if a.last_seen is null
     or a.last_seen < now() - interval '20 hours'
     or a.app_version is distinct from coalesce(left(p_version, 32), a.app_version)
     or (p_health is not null and a.health is distinct from p_health) then
    update public.accounts
       set last_seen = now(),
           app_version = coalesce(left(p_version, 32), app_version),
           health = coalesce(p_health, health)
     where install_id = a.install_id
     returning * into a;
  end if;
  return public.license_json(a);
end $$;
revoke all on function public.get_license(text, text, jsonb) from public;
grant execute on function public.get_license(text, text, jsonb) to anon, authenticated;

-- ---------------------------------------------------------------------------
-- 3. Admin dashboard views (open them in Table Editor / SQL Editor).
--    Not readable with the public app key.
-- ---------------------------------------------------------------------------
create or replace view public.admin_health_summary as
select
  coalesce(app_version, 'unknown')                         as app_version,
  count(*)                                                  as devices,
  count(*) filter (where last_seen > now() - interval '7 days') as active_7d,
  count(*) filter (where (health->>'hosts_ok') = 'false')  as blocking_broken,
  count(*) filter (where coalesce(health->>'crash', '') <> '') as crashed,
  count(*) filter (where health->>'update' like '%failed%') as update_failed,
  count(*) filter (where public.effective_plan(accounts) = 'premium') as premium
from public.accounts
group by 1
order by 2 desc;

create or replace view public.admin_recent_problems as
select install_id, app_version, last_seen,
       health->>'os' as os, health->>'hosts_error' as hosts_error,
       health->>'crash' as crash, health->>'update' as update_status
from public.accounts
where (health->>'hosts_ok') = 'false'
   or coalesce(health->>'crash', '') <> ''
   or health->>'update' like '%failed%'
order by last_seen desc nulls last
limit 500;

revoke all on public.admin_health_summary from anon, authenticated;
revoke all on public.admin_recent_problems from anon, authenticated;
