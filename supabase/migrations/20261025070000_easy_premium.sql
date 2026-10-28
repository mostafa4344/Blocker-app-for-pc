-- (2026-10-25) EASY PREMIUM — manage everything from the Table Editor, no SQL.
--
--   A) premium_grants    : "Insert row" -> type the customer's Device ID and
--                          months (empty = lifetime) -> Save. Premium turns on
--                          and the row shows the result.
--   B) purchase_requests : customers send bKash/Nagad/Rocket TrxIDs from inside
--                          the app. Check the TrxID in your bKash app, then set
--                          status = approved (dropdown). Premium turns on.
--   C) app_config        : your bKash/Nagad/Rocket numbers and prices, shown in
--                          the app's Premium page. Edit them any time.
--
-- Safe to run more than once. Needs the earlier scripts (… 20261025060000).

-- ---------------------------------------------------------------------------
-- Shared: give Premium to a device for N months (null = lifetime). Renewing
-- early adds the months to what is left. Returns the new end (null = lifetime).
-- ---------------------------------------------------------------------------
create or replace function public.give_premium(p_install_id text, p_months int)
returns timestamptz language plpgsql security definer set search_path = public as $$
declare v_until timestamptz;
begin
  update public.accounts set
    plan = 'premium',
    premium_until = case
      when p_months is null then null
      when plan = 'premium' and premium_until is null then null
      else greatest(coalesce(premium_until, now()), now()) + make_interval(months => p_months)
    end,
    updated_at = now()
  where install_id = p_install_id
  returning premium_until into v_until;
  if not found then raise exception 'device not found' using errcode = 'P0002'; end if;
  return v_until;
end $$;
revoke all on function public.give_premium(text, int) from public, anon, authenticated;

-- Finds a device from what the admin typed: the full Device ID, or (if unique)
-- its first 8+ characters. Returns null if none or several match.
create or replace function public.find_device(p_text text)
returns text language plpgsql stable security definer set search_path = public as $$
declare v text := lower(trim(coalesce(p_text, ''))); n int; id text;
begin
  if length(v) < 8 or v !~ '^[a-z0-9-]+$' then return null; end if;
  select count(*), min(install_id) into n, id from public.accounts where lower(install_id) like v || '%';
  if n = 1 then return id; end if;
  select install_id into id from public.accounts where lower(install_id) = v;
  return id;
end $$;
revoke all on function public.find_device(text) from public, anon, authenticated;

-- ---------------------------------------------------------------------------
-- A) premium_grants
-- ---------------------------------------------------------------------------
create table if not exists public.premium_grants (
  id            bigint generated always as identity primary key,
  device_id     text not null,
  months        int check (months is null or months between 1 and 120),
  note          text not null default '',
  created_at    timestamptz not null default now(),
  result        text,           -- filled automatically
  premium_until timestamptz     -- filled automatically (empty = lifetime)
);
comment on table public.premium_grants is
  'Insert row: Device ID + months (empty = lifetime) -> Premium turns on. result/premium_until fill themselves.';
comment on column public.premium_grants.months is 'How many months. Leave EMPTY for lifetime.';
alter table public.premium_grants enable row level security;
revoke all on table public.premium_grants from anon, authenticated;

create or replace function public.premium_grants_apply() returns trigger
language plpgsql security definer set search_path = public as $$
declare dev text;
begin
  new.device_id := trim(new.device_id);
  dev := public.find_device(new.device_id);
  if dev is null then
    new.result := '❌ Device ID not found — copy it again from the app''s Premium page (ডিভাইস আইডি পাওয়া যায়নি)';
    new.premium_until := null;
    return new;
  end if;
  new.device_id := dev;
  new.premium_until := public.give_premium(dev, new.months);
  new.result := case when new.months is null then '✅ Premium ON — lifetime (আজীবন)'
                     else '✅ Premium ON — ' || new.months || ' month(s) (মাস)' end;
  return new;
end $$;
revoke all on function public.premium_grants_apply() from public, anon, authenticated;
drop trigger if exists premium_grants_apply on public.premium_grants;
create trigger premium_grants_apply before insert on public.premium_grants
  for each row execute function public.premium_grants_apply();

-- ---------------------------------------------------------------------------
-- B) purchase_requests: status becomes a dropdown; approving turns Premium on.
-- ---------------------------------------------------------------------------
do $$ begin
  create type public.request_status as enum ('pending', 'approved', 'rejected');
exception when duplicate_object then null; end $$;

create table if not exists public.purchase_requests (
  id          uuid primary key default gen_random_uuid(),
  created_at  timestamptz not null default now(),
  install_id  text not null check (char_length(install_id) between 8 and 128),
  name        text not null default '',
  email       text,
  phone       text,
  plan        text not null check (plan in ('monthly','yearly','lifetime')),
  method      text not null,
  txn_id      text not null check (char_length(txn_id) between 3 and 80),
  status      public.request_status not null default 'pending',
  note        text
);
alter table public.purchase_requests enable row level security;

