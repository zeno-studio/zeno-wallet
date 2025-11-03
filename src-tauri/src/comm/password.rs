//! password_check.rs
//! Enhanced zxcvbn-lite – suitable for wallets / high-security Tauri apps
//! Dependencies: serde + once_cell + regex
//! Output: PasswordResult JSON (for direct frontend use)

use once_cell::sync::Lazy;
use regex::Regex;
use serde::Serialize;
use std::collections::HashSet;

/// Output structure (returned as JSON to frontend)
#[derive(Debug, Serialize)]
pub struct PasswordResult {
    pub score: u8,             // 0~4
    pub entropy: f64,          // bits
    pub crack_time: String,    // human-readable
    pub warnings: Vec<String>, // e.g. ["too_short", "no_symbol", "repeated_seq"]
}

/// Main password strength checker
#[tauri::command]
pub fn check_password(pw: &str) -> PasswordResult {
    let mut warnings = Vec::new();
    let len = pw.len() as f64;
    let mut entropy = 0.0;

    // 1️⃣ Basic rules
    if len < 9.0 {
        warnings.push("too_short".into());
    }
    if len > 32.0 {
        warnings.push("too_long".into());
    }

    let has_lower = pw.chars().any(char::is_lowercase);
    let has_upper = pw.chars().any(char::is_uppercase);
    let has_digit = pw.chars().any(|arg0: char| char::is_ascii_digit(&arg0));
    let has_symbol = pw.chars().any(|c| !c.is_ascii_alphanumeric());

    if !has_lower {
        warnings.push("no_lowercase".into());
    }
    if !has_upper {
        warnings.push("no_uppercase".into());
    }
    if !has_digit {
        warnings.push("no_digit".into());
    }
    if !has_symbol {
        warnings.push("no_symbol".into());
    }

    // 2️⃣ Entropy estimation
    let mut charset = 0.0f64;
    if has_lower {
        charset += 26.0f64;
    }
    if has_upper {
        charset += 26.0f64;
    }
    if has_digit {
        charset += 10.0f64;
    }
    if has_symbol {
        charset += 32.0f64;
    }
    if charset > 0.0f64 {
        entropy = charset.log2() * len;
    }

    // 3️⃣ Pattern-based penalties
    if len < 10.0 {
        entropy *= 0.85;
    }
    if !has_symbol {
        entropy *= 0.9;
    }
    if has_repeated_sequence(pw, 3) {
        warnings.push("repeated_seq".into());
        entropy *= 0.7;
    }
    if has_keyboard_sequence(pw) {
        warnings.push("keyboard_seq".into());
        entropy *= 0.6;
    }
    if contains_common_substring(pw) {
        warnings.push("common_substring".into());
        entropy *= 0.5;
    }
    if looks_like_pattern(pw) {
        warnings.push("pattern_like".into());
        entropy *= 0.5;
    }
    if contains_date_like(pw) {
        warnings.push("date_like".into());
        entropy *= 0.7;
    }
    if looks_like_word_combo(pw) {
        warnings.push("word_combo".into());
        entropy *= 0.6;
    }

    // 4️⃣ Final score
    let score = entropy_to_score(entropy);
    let crack_time = format_crack_time(entropy);

    PasswordResult {
        score,
        entropy: entropy.round(),
        crack_time,
        warnings,
    }
}

/// Convert entropy (bits) to 0–4 score
fn entropy_to_score(entropy: f64) -> u8 {
    match entropy {
        e if e < 28.0 => 0,
        e if e < 36.0 => 1,
        e if e < 60.0 => 2,
        e if e < 100.0 => 3,
        _ => 4,
    }
}

/// Format crack time assuming 10¹⁰ guesses/sec
fn format_crack_time(entropy: f64) -> String {
    let guesses = 2.0f64.powf(entropy);
    let seconds = guesses / 10_000_000_000.0;

    if seconds < 1.0 {
        "instant".into()
    } else if seconds < 60.0 {
        format!("{}s", seconds.round() as u32)
    } else if seconds < 3600.0 {
        format!("{}min", (seconds / 60.0).round() as u32)
    } else if seconds < 86400.0 {
        format!("{}h", (seconds / 3600.0).round() as u32)
    } else if seconds < 31_536_000.0 {
        format!("{}d", (seconds / 86400.0).round() as u32)
    } else {
        format!("{}y", (seconds / 31_536_000.0).round() as u32)
    }
}

