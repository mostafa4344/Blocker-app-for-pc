-- SECURITY FIX (2026-10-25): the trial length must be decided by the server.
-- Before this, register_device trusted p_trial_days from the client (1..365),
-- so anyone calling the public RPC could give a new device a 365-day trial.
-- p_trial_days is kept in the signature (old app versions still send it) but
-- is now ignored. Safe to run more than once.
--
-- HOW TO APPLY: Supabase dashboard -> SQL Editor -> paste this file -> Run.

create or replace function public.register_device(
  p_install_id text, p_os text default '', p_version text default '', p_trial_days int default 4
) returns json language plpgsql security definer set search_path = public as $$
declare a public.accounts;
begin
  if p_install_id is null or char_length(p_install_id) < 16 then
    raise exception 'invalid install_id' using errcode = '22023';
  end if;
  insert into public.accounts (install_id, trial_days, os, version)
    values (p_install_id, 4, coalesce(left(p_os, 32), ''), coalesce(left(p_version, 32), ''))
    on conflict (install_id) do update set os = excluded.os, version = excluded.version, updated_at = now()
    returning * into a;
  return public.license_json(a);
end $$;

revoke all on function public.register_device(text,text,text,int) from public;
grant execute on function public.register_device(text,text,text,int) to anon, authenticated;

-- Clamp any trial that was already registered with a longer length.
update public.accounts set trial_days = 4 where trial_days > 4;