-- Older installs: text status -> dropdown; allow Rocket/Upay; name optional.
do $$ begin
  if (select data_type from information_schema.columns
      where table_schema = 'public' and table_name = 'purchase_requests' and column_name = 'status') = 'text' then
    alter table public.purchase_requests drop constraint if exists purchase_requests_status_check;
    alter table public.purchase_requests alter column status drop default;
    alter table public.purchase_requests alter column status type public.request_status
      using status::public.request_status;
    alter table public.purchase_requests alter column status set default 'pending';
  end if;
end $$;
alter table public.purchase_requests drop constraint if exists purchase_requests_method_check;
alter table public.purchase_requests add constraint purchase_requests_method_check
  check (method in ('bkash', 'nagad', 'rocket', 'upay', 'other'));
alter table public.purchase_requests drop constraint if exists purchase_requests_name_check;
alter table public.purchase_requests alter column name set default '';
alter table public.purchase_requests add column if not exists result text;
alter table public.purchase_requests add column if not exists premium_until timestamptz;
create index if not exists purchase_requests_status_idx on public.purchase_requests (status, created_at desc);
comment on column public.purchase_requests.status is
  'Check the TrxID in your bKash/Nagad app, then choose approved (Premium turns on) or rejected.';

create or replace function public.purchase_requests_apply() returns trigger
language plpgsql security definer set search_path = public as $$
declare months int;
begin
  if new.status = 'approved' and old.status is distinct from 'approved' then
    months := case new.plan when 'monthly' then 1 when 'yearly' then 12 else null end;
    begin
      new.premium_until := public.give_premium(new.install_id, months);
      new.result := '✅ Premium ON (' || new.plan || ')';
    exception when others then
      new.status := old.status;
      new.result := '❌ Device not found — Premium NOT given (ডিভাইস পাওয়া যায়নি)';
    end;
  elsif new.status = 'rejected' and old.status is distinct from 'rejected' then
    new.result := 'rejected';
  end if;
  return new;
end $$;
revoke all on function public.purchase_requests_apply() from public, anon, authenticated;
drop trigger if exists purchase_requests_apply on public.purchase_requests;
create trigger purchase_requests_apply before update on public.purchase_requests
  for each row execute function public.purchase_requests_apply();

-- The app sends requests through this (the Device ID comes from the app, so
-- customers can't mistype it). Limits spam: 5 open requests per device, and a
-- TrxID can't be reused once approved.
create or replace function public.submit_purchase_request(
  p_install_id text, p_plan text, p_method text, p_txn_id text, p_contact text default ''
) returns json language plpgsql security definer set search_path = public as $$
declare v_txn text := upper(trim(coalesce(p_txn_id, ''))); v_contact text := left(trim(coalesce(p_contact, '')), 120);
begin
  if not exists (select 1 from public.accounts where install_id = p_install_id) then
    raise exception 'not registered' using errcode = 'P0002';
  end if;
  if p_plan not in ('monthly', 'yearly', 'lifetime') then raise exception 'bad plan' using errcode = '22023'; end if;
  if p_method not in ('bkash', 'nagad', 'rocket', 'upay', 'other') then raise exception 'bad method' using errcode = '22023'; end if;
  if char_length(v_txn) not between 4 and 40 or v_txn !~ '^[A-Z0-9]+$' then
    raise exception 'bad txn' using errcode = '22023';
  end if;
  if exists (select 1 from public.purchase_requests where upper(txn_id) = v_txn and status = 'approved') then
    raise exception 'txn already used' using errcode = '22023';
  end if;
  if (select count(*) from public.purchase_requests where install_id = p_install_id and status = 'pending') >= 5 then
    raise exception 'too many pending' using errcode = '22023';
  end if;
  insert into public.purchase_requests (install_id, plan, method, txn_id, phone, name)
  values (p_install_id, p_plan, p_method, v_txn, nullif(v_contact, ''), '');
  return json_build_object('ok', true);
end $$;
revoke all on function public.submit_purchase_request(text, text, text, text, text) from public;
grant execute on function public.submit_purchase_request(text, text, text, text, text) to anon, authenticated;

-- ---------------------------------------------------------------------------
-- C) Payment details shown in the app (edit in app_config).
-- ---------------------------------------------------------------------------
insert into public.app_config (key, value) values
  ('bkash_number',   '01568689505'),
  ('nagad_number',   '01568689505'),
  ('rocket_number',  ''),
  ('price_monthly',  '৳199'),
  ('price_yearly',   '৳999'),
  ('price_lifetime', '৳1999')
on conflict (key) do nothing;

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
    'pay', json_build_object(
      'bkash',  public.app_config_value('bkash_number'),
      'nagad',  public.app_config_value('nagad_number'),
      'rocket', public.app_config_value('rocket_number'),
      'monthly',  public.app_config_value('price_monthly'),
      'yearly',   public.app_config_value('price_yearly'),
      'lifetime', public.app_config_value('price_lifetime')
    ),
    'accepts_health', true
  )
$$;
revoke all on function public.license_json(public.accounts) from public, anon, authenticated;
