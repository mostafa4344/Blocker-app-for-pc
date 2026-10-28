-- Ready-made block lists, maintained by the admin in Supabase.
-- (2026-10-25) The app downloads these every ~15 minutes and updates every
-- user's matching list: sites the admin adds appear, sites the admin removes
-- disappear, and anything the user added themselves is kept.
--
-- HOW TO APPLY: Supabase dashboard -> SQL Editor -> paste this file -> Run.
-- Safe to run more than once (existing rows are NOT overwritten).
--
-- HOW TO EDIT A LIST: Supabase dashboard -> Table Editor -> preset_lists ->
-- double-click the `sites` (or `keywords` / `apps`) cell of a row, edit, Save.
--   * id          never change it (the app links users' lists by this id)
--   * enabled     false hides the list from "Add a ready-made list"
--   * always_on   true = blocked 24/7 (lists are only ever switched ON by sync)
--   * sort_order  order shown in the app
-- Safety rules in the app: entries that aren't valid site names are ignored,
-- "*" (block the whole web) is never accepted, and an edit that removes more
-- than half of a list is ignored as a probable mistake.

create table if not exists public.preset_lists (
  id          text primary key check (id ~ '^[A-Za-z0-9_-]{1,40}$'),
  name_en     text not null,
  name_bn     text not null default '',
  icon        text not null default '',
  sites       text[] not null default '{}',
  apps        text[] not null default '{}',
  keywords    text[] not null default '{}',
  always_on   boolean not null default false,
  enabled     boolean not null default true,
  sort_order  int not null default 100,
  updated_at  timestamptz not null default now()
);

-- Only the admin (dashboard / service role) can read or change the table
-- directly; the app reads it through get_presets() below.
alter table public.preset_lists enable row level security;
revoke all on table public.preset_lists from anon, authenticated;

create or replace function public.preset_lists_touch() returns trigger
language plpgsql as $$
begin
  new.updated_at := now();
  return new;
end $$;

drop trigger if exists preset_lists_touch on public.preset_lists;
create trigger preset_lists_touch before update on public.preset_lists
  for each row execute function public.preset_lists_touch();

-- Read-only list for the app (anon key): enabled rows, in display order.
create or replace function public.get_presets()
returns json language sql stable security definer set search_path = public as $$
  select coalesce(json_agg(json_build_object(
           'id', id, 'name_en', name_en, 'name_bn', name_bn, 'icon', icon,
           'sites', sites, 'apps', apps, 'keywords', keywords,
           'always_on', always_on
         ) order by sort_order, id), '[]'::json)
  from public.preset_lists
  where enabled
$$;

revoke all on function public.get_presets() from public;
grant execute on function public.get_presets() to anon, authenticated;

-- Seed with the lists built into app version 1.1.0.
insert into public.preset_lists
  (id, name_en, name_bn, icon, sites, apps, keywords, always_on, sort_order)
values
  ('social', 'Social media', 'সোশ্যাল মিডিয়া', '💬', array['facebook.com', 'instagram.com', 'tiktok.com', 'x.com', 'twitter.com', 'reddit.com', 'snapchat.com', 'pinterest.com', 'threads.net', 'tumblr.com', 'linkedin.com/feed', 'messenger.com', 'bsky.app']::text[], array['facebook.exe', 'instagram.exe', 'tiktok.exe', 'messenger.exe']::text[], '{}'::text[], false, 10),
  ('video', 'Video & streaming', 'ভিডিও ও স্ট্রিমিং', '🎬', array['youtube.com', 'netflix.com', 'twitch.tv', 'primevideo.com', 'disneyplus.com', 'hotstar.com', 'hoichoi.tv', 'chorki.com', 'bioscopelive.com', 'vimeo.com', 'kick.com']::text[], array['netflix.exe', 'vlc.exe']::text[], '{}'::text[], false, 20),
  ('shorts', 'Short videos only (Shorts, Reels)', 'শুধু শর্ট ভিডিও (Shorts, Reels)', '📱', array['youtube.com/shorts', 'instagram.com/reels', 'instagram.com/reel', 'facebook.com/reel', 'facebook.com/reels', 'tiktok.com']::text[], '{}'::text[], '{}'::text[], false, 30),
  ('gaming', 'Games', 'গেমস', '🎮', array['store.steampowered.com', 'epicgames.com', 'roblox.com', 'miniclip.com', 'poki.com', 'crazygames.com']::text[], array['steam.exe', 'epicgameslauncher.exe', 'battle.net.exe', 'riotclientservices.exe', 'leagueclient.exe', 'robloxplayerbeta.exe', 'minecraft.exe', 'minecraftlauncher.exe', 'valorant.exe', 'fortnitelauncher.exe', 'origin.exe', 'eadesktop.exe', 'upc.exe', 'Steam', 'steam', 'Minecraft']::text[], '{}'::text[], false, 40),
  ('chat', 'Chat apps', 'চ্যাট অ্যাপ', '✉️', array['web.whatsapp.com', 'web.telegram.org', 'discord.com', 'messenger.com']::text[], array['discord.exe', 'telegram.exe', 'whatsapp.exe', 'Discord', 'Telegram', 'WhatsApp']::text[], '{}'::text[], false, 50),
  ('news', 'News', 'খবর', '📰', array['news.google.com', 'cnn.com', 'bbc.com', 'prothomalo.com', 'bdnews24.com', 'thedailystar.net', 'jugantor.com', 'kalerkantho.com', 'news.ycombinator.com']::text[], '{}'::text[], '{}'::text[], false, 60),
  ('shopping', 'Shopping', 'শপিং', '🛒', array['amazon.com', 'daraz.com.bd', 'aliexpress.com', 'ebay.com', 'temu.com', 'shein.com', 'rokomari.com']::text[], '{}'::text[], '{}'::text[], false, 70),
  ('adult', 'Adult content', 'প্রাপ্তবয়স্ক কন্টেন্ট', '🔞', array['pornhub.com', 'xvideos.com', 'xnxx.com', 'xhamster.com', 'onlyfans.com', 'chaturbate.com', 'stripchat.com', 'redtube.com', 'youporn.com', 'tube8.com', 'spankbang.com', 'youjizz.com', 'brazzers.com', 'bongacams.com', 'livejasmin.com', 'cam4.com', 'myfreecams.com', 'fansly.com', 'rule34.xxx', 'e-hentai.org', 'nhentai.net', 'hanime.tv', 'hentaihaven.xxx', 'motherless.com', 'beeg.com', 'eporner.com', 'porn.com', 'hqporner.com', 'txxx.com', 'fapello.com', 'erome.com', 'porntrex.com', 'thumbzilla.com', '4tube.com', 'ashleymadison.com', 'adultfriendfinder.com']::text[], '{}'::text[], array['porn', 'xxx', 'nsfw', 'hentai', 'sex-', '-sex', 'camgirl', 'escort', 'milf', 'blowjob', 'hardcore-', 'xxxvideo', 'adult-video', 'nude-']::text[], true, 80),
  ('gambling', 'Gambling & betting', 'জুয়া ও বাজি', '🎰', array['bet365.com', '1xbet.com', 'melbet.com', '22bet.com', 'betway.com', 'stake.com', 'pokerstars.com', '888casino.com', 'bovada.lv', 'draftkings.com', 'fanduel.com', 'williamhill.com', 'paripesa.com', 'parimatch.com', 'dafabet.com', 'betwinner.com', 'casino.com', 'roobet.com']::text[], '{}'::text[], array['casino', 'betting', '/bet-', 'sportsbook', 'pokerstars']::text[], true, 90),
  ('dating', 'Dating apps', 'ডেটিং অ্যাপ', '💘', array['tinder.com', 'bumble.com', 'badoo.com', 'okcupid.com', 'match.com', 'hinge.co', 'grindr.com', 'plentyoffish.com', 'meetme.com', 'tagged.com']::text[], array['tinder.exe', 'bumble.exe']::text[], '{}'::text[], false, 100),
  ('ai', 'AI chat & homework help', 'AI চ্যাট ও হোমওয়ার্ক', '🤖', array['chatgpt.com', 'chat.openai.com', 'gemini.google.com', 'claude.ai', 'copilot.microsoft.com', 'perplexity.ai', 'character.ai', 'poe.com', 'chegg.com', 'coursehero.com', 'quillbot.com']::text[], '{}'::text[], '{}'::text[], false, 110)
on conflict (id) do nothing;
