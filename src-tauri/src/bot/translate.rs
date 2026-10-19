//! Translation of chat messages the streamer may not understand. Language
//! detection runs locally (whatlang), so only foreign messages cost AI tokens.

use serde::Deserialize;
use whatlang::{Lang, Script};

use super::moderation::extract_json;
use super::prompt::language_name;

struct LangData {
    code: &'static str,
    whatlang: &'static str,
    /// Letters beyond a–z the language uses.
    letters: &'static str,
    /// Very common short words: a message containing them is almost surely
    /// in that language, even when it is too short for statistics.
    words: &'static [&'static str],
}

/// Languages the streamer can pick (ISO 639-1).
const LANGS: &[LangData] = &[
    LangData { code: "de", whatlang: "deu", letters: "äöüß", words: &["der", "die", "das", "und", "ist", "nicht", "ich", "du", "er", "wir", "ihr", "ein", "eine", "was", "wie", "war", "bin", "bist", "hast", "habe", "mit", "auf", "für", "zu", "ja", "nein", "aber", "auch", "noch", "mal", "schon", "jetzt", "hier", "geht", "gut", "echt", "doch", "dann", "wenn", "kann", "den", "dem", "mich", "dich", "sehr", "heute", "danke"] },
    LangData { code: "en", whatlang: "eng", letters: "", words: &["the", "and", "is", "are", "you", "it", "to", "of", "that", "this", "was", "what", "how", "not", "my", "your", "but", "with", "for", "have", "just", "me", "we", "they", "he", "she", "be", "do", "dont", "don't", "can", "im", "i'm", "its", "it's", "so", "why", "who", "when", "there", "very", "thanks", "love", "good"] },
    LangData { code: "es", whatlang: "spa", letters: "áéíñóúü¿¡", words: &["el", "la", "los", "las", "que", "qué", "y", "es", "en", "un", "una", "por", "para", "con", "como", "cómo", "pero", "muy", "esta", "está", "estoy", "hola", "tal", "todo", "bien", "gracias", "yo", "tu", "eres", "soy"] },
    LangData { code: "fr", whatlang: "fra", letters: "àâæçéèêëîïôœùûüÿ", words: &["le", "la", "les", "et", "est", "je", "tu", "il", "elle", "nous", "vous", "une", "des", "du", "que", "qui", "pas", "ne", "c'est", "mais", "pour", "avec", "très", "bien", "bonjour", "salut", "merci", "suis", "sur"] },
    LangData { code: "it", whatlang: "ita", letters: "àèéìíîòóù", words: &["il", "lo", "la", "gli", "le", "e", "è", "che", "di", "un", "una", "non", "per", "con", "sono", "ciao", "come", "va", "bene", "anche", "ma", "molto", "questo", "grazie", "sei"] },
    LangData { code: "pt", whatlang: "por", letters: "áâãàçéêíóôõú", words: &["o", "a", "os", "as", "e", "é", "que", "de", "do", "da", "não", "um", "uma", "para", "com", "eu", "você", "voce", "muito", "mas", "está", "isso", "tudo", "bem", "obrigado", "olá", "oi"] },
    LangData { code: "nl", whatlang: "nld", letters: "éëï", words: &["de", "het", "een", "en", "is", "ik", "je", "jij", "niet", "dat", "wat", "van", "met", "op", "voor", "maar", "ook", "zijn", "nog", "wel", "heel", "hoe", "dank", "goed", "ben", "weer", "jullie", "bedankt", "jongens"] },
    LangData { code: "pl", whatlang: "pol", letters: "ąćęłńóśźż", words: &["i", "w", "na", "nie", "to", "jest", "się", "że", "jak", "co", "ja", "ty", "z", "do", "ale", "tak", "już", "jestem", "bardzo", "dzięki", "cześć"] },
    LangData { code: "ru", whatlang: "rus", letters: "", words: &[] },
    LangData { code: "uk", whatlang: "ukr", letters: "", words: &[] },
    LangData { code: "tr", whatlang: "tur", letters: "çğıöşüâî", words: &["ve", "bir", "bu", "da", "de", "ne", "çok", "ben", "sen", "mi", "mı", "var", "yok", "için", "gibi", "ama", "değil", "evet", "selam", "nasılsın", "abi", "kanka", "lan", "teşekkürler"] },
    LangData { code: "sv", whatlang: "swe", letters: "åäöé", words: &["och", "är", "jag", "du", "det", "en", "ett", "att", "inte", "på", "med", "som", "har", "för", "men", "så", "vi", "hej", "tack"] },
    LangData { code: "no", whatlang: "nob", letters: "æøåé", words: &["og", "er", "jeg", "du", "det", "en", "et", "å", "ikke", "på", "med", "som", "har", "for", "men", "så", "vi", "hei", "takk"] },
    LangData { code: "da", whatlang: "dan", letters: "æøåé", words: &["og", "er", "jeg", "du", "det", "en", "et", "at", "ikke", "på", "med", "som", "har", "for", "men", "så", "vi", "hej", "tak"] },
    LangData { code: "fi", whatlang: "fin", letters: "äöå", words: &["ja", "on", "ei", "se", "että", "minä", "sinä", "tämä", "mutta", "kun", "niin", "hyvä", "kiitos", "moi"] },
    LangData { code: "cs", whatlang: "ces", letters: "áčďéěíňóřšťúůýž", words: &["a", "je", "to", "se", "na", "že", "v", "z", "jsem", "jsi", "ne", "ale", "jak", "co", "tak", "už", "ahoj", "díky"] },
    LangData { code: "hu", whatlang: "hun", letters: "áéíóöőúüű", words: &["a", "az", "és", "egy", "hogy", "nem", "is", "van", "ez", "de", "meg", "csak", "igen", "szia", "köszi"] },
    LangData { code: "ro", whatlang: "ron", letters: "ăâîșțşţ", words: &["și", "în", "este", "nu", "că", "un", "o", "pe", "la", "cu", "de", "ce", "mai", "sunt", "foarte", "salut", "mersi"] },
    LangData { code: "el", whatlang: "ell", letters: "", words: &[] },
    LangData { code: "ja", whatlang: "jpn", letters: "", words: &[] },
    LangData { code: "ko", whatlang: "kor", letters: "", words: &[] },
    LangData { code: "zh", whatlang: "cmn", letters: "", words: &[] },
    LangData { code: "ar", whatlang: "ara", letters: "", words: &[] },
    LangData { code: "hi", whatlang: "hin", letters: "", words: &[] },
    LangData { code: "th", whatlang: "tha", letters: "", words: &[] },
    LangData { code: "vi", whatlang: "vie", letters: "ăâđêôơưàáạảãằắặẳẵầấậẩẫèéẹẻẽềếệểễìíịỉĩòóọỏõồốộổỗờớợởỡùúụủũừứựửữỳýỵỷỹ", words: &["và", "là", "có", "không", "của", "tôi", "bạn", "này", "một", "được", "chào", "cảm", "ơn"] },
    LangData { code: "id", whatlang: "ind", letters: "", words: &["dan", "yang", "di", "ini", "itu", "tidak", "ada", "saya", "kamu", "aku", "dengan", "untuk", "apa", "juga", "sudah", "halo", "terima", "kasih"] },
];

