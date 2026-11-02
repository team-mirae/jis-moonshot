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
