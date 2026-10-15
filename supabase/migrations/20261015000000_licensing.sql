-- ContentGuard licensing schema for Supabase (Postgres).
-- The agent calls the SECURITY DEFINER RPCs below with the publishable (anon)
-- key. Tables have RLS enabled with no anon policies, so the anon key can only
-- act through these functions — it can never read or write rows directly.

create extension if not exists pgcrypto;

create table public.accounts (
  install_id    text primary key check (char_length(install_id) between 16 and 128),
  plan          text not null default 'trial' check (plan in ('trial', 'premium', 'expired')),
  trial_started timestamptz not null default now(),
  trial_days    int not null default 4 check (trial_days between 1 and 365),
  premium_until timestamptz,                 -- null + plan 'premium' = lifetime
  os            text not null default '',
  version       text not null default '',
  created_at    timestamptz not null default now(),
  updated_at    timestamptz not null default now()
);

-- Prepaid activation codes (created by the payment webhook / admin).
create table public.codes (
  code        text primary key,
  days        int check (days is null or days > 0),   -- null = lifetime
  redeemed_by text references public.accounts(install_id),
  redeemed_at timestamptz,
  created_at  timestamptz not null default now()
);

alter table public.accounts enable row level security;
alter table public.codes    enable row level security;
-- No policies for anon/authenticated: all access is via the RPCs below.

-- Collapses an expired trial to 'expired'; premium is sticky until its end.
create or replace function public.effective_plan(a public.accounts)
returns text language sql stable as $$
  select case
    when a.plan = 'premium' and (a.premium_until is null or a.premium_until > now()) then 'premium'
    when a.plan = 'premium' then 'expired'
    when now() < a.trial_started + make_interval(days => a.trial_days) then 'trial'
    else 'expired'
  end
$$;

create or replace function public.license_json(a public.accounts)
returns json language sql stable as $$
  select json_build_object(
    'plan', public.effective_plan(a),
    'trial_started', a.trial_started,
    'trial_days', a.trial_days,
    'premium_until', a.premium_until
  )
$$;

-- Registers an install (creating the trial on first contact) and returns status.
create or replace function public.register_device(
  p_install_id text, p_os text default '', p_version text default '', p_trial_days int default 4
) returns json
language plpgsql security definer set search_path = public as $$
declare a public.accounts;
begin
  if p_install_id is null or char_length(p_install_id) < 16 then
    raise exception 'invalid install_id' using errcode = '22023';
  end if;
  insert into public.accounts (install_id, trial_days, os, version)
    values (p_install_id, greatest(1, least(365, coalesce(p_trial_days, 4))), coalesce(p_os, ''), coalesce(p_version, ''))
    on conflict (install_id) do update
      set os = excluded.os, version = excluded.version, updated_at = now()
    returning * into a;
  return public.license_json(a);
end $$;

create or replace function public.get_license(p_install_id text)
returns json language plpgsql security definer set search_path = public as $$
declare a public.accounts;
begin
  select * into a from public.accounts where install_id = p_install_id;
  if not found then raise exception 'not registered' using errcode = 'P0002'; end if;
  return public.license_json(a);
end $$;

-- Redeems a one-time code, upgrading the account to premium.
create or replace function public.redeem_code(p_install_id text, p_code text)
returns json language plpgsql security definer set search_path = public as $$
declare a public.accounts; c public.codes;
begin
  select * into a from public.accounts where install_id = p_install_id;
  if not found then raise exception 'not registered' using errcode = 'P0002'; end if;
  select * into c from public.codes where code = upper(trim(p_code)) for update;
  if not found or c.redeemed_by is not null then
    raise exception 'invalid or used code' using errcode = '22023';
  end if;
  update public.codes set redeemed_by = a.install_id, redeemed_at = now() where code = c.code;
  update public.accounts set plan = 'premium',
    premium_until = case when c.days is null then null else now() + make_interval(days => c.days) end,
    updated_at = now()
    where install_id = a.install_id returning * into a;
  return public.license_json(a);
end $$;

-- The anon/publishable key may execute only these functions.
revoke all on function public.register_device(text, text, text, int) from public;
revoke all on function public.get_license(text) from public;
revoke all on function public.redeem_code(text, text) from public;
grant execute on function public.register_device(text, text, text, int) to anon, authenticated;
grant execute on function public.get_license(text) to anon, authenticated;
grant execute on function public.redeem_code(text, text) to anon, authenticated;
