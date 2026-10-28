-- SCALE (2026-10-25): tell the app when the preset lists changed, so millions
-- of devices don't re-download unchanged lists. Every license answer now
-- carries a tiny `presets_version`; the app downloads get_presets() only when
-- it differs from the version it already has.
--
-- HOW TO APPLY: Supabase dashboard -> SQL Editor -> paste this file -> Run.
-- Safe to run more than once. Needs 20261025010000_preset_lists.sql first.

-- Changes whenever any enabled/disabled list is added, removed or edited.
create or replace function public.presets_version()
returns text language sql stable security definer set search_path = public as $$
  select coalesce(to_char(max(updated_at), 'YYYYMMDDHH24MISSUS'), '0') || '-' || count(*)::text
  from public.preset_lists
$$;

revoke all on function public.presets_version() from public;

create or replace function public.license_json(a public.accounts)
returns json language sql stable security definer set search_path = public as $$
  select json_build_object(
    'plan', public.effective_plan(a),
    'trial_started', a.trial_started,
    'trial_days', a.trial_days,
    'premium_until', a.premium_until,
    'presets_version', public.presets_version()
  )
$$;

revoke all on function public.license_json(public.accounts) from public;