/// Chat words that belong to no language in particular.
const CHAT_WORDS: &[&str] = &[
    "lol", "lmao", "lmfao", "rofl", "gg", "ggs", "gl", "hf", "glhf", "ez", "xd", "xdd", "omg", "wtf", "pog", "poggers", "kekw",
    "lul", "kappa", "bruh", "brb", "afk", "ok", "okay", "nice", "o7", "pls", "plz", "ty", "thx", "np", "hype", "stream",
    "chat", "streamer", "bro", "hi", "hey", "yo", "wow",
];

fn data(lang: Lang) -> Option<&'static LangData> {
    LANGS.iter().find(|d| Lang::from_code(d.whatlang) == Some(lang))
}

/// "hahaha", "jajaja", "kkkk", "wkwk": laughter in any language.
fn is_laughter(w: &str) -> bool {
    ["ha", "he", "hi", "ah", "ja", "je", "xd", "wk", "k", "rs", "lo"].iter().any(|unit| {
        w.len() >= unit.len() * 2 && w.len().is_multiple_of(unit.len()) && w.as_bytes().chunks(unit.len()).all(|c| c == unit.as_bytes())
    })
}

/// How sure the local detection is that a message needs translating.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Foreign {
    No,
    /// Possibly another language; checked when the AI sees the message anyway.
    Maybe,
    /// Clearly another language (foreign script, letters or common words).
    Yes,
}

