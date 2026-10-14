//! Ready-made block lists offered during onboarding.

use serde::Serialize;

use crate::model::BlockList;

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
            "facebook.com", "instagram.com", "tiktok.com", "x.com", "twitter.com", "reddit.com",
            "snapchat.com", "pinterest.com", "threads.net", "tumblr.com", "linkedin.com/feed",
            "messenger.com", "bsky.app",
        ],
        apps: &["facebook.exe", "instagram.exe", "tiktok.exe", "messenger.exe"],
        keywords: &[],
    },
    Preset {
        id: "video",
        name_en: "Video & streaming",
        name_bn: "ভিডিও ও স্ট্রিমিং",
        icon: "🎬",
        sites: &[
            "youtube.com", "netflix.com", "twitch.tv", "primevideo.com", "disneyplus.com",
            "hotstar.com", "hoichoi.tv", "chorki.com", "bioscopelive.com", "vimeo.com", "kick.com",
        ],
        apps: &["netflix.exe", "vlc.exe"],
        keywords: &[],
    },
    Preset {
        id: "shorts",
        name_en: "Short videos only (Shorts, Reels)",
        name_bn: "শুধু শর্ট ভিডিও (Shorts, Reels)",
        icon: "📱",
        sites: &["youtube.com/shorts", "instagram.com/reels", "instagram.com/reel", "facebook.com/reel", "facebook.com/reels", "tiktok.com"],
        apps: &[],
        keywords: &[],
    },
    Preset {
        id: "gaming",
        name_en: "Games",
        name_bn: "গেমস",
        icon: "🎮",
        sites: &["store.steampowered.com", "epicgames.com", "roblox.com", "miniclip.com", "poki.com", "crazygames.com"],
        apps: &[
            "steam.exe", "epicgameslauncher.exe", "battle.net.exe", "riotclientservices.exe",
            "leagueclient.exe", "robloxplayerbeta.exe", "minecraft.exe", "minecraftlauncher.exe",
            "valorant.exe", "fortnitelauncher.exe", "origin.exe", "eadesktop.exe", "upc.exe",
            "Steam", "steam", "Minecraft",
        ],
        keywords: &[],
    },
    Preset {
        id: "chat",
        name_en: "Chat apps",
        name_bn: "চ্যাট অ্যাপ",
        icon: "✉️",
        sites: &["web.whatsapp.com", "web.telegram.org", "discord.com", "messenger.com"],
        apps: &["discord.exe", "telegram.exe", "whatsapp.exe", "Discord", "Telegram", "WhatsApp"],
        keywords: &[],
    },
    Preset {
        id: "news",
        name_en: "News",
        name_bn: "খবর",
        icon: "📰",
        sites: &[
            "news.google.com", "cnn.com", "bbc.com", "prothomalo.com", "bdnews24.com",
            "thedailystar.net", "jugantor.com", "kalerkantho.com", "news.ycombinator.com",
        ],
        apps: &[],
        keywords: &[],
    },
    Preset {
        id: "shopping",
        name_en: "Shopping",
        name_bn: "শপিং",
        icon: "🛒",
        sites: &["amazon.com", "daraz.com.bd", "aliexpress.com", "ebay.com", "temu.com", "shein.com", "rokomari.com"],
        apps: &[],
        keywords: &[],
    },
    Preset {
        id: "adult",
        name_en: "Adult content",
        name_bn: "প্রাপ্তবয়স্ক কন্টেন্ট",
        icon: "🔞",
        // A short seed list; category DNS filtering (see NOTES.md) gives full coverage.
        sites: &["pornhub.com", "xvideos.com", "xnxx.com", "xhamster.com", "onlyfans.com", "chaturbate.com", "stripchat.com"],
        apps: &[],
        keywords: &["porn", "xxx", "nsfw", "hentai"],
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
            name: if language == "bn" { self.name_bn } else { self.name_en }.to_string(),
            sites: own(self.sites),
            apps: own(self.apps),
            keywords: own(self.keywords),
            allowlist_mode: false,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::matcher::SiteRule;

    #[test]
    fn presets_parse() {
        for p in PRESETS {
            for s in p.sites {
                assert!(SiteRule::parse(s).is_some(), "{} in {}", s, p.id);
            }
        }
        assert_eq!(preset("social").unwrap().to_block_list("bn").name, "সোশ্যাল মিডিয়া");
    }
}
