//! Persistent data model: the user's configuration (`Config`) and the
//! agent's runtime state (`State`). Both are stored as JSON.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

pub const CONFIG_VERSION: u32 = 1;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(default)]
pub struct Config {
    pub version: u32,
    pub block_lists: Vec<BlockList>,
    pub schedules: Vec<Schedule>,
    pub limits: Vec<Limit>,
    pub protection: Protection,
    pub settings: Settings,
}

impl Default for Config {
    fn default() -> Self {
        Config {
            version: CONFIG_VERSION,
            block_lists: Vec::new(),
            schedules: Vec::new(),
            limits: Vec::new(),
            protection: Protection::default(),
            settings: Settings::default(),
        }
    }
}

impl Config {
    pub fn block_list(&self, id: &str) -> Option<&BlockList> {
        self.block_lists.iter().find(|b| b.id == id)
    }
}

/// A named set of things to block.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
#[serde(default)]
pub struct BlockList {
    pub id: String,
    pub name: String,
    /// Domain rules: `facebook.com`, `*.reddit.com`, `youtube.com/shorts`.
    pub sites: Vec<String>,
    /// Executable names: `steam.exe`, `Discord`, `slack`.
    pub apps: Vec<String>,
    /// Substrings matched against the full URL (lower-case).
    pub keywords: Vec<String>,
    /// When true, `sites` is an allowlist: everything else on the web is blocked.
    pub allowlist_mode: bool,
}

/// Weekly recurring window during which a block list is active.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
#[serde(default)]
pub struct Schedule {
    pub id: String,
    pub block_list_id: String,
    /// 0 = Monday … 6 = Sunday.
    pub days: Vec<u8>,
    /// Minutes since local midnight. `end <= start` means the window crosses midnight.
    pub start_minute: u16,
    pub end_minute: u16,
    pub enabled: bool,
    /// Locked schedules can only be made stricter.
    pub locked: bool,
}

/// Daily allowance: once used up, the block list is active for the rest of the day.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
#[serde(default)]
pub struct Limit {
    pub id: String,
    pub block_list_id: String,
    pub daily_minutes: u32,
    /// Optional cap on how many times per day the sites/apps may be opened.
    pub max_opens: Option<u32>,
    pub enabled: bool,
    pub locked: bool,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord, Default)]
#[serde(rename_all = "snake_case")]
pub enum ProtectionLevel {
    /// Uninstall and settings changes are unrestricted.
    #[default]
    Off,
    /// Requires typing a random text challenge, then a short wait.
    Mindful,
    /// Requests take effect only after a cooldown (24–72 h).
    Committed,
    /// Requests need approval from an accountability partner
    /// (auto-approved after `partner_timeout_days` so nobody is locked in forever).
    Partner,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(default)]
pub struct Protection {
    pub level: ProtectionLevel,
    pub cooldown_hours: u32,
    pub partner: Option<Partner>,
    pub partner_timeout_days: u32,
    /// Notify the partner when this device stops reporting.
    pub tamper_alerts: bool,
}

impl Default for Protection {
    fn default() -> Self {
        Protection {
            level: ProtectionLevel::Off,
            cooldown_hours: 24,
            partner: None,
            partner_timeout_days: 7,
            tamper_alerts: true,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
#[serde(default)]
pub struct Partner {
    pub name: String,
    pub email: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(default)]
pub struct Settings {
    /// `bn` or `en`.
    pub language: String,
    /// Shown on the block page.
    pub goal: String,
    /// Close browsers that do not have the ContentGuard extension connected
    /// while a block is active.
    pub require_browser_extension: bool,
    /// Base URL of the sync/partner backend; empty = offline only.
    pub backend_url: String,
    /// Practice mode: during the first days, locks are capped in length.
    pub practice_mode_until: Option<DateTime<Utc>>,
    /// Partial blocking done by the browser extension, e.g. `youtube_shorts`,
    /// `youtube_recommendations`, `facebook_feed`, `instagram_reels`.
    pub distraction_filters: Vec<String>,
    /// Completed onboarding.
    pub onboarded: bool,
}

impl Default for Settings {
    fn default() -> Self {
        Settings {
            language: "bn".into(),
            goal: String::new(),
            require_browser_extension: false,
            backend_url: String::new(),
            practice_mode_until: None,
            distraction_filters: Vec::new(),
            onboarded: false,
        }
    }
}

/// How a locked session may be ended early.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum LockKind {
    /// Not locked: can be stopped any time.
    None,
    /// Cannot be stopped before it ends.
    Timer,
    /// Can be stopped after typing a random text of `length` characters.
    RandomText { length: usize },
    /// Can be stopped after the computer has been restarted.
    Restart,
    /// Needs the accountability partner's approval.
    Partner,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Session {
    pub id: String,
    pub block_list_id: String,
    pub started_at: DateTime<Utc>,
    pub ends_at: DateTime<Utc>,
    pub lock: LockKind,
}

impl Session {
    pub fn is_active(&self, now: DateTime<Utc>) -> bool {
        self.started_at <= now && now < self.ends_at
    }
    pub fn is_locked(&self, now: DateTime<Utc>) -> bool {
        self.is_active(now) && self.lock != LockKind::None
    }
}

/// Usage for one limit on one local day.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
#[serde(default)]
pub struct Usage {
    /// `YYYY-MM-DD` in local time.
    pub day: String,
    pub limit_id: String,
    pub seconds: u64,
    pub opens: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
#[serde(default)]
pub struct State {
    pub sessions: Vec<Session>,
    pub usage: Vec<Usage>,
    pub requests: Vec<crate::protection::Request>,
    pub challenge: Option<crate::challenge::Challenge>,
    /// Highest trusted time seen; used to detect the clock being set back.
    pub trusted_now: Option<DateTime<Utc>>,
    /// Daily totals per block list for the dashboard (`day` -> list id -> seconds).
    pub stats: Vec<DailyStat>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
#[serde(default)]
pub struct DailyStat {
    pub day: String,
    pub key: String,
    pub seconds: u64,
    pub blocked_attempts: u32,
}