pub struct Detector {
    understood: Vec<Lang>,
}

impl Detector {
    pub fn new(codes: &[String]) -> Self {
        let mut understood: Vec<Lang> = codes
            .iter()
            .filter_map(|c| LANGS.iter().find(|d| d.code == c.trim().to_lowercase()))
            .filter_map(|d| Lang::from_code(d.whatlang))
            .collect();
        // Japanese is also written in kanji, which whatlang reports as Mandarin script.
        if understood.contains(&Lang::Jpn) && !understood.contains(&Lang::Cmn) {
            understood.push(Lang::Cmn);
        }
        Self { understood }
    }

    /// `is_emote` tells whether a word is an emote name (they are not language).
    pub fn assess(&self, text: &str, is_emote: impl Fn(&str) -> bool) -> Foreign {
        if self.understood.is_empty() {
            return Foreign::No;
        }
        let words: Vec<String> = text
            .split_whitespace()
            .filter(|w| !(w.starts_with('@') || w.starts_with('!') || w.contains("://") || w.starts_with("www.") || is_emote(w)))
            .map(|w| w.trim_matches(|c: char| !c.is_alphanumeric() && c != '\'').to_lowercase())
            .filter(|w| w.chars().any(char::is_alphabetic) && w != "gif" && !CHAT_WORDS.contains(&w.as_str()) && !is_laughter(w))
            .collect();
        let clean = words.join(" ");
        let letters = clean.chars().filter(|c| c.is_alphabetic()).count();
        if letters < 2 {
            return Foreign::No;
        }
        let Some(script) = whatlang::detect_script(&clean) else {
            return Foreign::No;
        };
        // A script none of the streamer's languages uses: foreign even when short.
        if script != Script::Latin {
            let known = self.understood.iter().any(|l| script.langs().contains(l));
            return if known { Foreign::No } else { Foreign::Yes };
        }
        let mine: Vec<&LangData> = self.understood.iter().filter_map(|l| data(*l)).collect();
        if !self.understood.iter().any(|l| Script::Latin.langs().contains(l)) {
            // Only non-Latin languages understood: Latin words are foreign.
            return if letters >= 6 { Foreign::Yes } else { Foreign::No };
        }

        // Latin script. Common words decide short messages; statistics alone
        // are unreliable for chat ("lol that was so funny" looks Spanish).
        let own_hits = words.iter().filter(|w| mine.iter().any(|d| d.words.contains(&w.as_str()))).count();
        let foreign_hits = words
            .iter()
            .filter(|w| w.chars().count() >= 2)
            .filter(|w| {
                !mine.iter().any(|d| d.words.contains(&w.as_str()))
                    && LANGS.iter().any(|d| !mine.iter().any(|m| m.code == d.code) && d.words.contains(&w.as_str()))
            })
            .count();
        let foreign_letter = clean.chars().any(|c| {
            let latin_ext = ('\u{00C0}'..='\u{024F}').contains(&c) || ('\u{1E00}'..='\u{1EFF}').contains(&c);
            latin_ext && c.is_alphabetic() && !mine.iter().any(|d| d.letters.contains(c))
        });
        if own_hits > 0 && own_hits >= foreign_hits {
            return Foreign::No;
        }
        let detected = whatlang::detect(&clean);
        let other_lang = detected.as_ref().is_some_and(|i| !self.understood.contains(&i.lang()));
        if !other_lang && !foreign_letter && foreign_hits == 0 {
            return Foreign::No;
        }
        if (foreign_letter && (other_lang || foreign_hits > 0))
            || (foreign_hits >= 2 && own_hits == 0)
            || (detected.as_ref().is_some_and(|i| i.is_reliable()) && other_lang && letters >= 14)
        {
            return Foreign::Yes;
        }
        if (own_hits == 0 || foreign_hits > own_hits * 2) && letters >= 6 && (words.len() >= 2 || foreign_letter) {
            Foreign::Maybe
        } else {
            Foreign::No
        }
    }
}

