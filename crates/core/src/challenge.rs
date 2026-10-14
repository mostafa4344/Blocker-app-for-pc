//! Random-text challenges: a deliberate pause before loosening a block.

use chrono::{DateTime, Duration, Utc};
use rand::Rng;
use serde::{Deserialize, Serialize};

// No look-alike characters (0/O, 1/l/I) so the text is fair to type.
const ALPHABET: &[u8] = b"abcdefghjkmnpqrstuvwxyzABCDEFGHJKLMNPQRSTUVWXYZ23456789";

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Challenge {
    pub id: String,
    pub text: String,
    /// What completing the challenge unlocks, e.g. `session:<id>` or `request:<id>`.
    pub purpose: String,
    pub expires_at: DateTime<Utc>,
}

pub fn generate(length: usize, purpose: &str, now: DateTime<Utc>) -> Challenge {
    let mut rng = rand::thread_rng();
    let mut text = String::with_capacity(length + length / 8);
    for i in 0..length {
        if i > 0 && i % 8 == 0 {
            text.push(' ');
        }
        text.push(ALPHABET[rng.gen_range(0..ALPHABET.len())] as char);
    }
    Challenge {
        id: crate::new_id(),
        text,
        purpose: purpose.to_string(),
        // Long enough to type carefully, short enough that it can't be prepared in advance.
        expires_at: now + Duration::minutes(15),
    }
}

pub fn verify(ch: &Challenge, purpose: &str, answer: &str, now: DateTime<Utc>) -> bool {
    now < ch.expires_at && ch.purpose == purpose && answer.trim() == ch.text
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn roundtrip() {
        let now = Utc::now();
        let c = generate(40, "session:1", now);
        assert_eq!(c.text.replace(' ', "").len(), 40);
        assert!(verify(&c, "session:1", &c.text, now));
        assert!(!verify(&c, "session:2", &c.text, now));
        assert!(!verify(&c, "session:1", "nope", now));
        assert!(!verify(&c, "session:1", &c.text, now + Duration::hours(1)));
    }
}
