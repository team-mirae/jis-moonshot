//! The 半角/全角 key (docs/moonshot.md §4.1): text to full-width or half-width, and which text in
//! the app it applies to when nothing is staged.

use crate::engine::{full_width, half_width};

/// Full-width (全角) or half-width (半角).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Width {
    Full,
    Half,
}

/// Half-width katakana U+FF66–U+FF9D and their full-width forms, in order.
const HALF_KANA: &str = "ｦｧｨｩｪｫｬｭｮｯｰｱｲｳｴｵｶｷｸｹｺｻｼｽｾｿﾀﾁﾂﾃﾄﾅﾆﾇﾈﾉﾊﾋﾌﾍﾎﾏﾐﾑﾒﾓﾔﾕﾖﾗﾘﾙﾚﾛﾜﾝ";
const FULL_KANA: &str = "ヲァィゥェォャュョッーアイウエオカキクケコサシスセソタチツテトナニヌネノハヒフヘホマミムメモヤユヨラリルレロワン";
const VOICED_MARK: char = 'ﾞ';
const SEMI_VOICED_MARK: char = 'ﾟ';

/// Katakana with a voiced form one code point up (ガ for カ).
const VOICEABLE: &str = "カキクケコサシスセソタチツテトハヒフヘホ";
/// Katakana with a semi-voiced form two code points up (パ for ハ).
const SEMI_VOICEABLE: &str = "ハヒフヘホ";

fn offset(c: char, by: u32) -> char {
    char::from_u32(c as u32 + by).unwrap_or(c)
}

/// A full-width katakana as half-width: one or two characters (ガ → ｶﾞ). `None` if it has none.
fn half_kana(c: char) -> Option<String> {
    let pick = |base: char| FULL_KANA.chars().position(|k| k == base).and_then(|i| HALF_KANA.chars().nth(i));
    if let Some(h) = pick(c) {
        return Some(h.to_string());
    }
    if c == 'ヴ' {
        return Some(format!("ｳ{VOICED_MARK}"));
    }
    let base = VOICEABLE.chars().find(|&b| offset(b, 1) == c);
    if let Some(h) = base.and_then(pick) {
        return Some(format!("{h}{VOICED_MARK}"));
    }
    let base = SEMI_VOICEABLE.chars().find(|&b| offset(b, 2) == c);
    base.and_then(pick).map(|h| format!("{h}{SEMI_VOICED_MARK}"))
}

/// `text` in `width`: every character with a form in that width takes it, the rest stay
/// (spec §5.3's pairs). Half-width also makes kana half-width katakana (かな → ｶﾅ, ガ → ｶﾞ), and
/// full-width puts half-width katakana back (ｶﾞ → ガ); Hangul has no other width.
pub fn convert(text: &str, width: Width) -> String {
    let mut out = String::with_capacity(text.len());
    let mut chars = text.chars().peekable();
    while let Some(c) = chars.next() {
        match width {
            Width::Full => {
                if let Some(i) = HALF_KANA.chars().position(|k| k == c) {
                    let full = FULL_KANA.chars().nth(i).unwrap_or(c);
                    let next = chars.peek().copied();
                    if next == Some(VOICED_MARK) && full == 'ウ' {
                        chars.next();
                        out.push('ヴ');
                    } else if next == Some(VOICED_MARK) && VOICEABLE.contains(full) {
                        chars.next();
                        out.push(offset(full, 1));
                    } else if next == Some(SEMI_VOICED_MARK) && SEMI_VOICEABLE.contains(full) {
                        chars.next();
                        out.push(offset(full, 2));
                    } else {
                        out.push(full);
                    }
                } else if c == VOICED_MARK {
                    out.push('゛');
                } else if c == SEMI_VOICED_MARK {
                    out.push('゜');
                } else {
                    out.push(full_width(c));
                }
            }
            Width::Half => {
                // Hiragana goes as katakana, as Japanese IMEs make it half-width.
                let kata = if ('\u{3041}'..='\u{3096}').contains(&c) { offset(c, 0x60) } else { c };
                match (kata, half_kana(kata)) {
                    (_, Some(half)) => out.push_str(&half),
                    ('゛', _) => out.push(VOICED_MARK),
                    ('゜', _) => out.push(SEMI_VOICED_MARK),
                    _ => out.push(half_width(c)),
                }
            }
        }
    }
    out
}

