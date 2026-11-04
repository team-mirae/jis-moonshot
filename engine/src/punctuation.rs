//! Symbol variants for the predictions (docs/moonshot.md §5.2): a reading that ends in a symbol is
//! also offered with that symbol's variants, as a Japanese IME offers 、 and ， for a comma.

/// The variants offered for a symbol at the end of a reading, in order; empty for anything else.
pub fn alternatives(c: char) -> &'static [&'static str] {
    match c {
        ',' => &["、", "，"],
        '、' => &[",", "，"],
        '，' => &[",", "、"],
        '.' => &["。", "…", "...", "．"],
        '。' => &[".", "…", "．"],
        '．' => &[".", "。"],
        '…' => &["...", "。"],
        '/' => &["・", "／"],
        '·' => &["・", "･"],
        '・' => &["·", "/"],
        '[' => &["「", "『", "【", "［", "〔"],
        ']' => &["」", "』", "】", "］", "〕"],
        '「' => &["『", "【", "["],
        '」' => &["』", "】", "]"],
        '『' => &["「", "【"],
        '』' => &["」", "】"],
        '(' => &["（"],
        ')' => &["）"],
        '{' => &["｛"],
        '}' => &["｝"],
        '<' => &["＜", "〈", "《", "«"],
        '>' => &["＞", "〉", "》", "»"],
        '!' => &["！"],
        '?' => &["？"],
        ':' => &["："],
        ';' => &["；"],
        '~' => &["〜", "～"],
        '-' => &["ー", "—", "–", "－"],
        '\'' => &["’", "‘"],
        '"' => &["“", "”"],
        // The ¥ key typed · until 2026-11-05 (docs/moonshot.md §9.2): the middle dots first.
        '¥' => &["·", "・", "￥", "\\"],
        '\\' => &["¥", "＼"],
        '_' => &["＿"],
        '^' => &["＾"],
        '|' => &["｜"],
        '@' => &["＠"],
        '#' => &["＃"],
        '$' => &["＄"],
        '%' => &["％"],
        '&' => &["＆"],
        '*' => &["＊", "×"],
        '+' => &["＋"],
        '=' => &["＝"],
        '`' => &["｀"],
        '！' => &["!"],
        '？' => &["?"],
        _ => &[],
    }
}

/// `text` with its last character swapped for each of its variants; empty when it has none.
pub fn variants(text: &str) -> Vec<String> {
    let Some(last) = text.chars().last() else { return Vec::new() };
    let head = &text[..text.len() - last.len_utf8()];
    alternatives(last).iter().map(|alt| format!("{head}{alt}")).collect()
}


/// Korean text with the symbol keys between two syllables read as the middle dot · (U+00B7,
/// 가운뎃점; docs/moonshot.md §5.6), as the kana reading reads Shift+`/` as ・: `?` (Shift+`/`, ・
/// on the JIS kana layout), and `/` too. Whether the dotted text goes above the text as typed: yes
/// unless a `/` is all it changes (남/여 is written so too).
/// None when no such symbol stands between two syllables.
pub fn middle_dotted(text: &str) -> Option<(String, bool)> {
    let syllable = |c: Option<&char>| c.is_some_and(|c| ('\u{AC00}'..='\u{D7A3}').contains(c));
    let chars: Vec<char> = text.chars().collect();
    let (mut out, mut changed, mut above) = (String::with_capacity(text.len()), false, false);
    for (i, &c) in chars.iter().enumerate() {
        if matches!(c, '?' | '/') && i > 0 && syllable(chars.get(i - 1)) && syllable(chars.get(i + 1)) {
            out.push('·');
            changed = true;
            above |= c != '/';
        } else {
            out.push(c);
        }
    }
    changed.then_some((out, above))
}

/// The marks the symbol keys stand for after a Korean word (docs/moonshot.md §5.6), then the
/// first syllables those keys type at a word's end (을 를 의 서 에서 로 으로 가 께서 왜 예 얘): the
/// Korean columns of `predict/punctuation.tsv`, in order.
pub const MARKS: &str = ".,?/;:-~";
const KOREAN_COLUMNS: &str = ".,?/;:-~을를의서에로으가께왜예얘";

/// The marks after a Japanese word (§5.6: a romaji typist's `.` `,` `/` give 。、？), then the kana
/// the same keys type on the JIS kana layout: the Japanese columns.
pub const KANA_MARKS: &str = "。、？・";
const JAPANESE_COLUMNS: &str = "。、？・るねめ";

/// The Japanese mark a symbol key stands for after kana (§5.6): its stock character as a romaji
/// typist's IME reads it (. 。, , 、, / and ? ？). None for the other keys.
pub fn kana_mark(stock: char) -> Option<char> {
    match stock {
        '.' | '>' => Some('。'),
        ',' | '<' => Some('、'),
        '/' | '?' => Some('？'),
        _ => None,
    }
}

/// Which language's counts to read.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Language {
    Korean,
    Japanese,
}

