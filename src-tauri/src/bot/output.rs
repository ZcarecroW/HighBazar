//! Cleans model output into a safe single chat line.

use std::sync::LazyLock;

use regex::Regex;

static URL: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"(?i)(https?://|www\.)\S+|\b[a-z0-9-]+\.(com|net|org|io|gg|tv|ly|xyz|ru)/\S*").unwrap()
});

pub const TWITCH_MAX: usize = 500;

/// Returns `None` if the model chose to skip or produced nothing usable.
pub fn sanitize(raw: &str, bot_name: &str, prefix: &str, max_len: usize) -> Option<String> {
    let mut s = raw.trim().to_string();
    if s.is_empty() || s.contains("[SKIP]") {
        return None;
    }
    // Single line.
    s = s.split_whitespace().collect::<Vec<_>>().join(" ");
    // Strip wrapping quotes and markdown emphasis.
    for _ in 0..2 {
        let t = s.trim();
        let stripped = t
            .strip_prefix('"')
            .and_then(|x| x.strip_suffix('"'))
            .or_else(|| t.strip_prefix('“').and_then(|x| x.strip_suffix('”')))
            .or_else(|| t.strip_prefix('„').and_then(|x| x.strip_suffix('“')));
        if let Some(x) = stripped {
            s = x.trim().to_string();
        }
    }
    s = s.replace("**", "").replace("__", "");
    // Remove a leading "Name:" or the prefix if the model added it.
    for lead in [prefix.trim(), &format!("{bot_name}:"), &format!("[{bot_name}]"), &format!("{bot_name} :")] {
        if !lead.is_empty() && s.to_lowercase().starts_with(&lead.to_lowercase()) {
            s = s[lead.len()..].trim_start_matches([':', ' ', '-']).trim().to_string();
        }
    }
    // Never post links; never send chat commands.
    if URL.is_match(&s) {
        return None;
    }
    while s.starts_with('/') || s.starts_with('.') || s.starts_with('!') {
        s = s[1..].trim_start().to_string();
    }
    if s.trim().is_empty() {
        return None;
    }
    let budget = max_len.min(TWITCH_MAX.saturating_sub(prefix.chars().count() + 1)).max(20);
    Some(truncate_words(&s, budget))
}

pub fn truncate_words(s: &str, max_chars: usize) -> String {
    if s.chars().count() <= max_chars {
        return s.to_string();
    }
    let cut: String = s.chars().take(max_chars.saturating_sub(1)).collect();
    // Prefer ending after a complete sentence over a cut-off one.
    let sentence_end = cut
        .char_indices()
        .filter(|(i, ch)| matches!(ch, '.' | '!' | '?') && cut[i + ch.len_utf8()..].starts_with(' '))
        .map(|(i, ch)| i + ch.len_utf8())
        .next_back();
    if let Some(end) = sentence_end
        && cut[..end].chars().count() > max_chars / 2 {
            return cut[..end].to_string();
        }
    let trimmed = match cut.rfind(' ') {
        Some(i) if i > max_chars / 2 => cut[..i].to_string(),
        _ => cut,
    };
    format!("{}…", trimmed.trim_end_matches([',', ';', ':', ' ']))
}

pub fn with_prefix(text: &str, enabled: bool, prefix: &str) -> String {
    let p = prefix.trim();
    if enabled && !p.is_empty() { format!("{p} {text}") } else { text.to_string() }
}

/// Caps the number of known emotes in a message (unknown words are left
/// alone, since they may be normal words).
pub fn limit_emotes(text: &str, is_emote: impl Fn(&str) -> bool, max: usize) -> String {
    let mut count = 0;
    text.split(' ')
        .filter(|w| {
            if is_emote(w) {
                count += 1;
                count <= max
            } else {
                true
            }
        })
        .collect::<Vec<_>>()
        .join(" ")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cleans_output() {
        assert_eq!(sanitize("\"Hello chat!\"", "HighBazar", "[HighBazar]", 200).unwrap(), "Hello chat!");
        assert_eq!(sanitize("HighBazar: yo\nwhat's up", "HighBazar", "[HighBazar]", 200).unwrap(), "yo what's up");
        assert_eq!(sanitize("[HighBazar] hi", "HighBazar", "[HighBazar]", 200).unwrap(), "hi");
        assert_eq!(sanitize("/ban someone", "HighBazar", "", 200).unwrap(), "ban someone");
        assert!(sanitize("[SKIP]", "HighBazar", "", 200).is_none());
        assert!(sanitize("visit https://evil.example now", "HighBazar", "", 200).is_none());
    }

    #[test]
    fn truncates_on_word_boundary() {
        let t = truncate_words("one two three four five six", 15);
        assert!(t.ends_with('…') && t.chars().count() <= 15);
        let t = truncate_words("Great run so far! Who is ready for the next boss fight tonight", 30);
        assert_eq!(t, "Great run so far!");
    }

    #[test]
    fn caps_emotes() {
        let t = limit_emotes("KEKW KEKW nice KEKW", |w| w == "KEKW", 2);
        assert_eq!(t, "KEKW KEKW nice");
    }
}
