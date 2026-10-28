//! Ready-made block lists offered during onboarding.

use serde::{Deserialize, Serialize};

use crate::matcher::SiteRule;
use crate::model::{BlockList, Config};

#[derive(Debug, Clone, Serialize)]
pub struct Preset {
    pub id: &'static str,
    pub name_en: &'static str,
    pub name_bn: &'static str,
    pub icon: &'static str,
    pub sites: &'static [&'static str],
    pub apps: &'static [&'static str],
    pub keywords: &'static [&'static str],
}

pub const PRESETS: &[Preset] = &[
    Preset {
        id: "social",
        name_en: "Social media",
        name_bn: "সোশ্যাল মিডিয়া",
        icon: "💬",
        sites: &[
            "facebook.com",
            "instagram.com",
            "tiktok.com",
            "x.com",
            "twitter.com",
            "reddit.com",
            "snapchat.com",
            "pinterest.com",
            "threads.net",
            "tumblr.com",
            "linkedin.com/feed",
            "messenger.com",
            "bsky.app",
        ],
        apps: &[
            "facebook.exe",
            "instagram.exe",
            "tiktok.exe",
            "messenger.exe",
        ],
        keywords: &[],
    },
    Preset {
        id: "video",
        name_en: "Video & streaming",
        name_bn: "ভিডিও ও স্ট্রিমিং",
        icon: "🎬",
        sites: &[
            "youtube.com",
            "netflix.com",
            "twitch.tv",
            "primevideo.com",
            "disneyplus.com",
            "hotstar.com",
            "hoichoi.tv",
            "chorki.com",
            "bioscopelive.com",
            "vimeo.com",
            "kick.com",
        ],
        apps: &["netflix.exe", "vlc.exe"],
        keywords: &[],
    },
    Preset {
        id: "shorts",
        name_en: "Short videos only (Shorts, Reels)",
        name_bn: "শুধু শর্ট ভিডিও (Shorts, Reels)",
        icon: "📱",
        sites: &[
            "youtube.com/shorts",
            "instagram.com/reels",
            "instagram.com/reel",
            "facebook.com/reel",
            "facebook.com/reels",
            "tiktok.com",
        ],
        apps: &[],
        keywords: &[],
    },
    Preset {
        id: "gaming",
        name_en: "Games",
        name_bn: "গেমস",
        icon: "🎮",
        sites: &[
            "store.steampowered.com",
            "epicgames.com",
            "roblox.com",
            "miniclip.com",
            "poki.com",
            "crazygames.com",
        ],
        apps: &[
            "steam.exe",
            "epicgameslauncher.exe",
            "battle.net.exe",
            "riotclientservices.exe",
            "leagueclient.exe",
            "robloxplayerbeta.exe",
            "minecraft.exe",
            "minecraftlauncher.exe",
            "valorant.exe",
            "fortnitelauncher.exe",
            "origin.exe",
            "eadesktop.exe",
            "upc.exe",
            "Steam",
            "steam",
            "Minecraft",
        ],
        keywords: &[],
    },
    Preset {
        id: "chat",
        name_en: "Chat apps",
        name_bn: "চ্যাট অ্যাপ",
        icon: "✉️",
        sites: &[
            "web.whatsapp.com",
            "web.telegram.org",
            "discord.com",
            "messenger.com",
        ],
        apps: &[
            "discord.exe",
            "telegram.exe",
            "whatsapp.exe",
            "Discord",
            "Telegram",
            "WhatsApp",
        ],
        keywords: &[],
    },
    Preset {
        id: "news",
        name_en: "News",
        name_bn: "খবর",
        icon: "📰",
        sites: &[
            "news.google.com",
            "cnn.com",
            "bbc.com",
            "prothomalo.com",
            "bdnews24.com",
            "thedailystar.net",
            "jugantor.com",
            "kalerkantho.com",
            "news.ycombinator.com",
        ],
        apps: &[],
        keywords: &[],
    },
    Preset {
        id: "shopping",
        name_en: "Shopping",
        name_bn: "শপিং",
        icon: "🛒",
        sites: &[
            "amazon.com",
            "daraz.com.bd",
            "aliexpress.com",
            "ebay.com",
            "temu.com",
            "shein.com",
            "rokomari.com",
        ],
        apps: &[],
        keywords: &[],
    },
    Preset {
        id: "adult",
        name_en: "Adult content",
        name_bn: "প্রাপ্তবয়স্ক কন্টেন্ট",
        icon: "🔞",
        // A curated list of well-known adult sites. On supported browsers the
        // keyword scan catches many more by URL; full category coverage still
        // needs DNS/category filtering (planned — see NOTES.md).
        sites: &[
            "pornhub.com",
            "xvideos.com",
            "xnxx.com",
            "xhamster.com",
            "onlyfans.com",
            "chaturbate.com",
            "stripchat.com",
            "redtube.com",
            "youporn.com",
            "tube8.com",
            "spankbang.com",
            "youjizz.com",
            "brazzers.com",
            "bongacams.com",
            "livejasmin.com",
            "cam4.com",
            "myfreecams.com",
            "fansly.com",
            "rule34.xxx",
            "e-hentai.org",
            "nhentai.net",
            "hanime.tv",
            "hentaihaven.xxx",
            "motherless.com",
            "beeg.com",
            "eporner.com",
            "porn.com",
            "hqporner.com",
            "txxx.com",
            "fapello.com",
            "erome.com",
            "porntrex.com",
            "thumbzilla.com",
            "4tube.com",
            "ashleymadison.com",
            "adultfriendfinder.com",
        ],
        apps: &[],
        keywords: &[
            "porn",
            "xxx",
            "nsfw",
            "hentai",
            "sex-",
            "-sex",
            "camgirl",
            "escort",
            "milf",
            "blowjob",
            "hardcore-",
            "xxxvideo",
            "adult-video",
            "nude-",
        ],
    },
    Preset {
        id: "gambling",
        name_en: "Gambling & betting",
        name_bn: "জুয়া ও বাজি",
        icon: "🎰",
        sites: &[
            "bet365.com",
            "1xbet.com",
            "melbet.com",
            "22bet.com",
            "betway.com",
            "stake.com",
            "pokerstars.com",
            "888casino.com",
            "bovada.lv",
            "draftkings.com",
            "fanduel.com",
            "williamhill.com",
            "paripesa.com",
            "parimatch.com",
            "dafabet.com",
            "betwinner.com",
            "casino.com",
            "roobet.com",
        ],
        apps: &[],
        keywords: &["casino", "betting", "/bet-", "sportsbook", "pokerstars"],
    },
    Preset {
        id: "dating",
        name_en: "Dating apps",
        name_bn: "ডেটিং অ্যাপ",
        icon: "💘",
        sites: &[
            "tinder.com",
            "bumble.com",
            "badoo.com",
            "okcupid.com",
            "match.com",
            "hinge.co",
            "grindr.com",
            "plentyoffish.com",
            "meetme.com",
            "tagged.com",
        ],
        apps: &["tinder.exe", "bumble.exe"],
        keywords: &[],
    },
    Preset {
        id: "ai",
        name_en: "AI chat & homework help",
        name_bn: "AI চ্যাট ও হোমওয়ার্ক",
        icon: "🤖",
        sites: &[
            "chatgpt.com",
            "chat.openai.com",
            "gemini.google.com",
            "claude.ai",
            "copilot.microsoft.com",
            "perplexity.ai",
            "character.ai",
            "poe.com",
            "chegg.com",
            "coursehero.com",
            "quillbot.com",
        ],
        apps: &[],
        keywords: &[],
    },
];

