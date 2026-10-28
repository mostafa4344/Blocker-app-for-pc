-- Ready-made lists become plain text to edit (no JSON, no quotes, no brackets).
--
-- Table Editor -> preset_lists -> click a cell of `sites`, `apps` or `keywords`
-- and just type, for example:
--   sites:     pornhub.com, xvideos.com, https://www.example.com/
--   apps:      steam.exe, Discord, Epic Games Launcher
--   keywords:  porn, xxx, casino
-- Separate entries with commas or new lines (sites may also use spaces).
-- Cleaned up automatically: http(s)://, www., trailing "/", quotes and
-- brackets (so pasting an old JSON list also works), duplicates, upper case
-- in sites/keywords. Every app on every PC gets the change within minutes.
--
-- Safe to run more than once.

-- 1) text[] columns -> plain text (existing entries kept, comma separated).
do $$
declare col text;
begin
  foreach col in array array['sites', 'apps', 'keywords'] loop
    if (select data_type from information_schema.columns
        where table_schema = 'public' and table_name = 'preset_lists' and column_name = col) = 'ARRAY' then
      execute format('alter table public.preset_lists alter column %I drop default', col);
      execute format('alter table public.preset_lists alter column %I type text using array_to_string(%I, '', '')', col, col);
      execute format('alter table public.preset_lists alter column %I set default ''''', col);
    end if;
  end loop;
end $$;

comment on column public.preset_lists.sites is
  'Websites, separated by commas or new lines. e.g. facebook.com, youtube.com/shorts';
comment on column public.preset_lists.apps is
  'Programs, separated by commas or new lines. e.g. steam.exe, Discord';
comment on column public.preset_lists.keywords is
  'Words blocked in any web address, separated by commas or new lines. e.g. porn, casino';

-- 2) Turns what was typed into a clean list. kind: 'site' | 'app' | 'word'.
create or replace function public.split_entries(p_text text, p_kind text)
returns text[] language sql immutable set search_path = public as $$
  with parts as (
    select ord, btrim(part, E' \t\r\n') as x
    from regexp_split_to_table(
           coalesce(p_text, ''),
           case when p_kind = 'site' then E'[,;\\s]+' else E'[,;\\r\\n]+' end
         ) with ordinality as t(part, ord)
  ),
  cleaned as (
    select ord,
      case p_kind
        when 'site' then regexp_replace(regexp_replace(regexp_replace(lower(
               regexp_replace(x, E'^[\\[\\]"''`]+|[\\[\\]"''`]+$', '', 'g')),
               '^[a-z]+://', ''), '^www\.', ''), '/+$', '')
        when 'word' then lower(regexp_replace(x, E'^[\\[\\]"''`]+|[\\[\\]"''`]+$', '', 'g'))
        else btrim(regexp_replace(x, E'^[\\[\\]"''`]+|[\\[\\]"''`]+$', '', 'g'))
      end as x
    from parts
  )
  select coalesce(array_agg(x order by first_ord), '{}')
  from (select x, min(ord) as first_ord from cleaned where x <> '' group by x) d
$$;
revoke all on function public.split_entries(text, text) from public, anon, authenticated;

-- 3) What the app downloads: same shape as before (arrays), built from the text.
create or replace function public.get_presets()
returns json language sql stable security definer set search_path = public as $$
  select coalesce(json_agg(json_build_object(
           'id', id, 'name_en', name_en, 'name_bn', name_bn, 'icon', icon,
           'sites', public.split_entries(sites, 'site'),
           'apps', public.split_entries(apps, 'app'),
           'keywords', public.split_entries(keywords, 'word'),
           'always_on', always_on
         ) order by sort_order, id), '[]'::json)
  from public.preset_lists
  where enabled
$$;
revoke all on function public.get_presets() from public;
grant execute on function public.get_presets() to anon, authenticated;