impl Language {
    fn columns(self) -> &'static str {
        match self {
            Language::Korean => KOREAN_COLUMNS,
            Language::Japanese => JAPANESE_COLUMNS,
        }
    }

    fn marks(self) -> usize {
        match self {
            Language::Korean => MARKS.chars().count(),
            Language::Japanese => KANA_MARKS.chars().count(),
        }
    }

    fn code(self) -> &'static str {
        match self {
            Language::Korean => "ko",
            Language::Japanese => "ja",
        }
    }

    /// A character the counts are kept after: a Hangul syllable; a kana (katakana as hiragana).
    fn letter(self, c: char) -> Option<char> {
        match self {
            Language::Korean => ('\u{AC00}'..='\u{D7A3}').contains(&c).then_some(c),
            Language::Japanese => match c {
                '\u{3041}'..='\u{3096}' | 'ー' => Some(c),
                '\u{30A1}'..='\u{30F6}' => char::from_u32(c as u32 - 0x60),
                _ => None,
            },
        }
    }
}

/// What follows each syllable or kana: `predict/punctuation.tsv`, from the open corpora
/// (`engine/tools/corpora/build_punctuation.py`), compiled in.
#[cfg(feature = "predict")]
const TABLE: &str = include_str!("../predict/punctuation.tsv");

/// Times seen and what came right after (one count per column), by language and context (a
/// character, or the two ending there; "" for all).
#[cfg(feature = "predict")]
type Counts = std::collections::HashMap<(&'static str, String), (f64, Vec<f64>)>;

#[cfg(feature = "predict")]
fn table() -> &'static Counts {
    static COUNTS: std::sync::OnceLock<Counts> = std::sync::OnceLock::new();
    COUNTS.get_or_init(|| {
        let mut counts = Counts::new();
        for line in TABLE.lines().filter(|l| !l.starts_with('#')) {
            let fields: Vec<&str> = line.split('\t').collect();
            let Some(language) = [Language::Korean, Language::Japanese].into_iter().find(|l| fields.first() == Some(&l.code())) else {
                continue;
            };
            if fields.len() != 3 + language.columns().chars().count() {
                continue;
            }
            let number = |s: &str| s.parse::<f64>().unwrap_or(0.0);
            let row = fields[3..].iter().map(|f| number(f)).collect();
            counts.insert((language.code(), fields[1].to_string()), (number(fields[2]), row));
        }
        counts
    })
}

/// How much less likely a mark is after a character the table doesn't have (seen too rarely).
#[cfg(feature = "predict")]
const RARE: f64 = 8.0;

/// How strongly a context's own counts count against the shorter one's, in times seen.
#[cfg(feature = "predict")]
const PRIOR: f64 = 20.0;

/// log2 of the chance that `next` (a mark, or a syllable or kana a symbol key types: the
/// language's columns) comes right after `text` (docs/moonshot.md §5.6): its last two characters'
/// counts, backed off to the last one's and to every character's. After a bare jamo a mark
/// hardly comes (one in 64 of the average), nor after a character seen too rarely to be in the
/// table (one in 8). None for a `next` the table doesn't have.
#[cfg(feature = "predict")]
pub fn follow_bits(language: Language, text: &str, next: char) -> Option<f64> {
    let column = language.columns().chars().position(|c| c == next)?;
    let counts = table();
    let get = |context: String| counts.get(&(language.code(), context));
    let estimate = |context: String, prior: f64| match get(context) {
        Some((seen, row)) => (row[column] + PRIOR * prior) / (seen + PRIOR),
        None => prior,
    };
    let mut p = match get(String::new()) {
        Some((seen, row)) => (row[column] + 1.0) / (seen + 1.0),
        None => 0.01,
    };
    let chars: Vec<char> = text.chars().collect();
    let last = *chars.last()?;
    if let Some(b) = language.letter(last) {
        // A character the corpora hardly have (틍) hardly ends a word: an eighth of the average.
        if get(b.to_string()).is_none() && column < language.marks() {
            p /= RARE;
        }
        p = estimate(b.to_string(), p);
        if let [.., a, _] = chars[..]
            && let Some(a) = language.letter(a)
        {
            p = estimate(format!("{a}{b}"), p);
        }
    } else if ('\u{3131}'..='\u{318E}').contains(&last) {
        p /= 64.0;
    }
    Some(p.max(1e-9).log2())
}

/// The average chance, in log2, of a syllable or kana a symbol key types after any character: for
/// a reading the table can't place (a final moved onto the key's vowel, 채긔 for 책 + ㅢ).
#[cfg(feature = "predict")]
pub fn any_next_bits(language: Language) -> f64 {
    let Some((seen, row)) = table().get(&(language.code(), String::new())) else { return -7.0 };
    let typed = &row[language.marks()..];
    (typed.iter().sum::<f64>() / typed.len() as f64 / seen.max(1.0)).max(1e-9).log2()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn variants_swap_the_last_symbol_only() {
        assert_eq!(variants("한,"), ["한、", "한，"]);
        assert_eq!(variants("a[b["), ["a[b「", "a[b『", "a[b【", "a[b［", "a[b〔"]);
        assert!(variants("한").is_empty());
        assert!(variants("").is_empty());
    }

    #[test]
    fn symbols_between_syllables_read_as_middle_dots() {
        assert_eq!(middle_dotted("가?나?다"), Some(("가·나·다".into(), true)));
        assert_eq!(middle_dotted("가¥나"), None);
        assert_eq!(middle_dotted("남/여"), Some(("남·여".into(), false)));
        // Not at a word's end, before a space or next to anything else.
        assert_eq!(middle_dotted("뭐해요?"), None);
        assert_eq!(middle_dotted("뭐해요? 응"), None);
        assert_eq!(middle_dotted("a/b"), None);
    }
}