/// Names of the understood languages for prompts ("German, English").
pub fn names(codes: &[String]) -> String {
    codes
        .iter()
        .map(|c| language_name(c))
        .filter(|n| !n.is_empty())
        .collect::<Vec<_>>()
        .join(", ")
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct Translation {
    /// Source language, named in the target language.
    pub lang: String,
    pub text: String,
    /// What a flagged message really means (slang, coded terms); may be empty.
    pub meaning: String,
}

#[derive(Deserialize)]
struct Row {
    i: usize,
    #[serde(default)]
    lang: String,
    #[serde(default)]
    translation: String,
    #[serde(default)]
    meaning: String,
}

/// Reads `translation` fields from a `{"results":[…]}` answer (used by both
/// the moderation and the translation prompt).
pub fn parse(text: &str, count: usize) -> Vec<(usize, Translation)> {
    let Some(v) = extract_json(text) else {
        return Vec::new();
    };
    let Some(rows) = v["results"].as_array() else {
        return Vec::new();
    };
    rows.iter()
        .filter_map(|r| serde_json::from_value::<Row>(r.clone()).ok())
        .filter(|r| r.i < count && !r.translation.trim().is_empty())
        .map(|r| {
            (
                r.i,
                Translation {
                    lang: r.lang.trim().chars().take(40).collect(),
                    text: r.translation.trim().chars().take(600).collect(),
                    meaning: r.meaning.trim().chars().take(300).collect(),
                },
            )
        })
        .collect()
}

/// Removes a leading "Name:" the model sometimes copies from the message list.
pub fn without_name(mut t: Translation, name: &str) -> Translation {
    if let Some(rest) = t.text.strip_prefix(name).and_then(|r| r.trim_start().strip_prefix(':')) {
        t.text = rest.trim().to_string();
    }
    t
}

/// Section appended to the moderation prompt when some messages are marked "(translate)".
pub fn moderation_section(understood: &str, target: &str) -> String {
    format!(
        "\n\nTranslation: the streamer only understands {understood}. Messages marked (translate) may be written in another language. \
For every marked message that is NOT written in one of those languages, ALWAYS include an entry (verdict \"ok\" when it is fine) with \
\"lang\":\"<its language, named in {target}>\" and \"translation\":\"<faithful {target} translation of the message text only (not the sender name); keep emotes and @names; never soften insults>\". \
If such a message is a violation or borderline, also add \"meaning\":\"<in {target}: what is really meant or implied beyond the literal words – slang, coded terms, slurs, insinuations; do not repeat the translation; max 25 words>\". \
Judge foreign-language messages by their meaning, exactly like the others; writing in another language is never a violation by itself. \
Marked messages that are already in an understood language get no translation entry."
    )
}

/// Prompt for messages that only need translating (no moderation).
pub fn translation_system(understood: &str, target: &str) -> String {
    format!(
        "You translate Twitch chat messages for a streamer who only understands {understood}.\n\
For each listed message that is NOT written in one of these languages, give its language (named in {target}) and a faithful {target} translation of the message text (without the sender name). \
Keep emotes, @names and links unchanged and keep the tone, including slang and insults – never soften or censor.\n\
Skip messages that are already in an understood language or are only emotes, names or laughter.\n\
Return ONLY JSON: {{\"results\":[{{\"i\":<index>,\"lang\":\"<language>\",\"translation\":\"<translation>\"}}]}}"
    )
}

/// Note for Twitch moderators (sent as the timeout/ban reason, which only
/// mods, the broadcaster and the affected user see).
pub fn mod_note(t: &Translation, de: bool) -> String {
    // Twitch reasons are limited to 500 characters; keep room for the meaning.
    let text: String = if t.text.chars().count() > 280 { format!("{}…", t.text.chars().take(279).collect::<String>()) } else { t.text.clone() };
    let mut s = if de {
        format!("Übersetzung ({}): „{text}“", t.lang)
    } else {
        format!("Translation ({}): \"{text}\"", t.lang)
    };
    if !t.meaning.is_empty() {
        s.push_str(if de { " · Gemeint: " } else { " · Meaning: " });
        s.push_str(&t.meaning);
    }
    s
}

#[cfg(test)]
mod tests {
    use super::*;

    fn d(codes: &[&str]) -> Detector {
        Detector::new(&codes.iter().map(|c| c.to_string()).collect::<Vec<_>>())
    }

    #[test]
    fn all_languages_map() {
        for d in LANGS {
            assert!(Lang::from_code(d.whatlang).is_some(), "{}", d.code);
            assert!(!language_name(d.code).is_empty(), "{}", d.code);
        }
    }

    #[test]
    fn detects_foreign_messages() {
        let de_en = d(&["de", "en"]);
        let no = |_: &str| false;
        for s in [
            "Bu oyunu çok seviyorum, harika bir yayın oluyor", "selam nasılsın kanka", "hola a todos, qué tal el directo",
            "je suis là depuis le début du stream", "ciao ragazzi come va", "jestem tu pierwszy raz, fajny stream",
            "siktir git lan orospu çocuğu", "ты тупой урод", "ありがとう", "que onda chat como estan",
        ] {
            assert_eq!(de_en.assess(s, no), Foreign::Yes, "{s}");
        }
        for s in [
            "Hallo zusammen, wie geht es euch heute Abend?", "bro das war so sick", "this boss fight is absolutely insane right now",
            "lol that was so funny", "na wie gehts", "omg omg omg nice play", "geh sterben du opfer", "I like it a lot",
            "hahaha xD", "jajajaja", "gg wp", "Kartoffelsalat", "Grüße aus Österreich", "ok cool", "W streamer",
            "@someone https://example.com/x 123",
        ] {
            assert_eq!(de_en.assess(s, no), Foreign::No, "{s}");
        }
        assert_ne!(de_en.assess("ik ben er weer bij jongens", no), Foreign::No);
        // Emote names are not language.
        assert_eq!(de_en.assess("KEKW PepeLaugh OMEGALUL", |w| !w.is_empty()), Foreign::No);
        // A Russian speaker understands Cyrillic; nothing set up = nothing translated.
        assert_eq!(d(&["ru"]).assess("Привет всем, как дела?", no), Foreign::No);
        assert_eq!(d(&[]).assess("Bu oyunu çok seviyorum", no), Foreign::No);
    }

    #[test]
    fn parses_translations_and_note() {
        let out = r#"{"results":[{"i":0,"verdict":"ok","lang":"Türkisch","translation":"Ich liebe dieses Spiel"},{"i":1,"verdict":"violation","category":"hate","lang":"Russisch","translation":"x","meaning":"Beleidigung"},{"i":7,"translation":"y"},{"i":2,"verdict":"ok"}]}"#;
        let t = parse(out, 3);
        assert_eq!(t.len(), 2);
        assert_eq!(t[0].1.text, "Ich liebe dieses Spiel");
        assert_eq!(t[1].1.meaning, "Beleidigung");
        assert_eq!(mod_note(&t[1].1, true), "Übersetzung (Russisch): „x“ · Gemeint: Beleidigung");
        let named = Translation { text: "troll2: Hau ab".into(), ..Default::default() };
        assert_eq!(without_name(named, "troll2").text, "Hau ab");
    }
}
