-- ============================================================================
-- FamilyGuard — data retention (privacy: keep only what we need, for a while)
-- ============================================================================
-- Run AFTER schema.sql. Deletes old telemetry on a schedule. Retention windows
-- match docs/07-PRIVACY-POLICY.md; adjust to your final policy + local law.
--
-- Uses pg_cron (available on Supabase). If pg_cron is not enabled, enable it in
-- Dashboard → Database → Extensions, or just run familyguard_cleanup() manually.
-- ============================================================================

create extension if not exists pg_cron;

-- Retention windows (days)
--   usage_events : 90   (daily per-app summaries)
--   locations    : 30   (opt-in points)
--   heartbeats   : 14   (status history; latest status also lives on devices)
create or replace function familyguard_cleanup()
returns void language plpgsql security definer set search_path = public as $$
begin
  delete from usage_events where day        < (now() at time zone 'utc')::date - 90;
  delete from locations    where occurred_at < now() - interval '30 days';
  delete from heartbeats   where created_at  < now() - interval '14 days';
  -- Also drop consumed/expired enrollment tokens (they are single-use anyway).
  delete from enrollment_tokens
    where consumed_at is not null or expires_at < now() - interval '1 day';
end; $$;

-- Schedule daily at 03:17 UTC. Safe to re-run: drop any existing job first.
do $$
begin
  perform cron.unschedule('familyguard_cleanup');
exception when others then
  null; -- job did not exist yet
end $$;

select cron.schedule('familyguard_cleanup', '17 3 * * *', $$select familyguard_cleanup();$$);
