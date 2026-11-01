//! Japanese kana (docs/moonshot.md §2.1, §3): the JIS kana layout, the kana automaton (dakuten and
//! handakuten), and hiragana → katakana.
//!
//! Kana typing goes through a [`KanaScheme`]: [`KanaScheme::Jis`] is the only one built. A romaji
//! scheme would need its own composer (`ka` → か, `kk` → っk); [`KanaScheme::Romaji`] holds its place.

use crate::key::Key;

/// How keys turn into kana.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum KanaScheme {
    /// The JIS kana layout: one kana per key (docs/moonshot.md §3).
    #[default]
    Jis,
    /// Romaji (reserved, mode `A_ROMAJI`, id 12): not built. Every key types nothing.
    Romaji,
}

impl KanaScheme {
    /// The kana `key` types with or without Shift, or `None` for a key outside the scheme.
    pub fn kana(self, key: Key, shift: bool) -> Option<char> {
        match self {
            KanaScheme::Jis => jis(key, shift),
            KanaScheme::Romaji => None,
        }
    }
}

/// The JIS kana layout: (unshifted, shifted).
fn jis(key: Key, shift: bool) -> Option<char> {
    use Key::*;
    let (base, shifted) = match key {
        Digit1 => ('ぬ', 'ぬ'),
        Digit2 => ('ふ', 'ふ'),
        Digit3 => ('あ', 'ぁ'),
        Digit4 => ('う', 'ぅ'),
        Digit5 => ('え', 'ぇ'),
        Digit6 => ('お', 'ぉ'),
        Digit7 => ('や', 'ゃ'),
        Digit8 => ('ゆ', 'ゅ'),
        Digit9 => ('よ', 'ょ'),
        Digit0 => ('わ', 'を'),
        Minus => ('ほ', 'ほ'),
        Equal => ('へ', 'へ'),
        IntlYen => ('ー', 'ー'),
        KeyQ => ('た', 'た'),
        KeyW => ('て', 'て'),
        KeyE => ('い', 'ぃ'),
        KeyR => ('す', 'す'),
        KeyT => ('か', 'か'),
        KeyY => ('ん', 'ん'),
        KeyU => ('な', 'な'),
        KeyI => ('に', 'に'),
        KeyO => ('ら', 'ら'),
        KeyP => ('せ', 'せ'),
        BracketLeft => (DAKUTEN, DAKUTEN),
        BracketRight => (HANDAKUTEN, '「'),
        KeyA => ('ち', 'ち'),
        KeyS => ('と', 'と'),
        KeyD => ('し', 'し'),
        KeyF => ('は', 'は'),
        KeyG => ('き', 'き'),
        KeyH => ('く', 'く'),
        KeyJ => ('ま', 'ま'),
        KeyK => ('の', 'の'),
        KeyL => ('り', 'り'),
        Semicolon => ('れ', 'れ'),
        Quote => ('け', 'け'),
        Backslash => ('む', '」'),
        KeyZ => ('つ', 'っ'),
        KeyX => ('さ', 'さ'),
        KeyC => ('そ', 'そ'),
        KeyV => ('ひ', 'ひ'),
        KeyB => ('こ', 'こ'),
        KeyN => ('み', 'み'),
        KeyM => ('も', 'も'),
        Comma => ('ね', '、'),
        Period => ('る', '。'),
        Slash => ('め', '・'),
        IntlRo => ('ろ', 'ろ'),
        _ => return None,
    };
    Some(if shift { shifted } else { base })
}

/// The dakuten key's character (゛, U+309B), and the handakuten's (゜, U+309C).
pub const DAKUTEN: char = '\u{309B}';
pub const HANDAKUTEN: char = '\u{309C}';

/// Whether a key types a kana that isn't a letter in Korean: the symbol keys, which type
/// punctuation in Mode A (docs/moonshot.md §5, "kana-only keys").
pub fn is_kana_only(key: Key) -> bool {
    use Key::*;
    matches!(
        key,
        Minus | Equal | IntlYen | BracketLeft | BracketRight | Semicolon | Quote | Backslash | Comma | Period | Slash | IntlRo
    )
}

/// The voiced form of a kana (か → が, う → ゔ), if it has one.
fn voiced(c: char) -> Option<char> {
    match c {
        'う' => Some('ゔ'),
        'か' | 'き' | 'く' | 'け' | 'こ' | 'さ' | 'し' | 'す' | 'せ' | 'そ' | 'た' | 'ち' | 'つ' | 'て' | 'と' => {
            char::from_u32(c as u32 + 1)
        }
        'は' | 'ひ' | 'ふ' | 'へ' | 'ほ' => char::from_u32(c as u32 + 1),
        _ => None,
    }
}

/// The semi-voiced form of a kana of the は row (は → ぱ).
fn semi_voiced(c: char) -> Option<char> {
    match c {
        'は' | 'ひ' | 'ふ' | 'へ' | 'ほ' => char::from_u32(c as u32 + 2),
        _ => None,
    }
}

