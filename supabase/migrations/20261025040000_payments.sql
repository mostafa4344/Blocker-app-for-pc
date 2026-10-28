-- (2026-10-25) Automatic payments: every paid order grants Premium exactly once.
-- Used by the `payment-webhook` Edge Function (Paddle and other gateways).
-- Safe to run more than once.
--
-- HOW TO APPLY: Supabase dashboard -> SQL Editor -> paste this file -> Run.

-- One row per payment event, so a gateway re-sending the same notification
-- never grants twice. Only the service role (the webhook) can read/write.
create table if not exists public.payments (
  event_id     text primary key,
  provider     text not null,
  install_id   text not null,
  product      text not null default '',
  days         int,                      -- null = lifetime
  amount       text not null default '',
  currency     text not null default '',
  status       text not null default 'granted',
  created_at   timestamptz not null default now(),
  raw          jsonb
);
create index if not exists payments_install_idx on public.payments (install_id, created_at desc);
alter table public.payments enable row level security;
revoke all on table public.payments from anon, authenticated;

-- Grants Premium for a payment. Renewing early ADDS the days to what is left
-- (admin_grant_premium starts from today, which would lose the remainder).
-- Lifetime (p_days null) wins over any subscription. Idempotent per event.
create or replace function public.grant_premium_for_payment(
  p_event_id text, p_provider text, p_install_id text, p_days int,
  p_product text default '', p_amount text default '', p_currency text default '',
  p_raw jsonb default null
) returns json language plpgsql security definer set search_path = public as $$
declare a public.accounts;
begin
  insert into public.payments (event_id, provider, install_id, product, days, amount, currency, raw)
  values (p_event_id, p_provider, p_install_id, coalesce(p_product, ''), p_days,
          coalesce(p_amount, ''), coalesce(p_currency, ''), p_raw)
  on conflict (event_id) do nothing;
  if not found then
    -- Already processed: just report the current state.
    select * into a from public.accounts where install_id = p_install_id;
    if not found then raise exception 'not registered' using errcode = 'P0002'; end if;
    return public.license_json(a);
  end if;

  update public.accounts set
    plan = 'premium',
    premium_until = case
      when p_days is null then null                                 -- lifetime
      when plan = 'premium' and premium_until is null then null     -- already lifetime
      else greatest(coalesce(premium_until, now()), now()) + make_interval(days => p_days)
    end,
    updated_at = now()
  where install_id = p_install_id
  returning * into a;
  if not found then
    -- Unknown device: the whole call (incl. the payment row) is rolled back,
    -- so the gateway retries and the admin sees it in the function logs.
    raise exception 'not registered' using errcode = 'P0002';
  end if;
  return public.license_json(a);
end $$;

revoke all on function public.grant_premium_for_payment(text,text,text,int,text,text,text,jsonb) from public, anon, authenticated;
grant execute on function public.grant_premium_for_payment(text,text,text,int,text,text,text,jsonb) to service_role;

-- Admin view: latest payments, and any that couldn't be matched to a device.
create or replace view public.admin_recent_payments as
select created_at, provider, install_id, product, days, amount, currency, status
from public.payments order by created_at desc limit 500;
revoke all on public.admin_recent_payments from anon, authenticated;
