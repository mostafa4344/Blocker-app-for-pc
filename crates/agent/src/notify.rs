//! Short heads-up messages for the signed-in user (e.g. "5 minutes left").
//!
//! The agent runs as a Windows service, which has no desktop of its own, so it
//! uses the OS-sanctioned `WTSSendMessageW`: a small message box on the active
//! user's screen that closes by itself and never blocks the agent.

use std::collections::{HashMap, HashSet};

/// Warn this many seconds before a daily allowance runs out.
pub const WARN_BEFORE_SECS: u64 = 5 * 60;

/// Remembers which limits were already warned about today, and only warns
/// while an allowance is actually being used (its remaining time is falling).
#[derive(Default)]
pub struct LimitWarnings {
    last_remaining: HashMap<String, u64>,
    warned: HashSet<(String, String)>,
}

impl LimitWarnings {
    /// Returns the limits to warn about now: `(limit_id, remaining_seconds)`.
    pub fn due(&mut self, day: &str, metered: &[(String, u64)]) -> Vec<(String, u64)> {
        self.warned.retain(|(d, _)| d == day);
        let mut out = Vec::new();
        for (id, remaining) in metered {
            let falling = self
                .last_remaining
                .get(id)
                .is_some_and(|last| remaining < last);
            self.last_remaining.insert(id.clone(), *remaining);
            if falling
                && *remaining > 0
                && *remaining <= WARN_BEFORE_SECS
                && self.warned.insert((day.to_string(), id.clone()))
            {
                out.push((id.clone(), *remaining));
            }
        }
        out
    }
}

/// The message text in the user's language.
pub fn limit_text(lang: &str, what: &str, remaining_secs: u64) -> (String, String) {
    let m = remaining_secs.div_ceil(60).max(1);
    let title = "HTL Content Guard".to_string();
    let body = match lang {
        "bn" => format!(
            "{what}: আজ আর {} মিনিট বাকি। এরপর আজ রাত ১২টা পর্যন্ত এটা ব্লক থাকবে।",
            bn_digits(m)
        ),
        "ar" => format!("{what}: بقيت {m} دقائق اليوم. بعدها سيُحظر حتى منتصف الليل."),
        "ur" => format!("{what}: آج {m} منٹ باقی ہیں۔ اس کے بعد آدھی رات تک بلاک رہے گا۔"),
        "id" => {
            format!("{what}: sisa {m} menit hari ini. Setelah itu diblokir sampai tengah malam.")
        }
        "tr" => format!("{what}: bugün {m} dakika kaldı. Sonra gece yarısına kadar engellenecek."),
        _ => format!(
            "{what}: {m} minute{} left today. After that it is blocked until midnight.",
            if m == 1 { "" } else { "s" }
        ),
    };
    (title, body)
}

/// Heads-up before a browser without a working extension is closed.
pub fn browser_text(lang: &str, browser: &str) -> (String, String) {
    let title = "HTL Content Guard".to_string();
    let body = match lang {
        "bn" => format!("{browser} ৯০ সেকেন্ড পর বন্ধ হবে, কারণ এতে ফিল্টার এক্সটেনশন চলছে না। কাজ সেভ করে নিন, অথবা এক্সটেনশন আছে এমন ব্রাউজার ব্যবহার করুন।"),
        "ar" => format!("سيُغلق {browser} بعد 90 ثانية لأن إضافة التصفية لا تعمل فيه. احفظ عملك أو استخدم متصفحًا فيه الإضافة."),
        "ur" => format!("{browser} 90 سیکنڈ بعد بند ہو جائے گا کیونکہ اس میں فلٹر ایکسٹینشن نہیں چل رہا۔ اپنا کام محفوظ کریں یا ایکسٹینشن والا براؤزر استعمال کریں۔"),
        "id" => format!("{browser} akan ditutup dalam 90 detik karena ekstensi filter tidak berjalan di sana. Simpan pekerjaan Anda, atau gunakan browser yang ada ekstensinya."),
        "tr" => format!("{browser} 90 saniye sonra kapanacak çünkü filtre uzantısı onda çalışmıyor. İşinizi kaydedin ya da uzantılı bir tarayıcı kullanın."),
        _ => format!("{browser} will close in 90 seconds because the filter extension isn't running in it. Save your work, or use a browser that has the extension."),
    };
    (title, body)
}

fn bn_digits(n: u64) -> String {
    n.to_string()
        .chars()
        .map(|c| char::from_u32('০' as u32 + c.to_digit(10).unwrap_or(0)).unwrap_or(c))
        .collect()
}

/// Shows a message box on the active user's desktop without waiting for it.
#[cfg(windows)]
pub fn show(title: &str, body: &str) {
    #[link(name = "wtsapi32")]
    extern "system" {
        fn WTSSendMessageW(
            server: *mut core::ffi::c_void,
            session: u32,
            title: *const u16,
            title_len: u32,
            message: *const u16,
            message_len: u32,
            style: u32,
            timeout: u32,
            response: *mut u32,
            wait: i32,
        ) -> i32;
    }
    #[link(name = "kernel32")]
    extern "system" {
        fn WTSGetActiveConsoleSessionId() -> u32;
    }
    const MB_ICONINFORMATION: u32 = 0x40;
    const MB_SETFOREGROUND: u32 = 0x1_0000;
    const MB_TOPMOST: u32 = 0x4_0000;
    let title: Vec<u16> = title.encode_utf16().collect();
    let body: Vec<u16> = body.encode_utf16().collect();
    std::thread::spawn(move || {
        // SAFETY: plain Win32 calls with valid buffers that outlive the call.
        unsafe {
            let session = WTSGetActiveConsoleSessionId();
            if session == u32::MAX {
                return;
            }
            let mut response = 0u32;
            WTSSendMessageW(
                std::ptr::null_mut(),
                session,
                title.as_ptr(),
                (title.len() * 2) as u32,
                body.as_ptr(),
                (body.len() * 2) as u32,
                MB_ICONINFORMATION | MB_SETFOREGROUND | MB_TOPMOST,
                60,
                &mut response,
                0,
            );
        }
    });
}

#[cfg(not(windows))]
pub fn show(title: &str, body: &str) {
    eprintln!("contentguard: [{title}] {body}");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn warns_once_while_in_use() {
        let mut w = LimitWarnings::default();
        let m = |s: u64| vec![("l".to_string(), s)];
        // Not being used (first sight, or not falling): no warning.
        assert!(w.due("d1", &m(200)).is_empty());
        assert!(w.due("d1", &m(200)).is_empty());
        // Being used and under 5 minutes: one warning only.
        assert_eq!(w.due("d1", &m(190)), vec![("l".to_string(), 190)]);
        assert!(w.due("d1", &m(180)).is_empty());
        // Still plenty left: no warning.
        assert!(w.due("d2", &m(3000)).is_empty());
        assert!(w.due("d2", &m(2990)).is_empty());
        // A new day warns again.
        assert!(w.due("d2", &m(310)).is_empty());
        assert_eq!(w.due("d2", &m(300)).len(), 1);
    }

    #[test]
    fn texts() {
        assert!(limit_text("en", "YouTube", 300)
            .1
            .contains("5 minutes left"));
        assert!(limit_text("en", "YouTube", 50).1.contains("1 minute left"));
        assert!(limit_text("bn", "YouTube", 300).1.contains("৫ মিনিট"));
        assert!(browser_text("en", "Edge")
            .1
            .starts_with("Edge will close in 90 seconds"));
    }
}