/// A script, for where a word in the app ends (docs/moonshot.md §4.1).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Script {
    Blank,
    Latin,
    Hangul,
    Hiragana,
    Katakana,
    Han,
    /// Digits, punctuation, symbols, ー: part of whichever script is next to them.
    Common,
}

fn script(c: char) -> Script {
    match c {
        _ if c.is_whitespace() => Script::Blank,
        'ー' | 'ｰ' | '々' | '゛' | '゜' | 'ﾞ' | 'ﾟ' => Script::Common,
        'a'..='z' | 'A'..='Z' | 'ａ'..='ｚ' | 'Ａ'..='Ｚ' | '\u{00C0}'..='\u{024F}' => Script::Latin,
        '\u{AC00}'..='\u{D7A3}' | '\u{1100}'..='\u{11FF}' | '\u{3130}'..='\u{318F}' | '\u{FFA0}'..='\u{FFDC}' => Script::Hangul,
        '\u{3041}'..='\u{309F}' => Script::Hiragana,
        '\u{30A0}'..='\u{30FF}' | '\u{31F0}'..='\u{31FF}' | '\u{FF66}'..='\u{FF9D}' => Script::Katakana,
        '\u{3400}'..='\u{4DBF}' | '\u{4E00}'..='\u{9FFF}' | '\u{F900}'..='\u{FAFF}' => Script::Han,
        _ => Script::Common,
    }
}

/// What the 半角/全角 key does to the app's text with nothing staged: replace `before` characters
/// before the caret (or the selection's start) and `after` characters after it with `text`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct WidthEdit {
    pub before: usize,
    pub after: usize,
    pub text: String,
}

/// The 半角/全角 key on the app's text (docs/moonshot.md §4.1), with nothing staged: `selected`
/// converted if there's a selection; else the word the caret is on (the run of characters around
/// it up to a blank or a change of script, holding the character right after the caret: at the
/// end of a word there's none); else nothing. `before` and `after` are the text either side of
/// the caret or selection (as much as the host has; a word longer than that converts in part).
/// `None` when nothing would change.
pub fn width_edit(before: &str, selected: &str, after: &str, width: Width) -> Option<WidthEdit> {
    if !selected.is_empty() {
        let text = convert(selected, width);
        return (text != selected).then_some(WidthEdit { before: 0, after: 0, text });
    }
    let mut left: Vec<char> = before.chars().rev().take_while(|c| !c.is_whitespace()).collect();
    left.reverse();
    let right: Vec<char> = after.chars().take_while(|c| !c.is_whitespace()).collect();
    if right.is_empty() {
        return None;
    }
    let caret = left.len();
    let run: Vec<char> = left.into_iter().chain(right).collect();
    // Split the run where the script changes; common characters go with the piece they're in.
    let mut start = 0;
    let mut current = Script::Common;
    let mut pieces = Vec::new();
    for (i, &c) in run.iter().enumerate() {
        let s = script(c);
        if s == Script::Common {
            continue;
        }
        if current != Script::Common && s != current {
            pieces.push(start..i);
            start = i;
        }
        current = s;
    }
    pieces.push(start..run.len());
    let piece = pieces.into_iter().find(|p| p.contains(&caret))?;
    let word: String = run[piece.clone()].iter().collect();
    let text = convert(&word, width);
    (text != word).then_some(WidthEdit { before: caret - piece.start, after: piece.end - caret, text })
}
