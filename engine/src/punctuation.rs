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
        '¥' => &["￥", "\\"],
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


/// The marks the symbol keys stand for after a Korean word (docs/moonshot.md §5.6), then the
/// first syllables those keys type at a word's end (을 를 의 서 에서 로 으로 가 께서 왜 예 얘): the
/// columns of `predict/punctuation.tsv`, in order.
pub const MARKS: &str = ".,?/;:-~";
const COLUMNS: &str = ".,?/;:-~을를의서에로으가께왜예얘";
const WIDTH: usize = 20;

/// What follows each syllable: `predict/punctuation.tsv`, from the open corpora
/// (`engine/tools/corpora/build_punctuation.py`), compiled in.
#[cfg(feature = "predict")]
const TABLE: &str = include_str!("../predict/punctuation.tsv");

/// Times seen and what came right after, by context (a syllable, or the two ending there; "" for
/// all).
#[cfg(feature = "predict")]
type Counts = std::collections::HashMap<String, (f64, [f64; WIDTH])>;

#[cfg(feature = "predict")]
fn table() -> &'static Counts {
    static COUNTS: std::sync::OnceLock<Counts> = std::sync::OnceLock::new();
    COUNTS.get_or_init(|| {
        let mut counts = Counts::new();
        for line in TABLE.lines().filter(|l| !l.starts_with('#')) {
            let fields: Vec<&str> = line.split('\t').collect();
            if fields.len() != 3 + WIDTH || fields[0] != "ko" {
                continue;
            }
            let number = |s: &str| s.parse::<f64>().unwrap_or(0.0);
            let mut row = [0.0; WIDTH];
            for (r, field) in row.iter_mut().zip(&fields[3..]) {
                *r = number(field);
            }
            counts.insert(fields[1].to_string(), (number(fields[2]), row));
        }
        counts
    })
}

/// How much less likely a mark is after a syllable the table doesn't have (seen too rarely).
#[cfg(feature = "predict")]
const RARE: f64 = 8.0;

/// How strongly a context's own counts count against the shorter one's, in times seen.
#[cfg(feature = "predict")]
const PRIOR: f64 = 20.0;

/// log2 of the chance that `next` (a mark in [`MARKS`] or a syllable a symbol key types) comes
/// right after `text` (docs/moonshot.md §5.6): its last two syllables' counts, backed off to the
/// last one's and to every syllable's. After a bare jamo a mark hardly comes (one in 64 of the
/// average), nor after a syllable seen too rarely to be in the table (one in 8). None for a `next` the table doesn't have.
#[cfg(feature = "predict")]
pub fn follow_bits(text: &str, next: char) -> Option<f64> {
    let column = COLUMNS.chars().position(|c| c == next)?;
    let counts = table();
    let estimate = |context: &str, prior: f64| match counts.get(context) {
        Some(&(seen, row)) => (row[column] + PRIOR * prior) / (seen + PRIOR),
        None => prior,
    };
    let (seen, row) = counts.get("").copied().unwrap_or((1.0, [0.0; WIDTH]));
    let mut p = (row[column] + 1.0) / (seen + 1.0);
    let chars: Vec<char> = text.chars().collect();
    let last = chars.last().copied()?;
    let syllable = |c: char| ('\u{AC00}'..='\u{D7A3}').contains(&c);
    if syllable(last) {
        // A syllable the corpora hardly have (틍) hardly ends a word: an eighth of the average.
        if !counts.contains_key(&last.to_string()) && column < MARKS.len() {
            p /= RARE;
        }
        p = estimate(&last.to_string(), p);
        if let [.., a, b] = chars[..]
            && syllable(a)
        {
            p = estimate(&format!("{a}{b}"), p);
        }
    } else if ('\u{3131}'..='\u{318E}').contains(&last) {
        p /= 64.0;
    }
    Some(p.max(1e-9).log2())
}

/// The average chance, in log2, of a syllable a symbol key types after any syllable: for a
/// reading the table can't place (a final moved onto the key's vowel, 채긔 for 책 + ㅢ).
#[cfg(feature = "predict")]
pub fn any_next_bits() -> f64 {
    let (seen, row) = table().get("").copied().unwrap_or((1.0, [0.0; WIDTH]));
    let syllables = &row[MARKS.len()..];
    (syllables.iter().sum::<f64>() / syllables.len() as f64 / seen.max(1.0)).max(1e-9).log2()
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
}