pub fn preset(id: &str) -> Option<&'static Preset> {
    PRESETS.iter().find(|p| p.id == id)
}

impl Preset {
    pub fn to_block_list(&self, language: &str) -> BlockList {
        let own = |v: &[&str]| v.iter().map(|s| s.to_string()).collect();
        BlockList {
            id: crate::new_id(),
            name: if language == "bn" {
                self.name_bn
            } else {
                self.name_en
            }
            .to_string(),
            sites: own(self.sites),
            apps: own(self.apps),
            keywords: own(self.keywords),
            allow_sites: Vec::new(),
            allow_apps: Vec::new(),
            allowlist_mode: false,
            // Content nobody needs "sometimes" is blocked around the clock.
            always_on: self.always_on_by_default(),
            preset: self.id.to_string(),
        }
    }

    /// Presets that are blocked 24/7 by default (harmful content, not distractions).
    pub fn always_on_by_default(&self) -> bool {
        matches!(self.id, "adult" | "gambling")
    }
}

/// A ready-made list definition that can come from the server (the
/// `preset_lists` table an admin edits in Supabase) or from the built-ins.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
#[serde(default)]
pub struct PresetDef {
    pub id: String,
    pub name_en: String,
    pub name_bn: String,
    pub icon: String,
    pub sites: Vec<String>,
    pub apps: Vec<String>,
    pub keywords: Vec<String>,
    /// Lists from this preset are blocked 24/7.
    pub always_on: bool,
}

impl From<&Preset> for PresetDef {
    fn from(p: &Preset) -> Self {
        let own = |v: &[&str]| v.iter().map(|s| s.to_string()).collect();
        PresetDef {
            id: p.id.into(),
            name_en: p.name_en.into(),
            name_bn: p.name_bn.into(),
            icon: p.icon.into(),
            sites: own(p.sites),
            apps: own(p.apps),
            keywords: own(p.keywords),
            always_on: p.always_on_by_default(),
        }
    }
}

