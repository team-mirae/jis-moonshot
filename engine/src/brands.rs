//! Brand names (docs/moonshot.md §5.4): a reading that spells a brand, in any script, also offers
//! the brand in its default script (the country of origin's: Starbucks for 스타벅스, トヨタ for
//! 도요타, 카카오 for kakao). Only suggested, right below the reading that matched: never above
//! what was typed.
//!
//! The table is `predict/brands.tsv`, built from Wikidata (CC0) by `tools/brands/build_brands.py`.

use std::collections::HashMap;
use std::sync::OnceLock;

use crate::predict::{Kind, Prediction, Readings};

const TABLE: &str = include_str!("../predict/brands.tsv");

struct Brands {
    /// A normalized spelling → the brand's names in its default script, main name first.
    by_key: HashMap<&'static str, Vec<&'static str>>,
}

fn brands() -> &'static Brands {
    static BRANDS: OnceLock<Brands> = OnceLock::new();
    BRANDS.get_or_init(|| {
        let mut by_key = HashMap::new();
        for line in TABLE.lines().filter(|l| !l.starts_with('#')) {
            let mut fields = line.split('\t');
            let Some(names) = fields.next() else { continue };
            let names: Vec<&str> = names.split('|').filter(|n| !n.is_empty()).collect();
            if names.is_empty() {
                continue;
            }
            // The table lists the best-known brands first: a spelling stays theirs.
            for key in fields {
                by_key.entry(key).or_insert_with(|| names.clone());
            }
        }
        Brands { by_key }
    })
}

/// Korean or kana as the table spells it: no spaces or middle dots, kana as katakana.
fn norm_cjk(s: &str) -> String {
    s.chars()
        .filter(|c| !c.is_whitespace() && !matches!(c, '・' | '·' | '.'))
        .map(|c| if ('\u{3041}'..='\u{3096}').contains(&c) { char::from_u32(c as u32 + 0x60).unwrap_or(c) } else { c })
        .collect()
}

/// English as the table spells it: lower-case letters and digits only.
fn norm_en(s: &str) -> String {
    s.chars().filter(char::is_ascii_alphanumeric).map(|c| c.to_ascii_lowercase()).collect()
}

/// About how many syllables a name is said in, to pick the name closest to what was typed
/// (도요타 → トヨタ, not トヨタ自動車).
fn spoken(s: &str) -> f64 {
    s.chars()
        .map(|c| match c {
            '\u{AC00}'..='\u{D7A3}' => 1.0,
            'ァ' | 'ィ' | 'ゥ' | 'ェ' | 'ォ' | 'ャ' | 'ュ' | 'ョ' | 'ッ' | 'ー' | 'ぁ' | 'ぃ' | 'ぅ' | 'ぇ' | 'ぉ' | 'ゃ' | 'ゅ' | 'ょ' | 'っ' => 0.0,
            '\u{3040}'..='\u{30FF}' => 1.0,
            '\u{4E00}'..='\u{9FFF}' => 1.6,
            c if c.is_alphanumeric() => 0.45,
            _ => 0.0,
        })
        .sum()
}

/// The brand `key` spells, as the name in its default script closest in length to `typed`; the
/// main name unless an alias is clearly closer.
fn lookup(key: &str, typed: &str) -> Option<&'static str> {
    if key.chars().count() < 2 {
        return None;
    }
    let names = brands().by_key.get(key)?;
    let target = spoken(typed);
    names
        .iter()
        .enumerate()
        .map(|(i, n)| ((spoken(n) - target).abs() + if i == 0 { 0.0 } else { 1.0 }, *n))
        .min_by(|a, b| a.0.total_cmp(&b.0))
        .map(|(_, n)| n)
}

/// The brand a reading spells, if any: `korean` and `kana` are matched as Korean / kana, `english`
/// as letters (four or more, or capitals: LG, SKT); a reading left empty isn't matched.
fn brand_of(korean: &str, english: &str, kana: &str) -> Option<(&'static str, Kind)> {
    let k = norm_cjk(korean);
    if let Some(name) = lookup(&k, &k) {
        return Some((name, Kind::Korean));
    }
    let e = norm_en(english);
    let capitals = english.chars().filter(char::is_ascii_alphabetic).all(|c| c.is_ascii_uppercase());
    if (e.len() >= 4 || (e.len() >= 2 && capitals))
        && let Some(name) = lookup(&e, english)
    {
        return Some((name, Kind::English));
    }
    let j = norm_cjk(kana);
    lookup(&j, &j).map(|name| (name, Kind::Kana))
}

/// `predictions` with brand suggestions (docs/moonshot.md §5.4): the whole keys, or an English,
/// Korean or kana start before a particle (스타벅스에 → Starbucks에), spelling a brand. Each goes
/// right below the reading of the script it matched in (else at the end); never one that is
/// already listed, or that differs from what was typed only in case.
pub fn with_suggestions(mut predictions: Vec<Prediction>, readings: &Readings, ending: impl Fn(&str) -> bool) -> Vec<Prediction> {
    let mut found: Vec<(String, Kind)> = Vec::new();
    // Kana only where the predictor offered a kana reading: the kana of Korean typing is noise.
    let kana_shown = predictions.iter().any(|p| matches!(p.kind, Kind::Kana | Kind::Katakana));
    let kana = if kana_shown { readings.kana.as_str() } else { "" };
    if let Some((name, kind)) = brand_of(&readings.korean, &readings.english, kana) {
        found.push((name.to_string(), kind));
    }
    for s in &readings.splits {
        let korean_ending = ending(&s.korean);
        let kana_ending = crate::predict::KANA_PARTICLES.contains(&s.kana.as_str());
        if !korean_ending && !kana_ending {
            continue;
        }
        // One script before its particle: a Korean start before a Korean one, a kana start before
        // a kana one; an English start (where the predictor read it as English: a mixed reading
        // with it) before either.
        let english = predictions.iter().any(|p| p.kind == Kind::Mixed && p.text.starts_with(s.english.as_str()));
        let korean_start = if korean_ending { s.korean_start.as_str() } else { "" };
        let kana_start = if kana_ending && !korean_ending { s.kana_start.as_str() } else { "" };
        let english_start = if english { s.english.as_str() } else { "" };
        if let Some((name, kind)) = brand_of(korean_start, english_start, kana_start) {
            let tail = if korean_ending { &s.korean } else { &s.kana };
            found.push((format!("{name}{tail}"), kind));
        }
    }
    for (text, matched) in found {
        let typed = match matched {
            Kind::Korean => &readings.korean,
            Kind::English => &readings.english,
            _ => &readings.kana,
        };
        if predictions.iter().any(|p| p.text == text) || text.eq_ignore_ascii_case(typed) {
            continue;
        }
        let script = |k: Kind| match matched {
            Kind::Korean => matches!(k, Kind::Korean | Kind::Mixed),
            Kind::English => matches!(k, Kind::English | Kind::EnglishCapitalized | Kind::EnglishUpper | Kind::Mixed),
            _ => matches!(k, Kind::Kana | Kind::Katakana | Kind::Mixed),
        };
        let at = predictions.iter().position(|p| script(p.kind)).map_or(predictions.len(), |i| i + 1);
        let score = predictions.get(at.saturating_sub(1)).map_or(0.0, |p| p.score - 0.0005);
        predictions.insert(at, Prediction { text, kind: Kind::Brand, score });
    }
    predictions
}
