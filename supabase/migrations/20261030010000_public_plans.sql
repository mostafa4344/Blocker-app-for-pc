-- (2026-10-30) The website's "Get Premium" page shows the same plans and
-- prices as the app (Table Editor -> plans), read through this function.
-- Only public information: plan code, months, price, name, popular.
-- The page sends purchases through submit_purchase_request() (same checks as
-- the app); the table itself is not writable with the public key.
create or replace function public.get_plans() returns json
language sql stable security definer set search_path = public as $$
  select coalesce(json_agg(json_build_object(
           'code', public.plan_code(months),
           'months', months,
           'price', btrim(price, E' \t\r\n'),
           'name', btrim(name, E' \t\r\n'),
           'popular', popular) order by sort), '[]'::json)
    from public.plans where enabled
$$;
revoke all on function public.get_plans() from public;
grant execute on function public.get_plans() to anon, authenticated;
