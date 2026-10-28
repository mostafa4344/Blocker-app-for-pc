-- Plans the app offers, edited in Supabase Table Editor -> plans (no SQL).
-- One row per plan:
--   months   how long Premium lasts (EMPTY = lifetime)
--   price    what the app shows, e.g. ৳400
--   name     optional label; empty = automatic ("2 months", "Yearly"...)
--   popular  tick one plan to show it as "Popular" and pre-select it
--   enabled  untick to hide a plan without deleting it
--   sort     order in the app (smaller first)
-- A new row shows up in every app within minutes, and buying/approving it
-- gives exactly that many months. One row per duration (months is unique).

create table if not exists public.plans (
  id       bigint generated always as identity primary key,
  months   int check (months is null or months between 1 and 120),
  price    text not null check (char_length(price) between 1 and 40),
  name     text not null default '' check (char_length(name) <= 40),
  popular  boolean not null default false,
  enabled  boolean not null default true,
  sort     int not null default 100
);
create unique index if not exists plans_months_key on public.plans (coalesce(months, 0));
alter table public.plans enable row level security;
revoke all on public.plans from anon, authenticated;
comment on table public.plans is
  'Premium plans shown in the app. months EMPTY = lifetime. price is shown as written (e.g. ৳400).';
comment on column public.plans.months is 'How many months. Leave EMPTY for lifetime. One row per duration.';
comment on column public.plans.price is 'Shown in the app exactly as written, e.g. ৳400';
comment on column public.plans.name is 'Optional. Empty = automatic name (Monthly, 2 months, Yearly, Lifetime).';

-- The code a plan is known by in requests: monthly / yearly / lifetime for the
-- classic durations (older app versions send these), otherwise "<n>m".
create or replace function public.plan_code(p_months int) returns text
language sql immutable as $$
  select case when p_months is null then 'lifetime' when p_months = 1 then 'monthly'
              when p_months = 12 then 'yearly' else p_months || 'm' end
$$;

-- Months for a plan code; raises for anything that isn't a valid code.
create or replace function public.plan_months(p_code text) returns int
language plpgsql immutable as $$
begin
  if p_code = 'lifetime' then return null; end if;
  if p_code = 'monthly' then return 1; end if;
  if p_code = 'yearly' then return 12; end if;
  if p_code ~ '^[0-9]{1,3}m$' and left(p_code, -1)::int between 1 and 120 then
    return left(p_code, -1)::int;
  end if;
  raise exception 'bad plan' using errcode = '22023';
end $$;

-- Move the prices that were in app_config into the new table (once).
do $$
declare v text;
begin
  if not exists (select 1 from public.plans) then
    select value into v from public.app_config where key = 'price_monthly';
    if coalesce(v, '') <> '' then insert into public.plans (months, price, sort) values (1, v, 10); end if;
    select value into v from public.app_config where key = '2 Months';
    if coalesce(v, '') <> '' then insert into public.plans (months, price, sort) values (2, v, 20); end if;
    select value into v from public.app_config where key = 'price_yearly';
    if coalesce(v, '') <> '' then insert into public.plans (months, price, popular, sort) values (12, v, true, 30); end if;
    select value into v from public.app_config where key = 'price_lifetime';
    if coalesce(v, '') <> '' then insert into public.plans (months, price, sort) values (null, v, 40); end if;
  end if;
  -- Prices now live only in "plans", so there is one place to edit them.
  delete from public.app_config where key in ('price_monthly', 'price_yearly', 'price_lifetime', '2 Months');
end $$;

-- Requests may name any plan code; the trigger below checks it exists.
alter table public.purchase_requests drop constraint if exists purchase_requests_plan_check;
alter table public.purchase_requests add constraint purchase_requests_plan_check
  check (plan ~ '^(monthly|yearly|lifetime|[0-9]{1,3}m)$');

create or replace function public.purchase_requests_apply() returns trigger
language plpgsql security definer set search_path = public as $$
begin
  if new.status = 'approved' and old.status is distinct from 'approved' then
    begin
      new.premium_until := public.give_premium(new.install_id, public.plan_months(new.plan));
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

create or replace function public.submit_purchase_request(
  p_install_id text, p_plan text, p_method text, p_txn_id text, p_contact text default ''
) returns json language plpgsql security definer set search_path = public as $$
declare v_txn text := upper(trim(coalesce(p_txn_id, ''))); v_contact text := left(trim(coalesce(p_contact, '')), 120);
        v_months int;
begin
  if not exists (select 1 from public.accounts where install_id = p_install_id) then
    raise exception 'not registered' using errcode = 'P0002';
  end if;
  v_months := public.plan_months(coalesce(p_plan, ''));
  if not exists (select 1 from public.plans where enabled and months is not distinct from v_months) then
    raise exception 'bad plan' using errcode = '22023';
  end if;
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
  values (p_install_id, public.plan_code(v_months), p_method, v_txn, nullif(v_contact, ''), '');
  return json_build_object('ok', true);
end $$;
revoke all on function public.submit_purchase_request(text, text, text, text, text) from public;
grant execute on function public.submit_purchase_request(text, text, text, text, text) to anon, authenticated;

-- What the app receives: payment numbers + the plan list. The old fixed keys
-- (monthly/yearly/lifetime) are still sent so older app versions keep working.
create or replace function public.license_json(a public.accounts)
returns json language sql stable security definer set search_path = public as $$
  with p as (
    -- Spaces/new lines typed around a price or name in the Table Editor are dropped.
    select public.plan_code(months) as code, months, btrim(price, E' \t\r\n') as price,
           btrim(name, E' \t\r\n') as name, popular, sort
    from public.plans where enabled
  )
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
      'monthly',  (select price from p where months = 1),
      'yearly',   (select price from p where months = 12),
      'lifetime', (select price from p where months is null),
      'plans', coalesce((select json_agg(json_build_object(
                  'code', code, 'months', months, 'price', price, 'name', name, 'popular', popular)
                  order by sort, coalesce(months, 100000)) from p), '[]'::json)
    ),
    'accepts_health', true
  )
$$;
revoke all on function public.license_json(public.accounts) from public, anon, authenticated;
revoke all on function public.plan_code(int) from public, anon, authenticated;
revoke all on function public.plan_months(text) from public, anon, authenticated;