/// Detect repeated sequences like "aaa"
fn has_repeated_sequence(pw: &str, min: usize) -> bool {
    let mut count = 1;
    let mut prev = None;
    for c in pw.chars() {
        if Some(c) == prev {
            count += 1;
            if count >= min {
                return true;
            }
        } else {
            count = 1;
            prev = Some(c);
        }
    }
    false
}

/// Detect common keyboard sequences (including reversed)
fn has_keyboard_sequence(pw: &str) -> bool {
    static SEQS: Lazy<HashSet<&str>> = Lazy::new(|| {
        [
            "qwerty", "asdfgh", "zxcvbn", "qazwsx", "123456", "7890", "1qaz", "2wsx",
        ]
        .into_iter()
        .collect()
    });
    let lower = pw.to_lowercase();
    SEQS.iter()
        .any(|&s| lower.contains(s) || lower.contains(&s.chars().rev().collect::<String>()))
}

/// Detect common substrings like "123", "admin", "password"
fn contains_common_substring(pw: &str) -> bool {
    static COMMON: Lazy<HashSet<&str>> = Lazy::new(|| {
        [
            "pass", "word", "123", "abc", "qwe", "admin", "root", "user", "test", "love", "2023",
            "2024", "2025", "letmein", "welcome", "dragon", "football", "iloveyou", "shadow",
            "master",
        ]
        .into_iter()
        .collect()
    });
    let lower = pw.to_lowercase();
    COMMON.iter().any(|&s| lower.contains(s))
}

/// Detect date-like patterns
fn contains_date_like(pw: &str) -> bool {
    let lower = pw.to_lowercase();
    lower.contains("19") || lower.contains("20") || lower.contains("01") || lower.contains("12")
}

/// Detect structured patterns: date / phone / email
fn looks_like_pattern(pw: &str) -> bool {
    static DATE_RE: Lazy<Regex> =
        Lazy::new(|| Regex::new(r"\b(19|20)\d{2}[-/]?\d{1,2}[-/]?\d{1,2}\b").unwrap());
    static PHONE_RE: Lazy<Regex> = Lazy::new(|| Regex::new(r"\b1\d{10}\b").unwrap());
    static EMAIL_RE: Lazy<Regex> =
        Lazy::new(|| Regex::new(r"^[\w\.-]+@[\w\.-]+\.\w{2,}$").unwrap());

    DATE_RE.is_match(pw) || PHONE_RE.is_match(pw) || EMAIL_RE.is_match(pw)
}

/// Detect passwords made from multiple common words
fn looks_like_word_combo(pw: &str) -> bool {
    static WORDS: Lazy<HashSet<&str>> = Lazy::new(|| {
        [
            "dog", "cat", "sun", "moon", "star", "baby", "money", "love", "god", "happy", "sad",
            "cool", "warm", "light", "dark", "rain", "snow",
        ]
        .into_iter()
        .collect()
    });
    let lower = pw.to_lowercase();
    WORDS.iter().filter(|w| lower.contains(**w)).count() >= 2
}

// ──────────────────────────────
// Unit tests
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_strength_basic() {
        let r = check_password("CorrectHorseBatteryStaple");
        assert!(r.score >= 3);
    }

    #[test]
    fn test_common() {
        let r = check_password("123456");
        assert_eq!(r.score, 0);
        assert!(r.warnings.contains(&"common_substring".into()));
    }

    #[test]
    fn test_repeated() {
        let r = check_password("aaaabbbb");
        assert!(r.warnings.contains(&"repeated_seq".into()));
    }

    #[test]
    fn test_word_combo() {
        let r = check_password("loveMoney2024");
        assert!(r.warnings.contains(&"word_combo".into()));
    }
}
