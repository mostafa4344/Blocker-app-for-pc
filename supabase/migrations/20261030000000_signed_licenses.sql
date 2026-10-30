-- (2026-10-30) SIGNED LICENSES + DEVICE BINDING + CLOSED HOLES
--
-- Why: the app used to trust whatever plan the server (or its own license.json)
-- said. Someone could edit license.json, block the server, and keep Premium.
--
-- Now every license answer carries a TOKEN signed with an Ed25519 key that
-- lives only in this database (Supabase Vault). The app verifies it with the
-- public half built into the program, so a hand-edited file or a fake server
-- can't produce Premium. The token is also bound to a secret that never
-- leaves the customer's PC (device_key), so copying the license to another
-- computer doesn't work either.
--
-- Support: if a paying customer reinstalls Windows/the app and the app says
-- "linked to another computer", empty that device's device_key cell in
-- Table Editor -> accounts. The next check links the new install.
--
-- Also: removes the test codes that were printed in setup.sql, closes direct
-- table writes the app never needed, and rate-limits code guessing.
-- Safe to run more than once.

create extension if not exists pgsodium;

-- ---------------------------------------------------------------------------
-- 1. Holes
-- ---------------------------------------------------------------------------
-- Public test codes (anyone reading the repo could redeem lifetime Premium).
delete from public.codes
 where code in ('PREMIUM-LIFETIME', 'PREMIUM-30D', 'PREMIUM-365D') and redeemed_by is null;

-- The app only uses RPC functions. Take away direct table access as a second
-- wall behind RLS (anon could insert purchase requests directly, skipping the
-- spam checks, and even mark them "approved").
drop policy if exists "anon insert purchase requests" on public.purchase_requests;
revoke all on table public.accounts, public.codes, public.purchase_requests from anon, authenticated;

-- The old redeem function has no rate limit; the app now uses license_redeem.
revoke all on function public.redeem_code(text, text) from public, anon, authenticated;

-- ---------------------------------------------------------------------------
-- 2. The signing key (created once, kept in Vault, never leaves the database)
-- ---------------------------------------------------------------------------
do $$
begin
  if not exists (select 1 from vault.secrets where name = 'license_signing_seed') then
    perform vault.create_secret(
      encode(pgsodium.crypto_sign_new_seed(), 'hex'),
      'license_signing_seed',
      'Ed25519 seed that signs app licenses. Never share, change or delete it: installed apps only trust this key.');
  end if;
end $$;

create or replace function public.license_secret() returns bytea
language sql stable security definer set search_path = public as $$
  select (pgsodium.crypto_sign_seed_new_keypair(decode(decrypted_secret, 'hex'))).secret
    from vault.decrypted_secrets where name = 'license_signing_seed'
$$;
revoke all on function public.license_secret() from public, anon, authenticated;

-- Public half (hex) — this is what is built into the app.
create or replace function public.license_public_key() returns text
language sql stable security definer set search_path = public as $$
  select encode((pgsodium.crypto_sign_seed_new_keypair(decode(decrypted_secret, 'hex'))).public, 'hex')
    from vault.decrypted_secrets where name = 'license_signing_seed'
$$;
revoke all on function public.license_public_key() from public, anon, authenticated;

-- ---------------------------------------------------------------------------
-- 3. Device binding
-- ---------------------------------------------------------------------------
alter table public.accounts add column if not exists device_key text;
alter table public.accounts add column if not exists device_key_at timestamptz;
alter table public.accounts drop constraint if exists accounts_device_key_check;
alter table public.accounts add constraint accounts_device_key_check
  check (device_key is null or device_key ~ '^[0-9a-f]{64}$');
comment on column public.accounts.device_key is
  'Set by the app on first contact. Support: empty this cell to move the license to a reinstalled PC.';

-- Links the device key on first sight; never replaces a different one.
create or replace function public.bind_device(p_install_id text, p_device_key text)
returns public.accounts language plpgsql security definer set search_path = public as $$
declare a public.accounts;
begin
  if p_device_key is null or p_device_key !~ '^[0-9a-f]{64}$' then
    raise exception 'bad device key' using errcode = '22023';
  end if;
  update public.accounts set device_key = p_device_key, device_key_at = now()
   where install_id = p_install_id and device_key is null;
  select * into a from public.accounts where install_id = p_install_id;
  if not found then raise exception 'not registered' using errcode = 'P0002'; end if;
  return a;
end $$;
revoke all on function public.bind_device(text, text) from public, anon, authenticated;

