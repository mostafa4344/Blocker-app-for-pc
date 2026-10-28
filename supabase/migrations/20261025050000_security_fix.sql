-- SECURITY FIX (2026-10-25) — run as soon as possible.
--
-- Supabase gives the public app roles (anon, authenticated) EXECUTE on every
-- new function directly. Earlier scripts only revoked it from PUBLIC, so the
-- admin functions were callable with the public app key: anyone could grant
-- themselves lifetime Premium (admin_grant_premium) or mint activation codes
-- (admin_create_code). Verified on the live project before this fix.
--
-- This revokes everything the app must not call, keeps exactly what it needs,
-- and stops new functions from being public by default. Safe to run again.

-- Admin-only (the dashboard / payment webhook use service_role).
revoke all on function public.admin_grant_premium(text, int) from public, anon, authenticated;
revoke all on function public.admin_create_code(text, int, text) from public, anon, authenticated;
grant execute on function public.admin_grant_premium(text, int) to service_role;
grant execute on function public.admin_create_code(text, int, text) to service_role;

-- Internal helpers (only called from inside other functions).
revoke all on function public.effective_plan(public.accounts) from public, anon, authenticated;
revoke all on function public.license_json(public.accounts) from public, anon, authenticated;
revoke all on function public.presets_version() from public, anon, authenticated;
revoke all on function public.latest_release() from public, anon, authenticated;
revoke all on function public.app_config_value(text) from public, anon, authenticated;
revoke all on function public.preset_lists_touch() from public, anon, authenticated;
revoke all on function public.grant_premium_for_payment(text, text, text, int, text, text, text, jsonb)
  from public, anon, authenticated;

-- What the app is allowed to call with the public key — and nothing else.
grant execute on function public.register_device(text, text, text, int) to anon, authenticated;
grant execute on function public.get_license(text, text, jsonb) to anon, authenticated;
grant execute on function public.redeem_code(text, text) to anon, authenticated;
grant execute on function public.get_presets() to anon, authenticated;

-- From now on new functions are NOT callable by the app unless granted on purpose.
alter default privileges in schema public revoke execute on functions from public, anon, authenticated;