/// The presets compiled into the app (used offline / before the first sync).
pub fn builtin_defs() -> Vec<PresetDef> {
    PRESETS.iter().map(PresetDef::from).collect()
}

/// Limits for server data, so a mistake (or a compromised server) can't
/// flood the hosts file or block the whole web.
const MAX_ITEMS: usize = 20_000;
const MAX_ITEM_LEN: usize = 200;

fn clean(v: Vec<String>, ok: impl Fn(&str) -> bool) -> Vec<String> {
    let mut out: Vec<String> = Vec::new();
    for x in v {
        let x = x.trim().to_string();
        if !x.is_empty()
            && x.len() <= MAX_ITEM_LEN
            && !x.chars().any(char::is_control)
            && ok(&x)
            && !out.iter().any(|o| o.eq_ignore_ascii_case(&x))
            && out.len() < MAX_ITEMS
        {
            out.push(x);
        }
    }
    out
}

impl PresetDef {
    pub fn to_block_list(&self, language: &str) -> BlockList {
        BlockList {
            id: crate::new_id(),
            name: if language == "bn" && !self.name_bn.is_empty() {
                self.name_bn.clone()
            } else {
                self.name_en.clone()
            },
            sites: self.sites.clone(),
            apps: self.apps.clone(),
            keywords: self.keywords.clone(),
            allow_sites: Vec::new(),
            allow_apps: Vec::new(),
            allowlist_mode: false,
            always_on: self.always_on,
            preset: self.id.clone(),
        }
    }

    fn total(&self) -> usize {
        self.sites.len() + self.apps.len() + self.keywords.len()
    }

    /// Cleans a definition received from the server. Returns `None` if it is
    /// unusable. Invalid entries are dropped; "block everything" (`*`) is never
    /// accepted from the server.
    pub fn sanitized(mut self) -> Option<PresetDef> {
        let id_ok = !self.id.is_empty()
            && self.id.len() <= 40
            && self
                .id
                .chars()
                .all(|c| c.is_ascii_alphanumeric() || c == '_' || c == '-');
        if !id_ok || self.name_en.trim().is_empty() {
            return None;
        }
        self.sites = clean(self.sites, |s| {
            s != "*" && SiteRule::parse(s).is_some_and(|r| !r.domain.is_empty())
        });
        self.apps = clean(self.apps, |a| !a.contains(['/', '\\']));
        self.keywords = clean(self.keywords, |k| k.chars().count() >= 3);
        self.name_en = self.name_en.trim().chars().take(60).collect();
        self.name_bn = self.name_bn.trim().chars().take(60).collect();
        self.icon = self.icon.trim().chars().take(4).collect();
        Some(self)
    }
}

/// Whether a server update looks sane compared with what we had: an update
/// that would remove more than half of a list is ignored (most likely a
/// mistake while editing the table).
pub fn plausible_update(prev: &PresetDef, new: &PresetDef) -> bool {
    new.total() * 2 >= prev.total()
}

/// Brings the user's lists made from preset `new.id` up to date: items the
/// admin added appear, items the admin removed disappear, and anything the
/// user added themselves is kept. `prev` is the definition the lists were
/// last synced with. Lists are only ever switched *to* always-on. Returns
/// true if the config changed.
pub fn apply_update(config: &mut Config, prev: &PresetDef, new: &PresetDef) -> bool {
    fn merge(current: &[String], prev: &[String], new: &[String]) -> Vec<String> {
        let has = |v: &[String], x: &str| v.iter().any(|y| y.eq_ignore_ascii_case(x));
        let mut out: Vec<String> = new.to_vec();
        for x in current {
            // Kept if the user added it (it was never part of the preset).
            if !has(prev, x) && !has(&out, x) {
                out.push(x.clone());
            }
        }
        out
    }
    let mut changed = false;
    for list in config.block_lists.iter_mut().filter(|b| b.preset == new.id) {
        let sites = merge(&list.sites, &prev.sites, &new.sites);
        let apps = merge(&list.apps, &prev.apps, &new.apps);
        let keywords = merge(&list.keywords, &prev.keywords, &new.keywords);
        let always_on = list.always_on || new.always_on;
        if sites != list.sites
            || apps != list.apps
            || keywords != list.keywords
            || always_on != list.always_on
        {
            list.sites = sites;
            list.apps = apps;
            list.keywords = keywords;
            list.always_on = always_on;
            changed = true;
        }
    }
    changed
}