-- ---------------------------------------------------------------------------
-- 4. The signed token
--    "<payload hex>.<signature hex>", signature over
--    'htl-content-guard-license-v1' || LF || payload  (payload = JSON text).
-- ---------------------------------------------------------------------------
create or replace function public.license_token(a public.accounts) returns text
language plpgsql stable security definer set search_path = public as $$
declare payload text;
begin
  payload := json_build_object(
    'v', 1,
    'id', a.install_id,
    'dk', a.device_key,
    'plan', public.effective_plan(a),
    'trial_started', a.trial_started,
    'trial_days', a.trial_days,
    'premium_until', a.premium_until,
    'iat', now())::text;
  return encode(convert_to(payload, 'UTF8'), 'hex') || '.' || encode(
    pgsodium.crypto_sign_detached(
      convert_to('htl-content-guard-license-v1' || chr(10) || payload, 'UTF8'),
      public.license_secret()),
    'hex');
end $$;
revoke all on function public.license_token(public.accounts) from public, anon, authenticated;

-- The usual license answer plus the token. A different computer asking with
-- this Device ID gets no token, only device_mismatch = true.
create or replace function public.signed_license(a public.accounts, p_device_key text) returns json
language sql stable security definer set search_path = public as $$
  select (public.license_json(a)::jsonb || jsonb_build_object(
    'token', case when a.device_key = p_device_key then public.license_token(a) end,
    'device_mismatch', a.device_key is distinct from p_device_key))::json
$$;
revoke all on function public.signed_license(public.accounts, text) from public, anon, authenticated;

-- ---------------------------------------------------------------------------
-- 5. What the app calls (with the public key)
-- ---------------------------------------------------------------------------
create or replace function public.license_register(
  p_install_id text, p_device_key text, p_os text default '', p_version text default ''
) returns json language plpgsql security definer set search_path = public as $$
declare a public.accounts;
begin
  perform public.register_device(p_install_id, p_os, p_version, 4);
  a := public.bind_device(p_install_id, p_device_key);
  return public.signed_license(a, p_device_key);
end $$;

create or replace function public.license_check(
  p_install_id text, p_device_key text, p_version text default null, p_health jsonb default null
) returns json language plpgsql security definer set search_path = public as $$
declare a public.accounts;
begin
  -- Records version / last seen / health exactly like before.
  perform public.get_license(p_install_id, p_version, p_health);
  a := public.bind_device(p_install_id, p_device_key);
  return public.signed_license(a, p_device_key);
end $$;

-- Failed code attempts, to stop guessing (per device and per IP address).
create table if not exists public.redeem_attempts (
  id         bigint generated always as identity primary key,
  install_id text not null,
  ip         text not null default '',
  at         timestamptz not null default now()
);
create index if not exists redeem_attempts_device on public.redeem_attempts (install_id, at);
create index if not exists redeem_attempts_ip on public.redeem_attempts (ip, at);
alter table public.redeem_attempts enable row level security;
revoke all on table public.redeem_attempts from anon, authenticated;

create or replace function public.license_redeem(p_install_id text, p_device_key text, p_code text)
returns json language plpgsql security definer set search_path = public as $$
declare
  a public.accounts; c public.codes;
  v_code text := upper(trim(coalesce(p_code, '')));
  v_ip text := coalesce(split_part(
    (nullif(current_setting('request.headers', true), '')::json) ->> 'x-forwarded-for', ',', 1), '');
begin
  a := public.bind_device(p_install_id, p_device_key);
  if a.device_key <> p_device_key then
    return public.signed_license(a, p_device_key);
  end if;
  -- Errors are returned (not raised) so the failed attempt is still recorded.
  if (select count(*) from public.redeem_attempts
       where install_id = p_install_id and at > now() - interval '1 hour') >= 10
     or (v_ip <> '' and (select count(*) from public.redeem_attempts
       where ip = v_ip and at > now() - interval '1 hour') >= 30) then
    return json_build_object('error', 'too many attempts, try again in an hour');
  end if;
  select * into c from public.codes where code = v_code for update;
  if not found or c.redeemed_by is not null then
    delete from public.redeem_attempts where at < now() - interval '1 day';
    insert into public.redeem_attempts (install_id, ip) values (p_install_id, left(v_ip, 64));
    return json_build_object('error', 'invalid or used code');
  end if;
  update public.codes set redeemed_by = a.install_id, redeemed_at = now() where code = c.code;
  update public.accounts set plan = 'premium',
    premium_until = case
      when c.days is null then null
      when plan = 'premium' and premium_until is null then null
      else greatest(coalesce(premium_until, now()), now()) + make_interval(days => c.days)
    end,
    updated_at = now()
   where install_id = a.install_id returning * into a;
  return public.signed_license(a, p_device_key);
end $$;

revoke all on function public.license_register(text, text, text, text) from public;
revoke all on function public.license_check(text, text, text, jsonb) from public;
revoke all on function public.license_redeem(text, text, text) from public;
grant execute on function public.license_register(text, text, text, text) to anon, authenticated;
grant execute on function public.license_check(text, text, text, jsonb) to anon, authenticated;
grant execute on function public.license_redeem(text, text, text) to anon, authenticated;
