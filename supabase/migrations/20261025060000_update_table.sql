-- (2026-10-25) In-app updates: "what's new" text shown to users, and a storage
-- bucket for the installers. Safe to run more than once.
-- Needs 20261025030000_releases_and_health.sql first.

-- What's new in this version — shown in the app's "Update available" message.
alter table public.app_releases add column if not exists notes_bn text not null default '';

create or replace function public.latest_release()
returns json language sql stable security definer set search_path = public as $$
  select json_build_object('version', version, 'url', url, 'sha256', sha256,
                           'size', size, 'signature', signature, 'rollout', rollout,
                           'notes', notes, 'notes_bn', notes_bn)
  from public.app_releases
  where enabled and rollout > 0
  order by string_to_array(version, '.')::int[] desc
  limit 1
$$;
revoke all on function public.latest_release() from public, anon, authenticated;

-- Public download folder for installers ("releases"). Only the admin
-- (publish script, secret key) can upload; anyone can download.
do $$
begin
  if exists (select 1 from pg_namespace where nspname = 'storage') then
    insert into storage.buckets (id, name, public, file_size_limit)
    values ('releases', 'releases', true, 104857600)
    on conflict (id) do update set public = true, file_size_limit = 104857600;
  end if;
end $$;