/// Composes kana as typed: ゛ and ゜ attach to the kana before them where they can, and stay as
/// they are where they can't (docs/moonshot.md §3.1). Anything that isn't kana passes through.
pub fn compose(typed: impl IntoIterator<Item = char>) -> String {
    let mut out = String::new();
    for c in typed {
        let mark = match c {
            DAKUTEN => Some(voiced as fn(char) -> Option<char>),
            HANDAKUTEN => Some(semi_voiced as fn(char) -> Option<char>),
            _ => None,
        };
        if let Some(mark) = mark
            && let Some(last) = out.chars().last()
            && let Some(marked) = mark(last)
        {
            out.pop();
            out.push(marked);
            continue;
        }
        out.push(c);
    }
    out
}

/// How many ゛ or ゜ stand alone in composed kana: each is a sign the keys weren't meant as kana.
pub fn loose_marks(composed: &str) -> usize {
    composed.chars().filter(|&c| c == DAKUTEN || c == HANDAKUTEN).count()
}

/// Whether composed kana starts the way no Japanese word does: small kana, ん, ー, a mark.
pub fn bad_start(composed: &str) -> bool {
    composed
        .chars()
        .next()
        .is_some_and(|c| "ぁぃぅぇぉゃゅょっんーをゎ".contains(c) || c == DAKUTEN || c == HANDAKUTEN)
}

/// How many small kana in composed kana sit where Japanese never puts them: ゃ ゅ ょ after anything
/// but an い-row kana (きゃ, not かゃ), and っ at the end or before ん, a vowel or another small kana.
pub fn misplaced_small(composed: &str) -> usize {
    let chars: Vec<char> = composed.chars().collect();
    let i_row = "きしちにひみりぎじぢびぴ";
    (0..chars.len())
        .filter(|&i| match chars[i] {
            'ゃ' | 'ゅ' | 'ょ' => i == 0 || !i_row.contains(chars[i - 1]),
            'っ' => chars.get(i + 1).is_none_or(|&n| "んあいうえおぁぃぅぇぉゃゅょっー".contains(n)),
            _ => false,
        })
        .count()
}

/// Hiragana in the katakana block (か → カ, ゔ → ヴ); everything else stays.
pub fn katakana(hiragana: &str) -> String {
    hiragana
        .chars()
        .map(|c| match c {
            'ぁ'..='ゖ' => char::from_u32(c as u32 + 0x60).unwrap_or(c),
            _ => c,
        })
        .collect()
}

/// Katakana in the hiragana block; everything else stays.
pub fn hiragana(katakana: &str) -> String {
    katakana
        .chars()
        .map(|c| match c {
            'ァ'..='ヶ' => char::from_u32(c as u32 - 0x60).unwrap_or(c),
            _ => c,
        })
        .collect()
}

/// Whether a kana can be doubled by a small っ before it: the か, さ, た and ぱ rows, voiced ones
/// included (きって, がっこう, ざっし).
fn takes_sokuon(c: char) -> bool {
    "かきくけこがぎぐげごさしすせそざじずぜぞたちつてとだぢづでどぱぴぷぺぽ".contains(c)
}

/// Readings of composed kana with つ as っ, for a typist who skipped Shift+Z: one variant per
/// つ before a kana that takes a small っ, and one with all of them, at most `limit`.
pub fn sokuon_variants(composed: &str, limit: usize) -> Vec<String> {
    let chars: Vec<char> = composed.chars().collect();
    let places: Vec<usize> = (0..chars.len().saturating_sub(1))
        .filter(|&i| i > 0 && chars[i] == 'つ' && takes_sokuon(chars[i + 1]))
        .collect();
    let mut out: Vec<String> = Vec::new();
    let with = |set: &[usize]| -> String {
        chars.iter().enumerate().map(|(i, &c)| if set.contains(&i) { 'っ' } else { c }).collect()
    };
    if places.len() > 1 {
        out.push(with(&places));
    }
    for &p in &places {
        if out.len() >= limit {
            break;
        }
        let v = with(&[p]);
        if !out.contains(&v) {
            out.push(v);
        }
    }
    out.truncate(limit);
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn marks_attach_where_they_can() {
        assert_eq!(compose(['か', DAKUTEN]), "が");
        assert_eq!(compose(['は', HANDAKUTEN]), "ぱ");
        assert_eq!(compose(['う', DAKUTEN]), "ゔ");
        assert_eq!(compose(['あ', DAKUTEN]), "あ゛");
        assert_eq!(compose([DAKUTEN]), "゛");
        assert_eq!(compose(['か', HANDAKUTEN]), "か゜");
    }

    #[test]
    fn katakana_round_trips() {
        assert_eq!(katakana("こーひー"), "コーヒー");
        assert_eq!(katakana("ゔ"), "ヴ");
        assert_eq!(hiragana("コーヒー"), "こーひー");
    }

    #[test]
    fn sokuon() {
        assert_eq!(sokuon_variants("きつて", 4), vec!["きって"]);
        assert!(sokuon_variants("つき", 4).is_empty());
        assert_eq!(sokuon_variants("がつこうつか", 4), vec!["がっこうっか", "がっこうつか", "がつこうっか"]);
    }
}