/// Upgrades a config saved by an older version. Returns true if it changed.
///
/// Version 1 had no `always_on` / `preset` on block lists, so a list the user
/// created from the Adult or Gambling preset blocked nothing unless a session
/// or schedule was running. Those lists are recognised (by name in either
/// language, or by their tell-tale keywords) and made always-on, which is what
/// the user picked them for.
pub fn migrate(config: &mut crate::model::Config) -> bool {
    if config.version >= crate::model::CONFIG_VERSION {
        return false;
    }
    for list in config.block_lists.iter_mut() {
        if list.preset.is_empty() {
            if let Some(p) = PRESETS
                .iter()
                .find(|p| list.name == p.name_en || list.name == p.name_bn)
            {
                list.preset = p.id.to_string();
            } else if list.keywords.iter().any(|k| k == "porn" || k == "xxx") {
                list.preset = "adult".to_string();
            }
        }
        if let Some(p) = preset(&list.preset) {
            if p.always_on_by_default() {
                list.always_on = true;
            }
        }
    }
    config.version = crate::model::CONFIG_VERSION;
    true
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::matcher::SiteRule;

    #[test]
    fn server_update_merges_and_keeps_user_items() {
        let prev = PresetDef {
            id: "adult".into(),
            name_en: "Adult".into(),
            sites: vec!["a.com".into(), "b.com".into(), "c.com".into()],
            ..Default::default()
        };
        let mut list = prev.to_block_list("en");
        list.sites.push("mine.com".into()); // added by the user
        let mut c = Config {
            block_lists: vec![list],
            ..Default::default()
        };
        // Admin removed b.com, added d.com and made it always-on.
        let new = PresetDef {
            sites: vec!["a.com".into(), "c.com".into(), "d.com".into()],
            always_on: true,
            ..prev.clone()
        };
        assert!(plausible_update(&prev, &new));
        assert!(apply_update(&mut c, &prev, &new));
        let l = &c.block_lists[0];
        assert_eq!(l.sites, vec!["a.com", "c.com", "d.com", "mine.com"]);
        assert!(l.always_on);
        assert!(!apply_update(&mut c, &new, &new), "second run is a no-op");
        // An update that wipes most of the list is rejected.
        let wiped = PresetDef {
            sites: vec!["a.com".into()],
            ..prev.clone()
        };
        assert!(!plausible_update(&new, &wiped));
    }

    #[test]
    fn server_presets_are_sanitized() {
        let raw = PresetDef {
            id: "adult".into(),
            name_en: "Adult".into(),
            sites: vec![
                "*".into(),
                "ok.com".into(),
                "OK.com".into(),
                "evil.com\n1.2.3.4 bank.com".into(),
                "".into(),
            ],
            apps: vec!["game.exe".into(), "C:\\x.exe".into()],
            keywords: vec!["xx".into(), "porn".into()],
            ..Default::default()
        };
        let p = raw.sanitized().unwrap();
        assert_eq!(p.sites, vec!["ok.com"]);
        assert_eq!(p.apps, vec!["game.exe"]);
        assert_eq!(p.keywords, vec!["porn"]);
        let bad_id = PresetDef {
            id: "../x".into(),
            name_en: "X".into(),
            ..Default::default()
        };
        assert!(bad_id.sanitized().is_none());
        assert_eq!(builtin_defs().len(), PRESETS.len());
    }

    #[test]
    fn migrate_v1_makes_harmful_presets_always_on() {
        use crate::model::{BlockList, Config};
        let mut c = Config {
            version: 1,
            block_lists: vec![
                BlockList {
                    id: "a".into(),
                    name: "Adult content".into(),
                    ..Default::default()
                },
                BlockList {
                    id: "b".into(),
                    name: "জুয়া ও বাজি".into(),
                    ..Default::default()
                },
                BlockList {
                    id: "c".into(),
                    name: "Social media".into(),
                    ..Default::default()
                },
                BlockList {
                    id: "d".into(),
                    name: "My own list".into(),
                    keywords: vec!["porn".into()],
                    ..Default::default()
                },
            ],
            ..Default::default()
        };
        assert!(migrate(&mut c));
        let on: Vec<bool> = c.block_lists.iter().map(|b| b.always_on).collect();
        assert_eq!(on, vec![true, true, false, true]);
        assert_eq!(c.block_lists[2].preset, "social");
        // Runs only once.
        c.block_lists[0].always_on = false;
        assert!(!migrate(&mut c));
        assert!(!c.block_lists[0].always_on);
    }

    #[test]
    fn presets_parse() {
        for p in PRESETS {
            for s in p.sites {
                assert!(SiteRule::parse(s).is_some(), "{} in {}", s, p.id);
            }
        }
        assert_eq!(
            preset("social").unwrap().to_block_list("bn").name,
            "সোশ্যাল মিডিয়া"
        );
    }
}
