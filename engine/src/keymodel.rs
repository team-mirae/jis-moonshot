//! The three-language key model (`predict/ranker.bin`, format `JKR4`): order-4 key n-grams over
//! the 48 JIS keys with and without Shift, one model each for Korean 2-set typing (JIS Moonshot's
//! keys, docs/moonshot.md §9), English spelling and JIS kana typing (§3), Witten-Bell smoothed,
//! stored sparse with backoff weights. Built by `engine/tools/corpora/build_keymodel.py`, which
//! checks it against the dense table of every 4-gram (`engine/examples/keygrams.rs`).
//!
//! A word is scored from one space (the word before it), through its keys, to a space: each symbol
//! costs −log2 P(symbol | up to three before it), backing off to shorter histories with their
//! stored backoff cost when the n-gram wasn't seen. Two spaces never follow each other.

/// The keys, in the model's symbol order: `\` is the ¥ key, `_` the ろ key. With Shift: the same
/// order, from [`SHIFTED`].
const KEYS: &str = "abcdefghijklmnopqrstuvwxyz0123456789-^\\@[;:],./_";
/// The first symbol of a key with Shift.
pub const SHIFTED: u8 = 48;
/// The space between words, before and after each one.
pub const SPACE: u8 = 96;
const ALPHABET: u32 = 97;
const ORDER: usize = 4;

/// A language the model scores.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Class {
    Korean = 0,
    English = 1,
    Kana = 2,
}

#[derive(Debug, Default)]
struct Table {
    /// N-gram keys (see [`key`]), ascending, and each one's cost in 1/16 bits.
    ngrams: Vec<u32>,
    costs: Vec<u8>,
    /// History keys, ascending, and the cost of backing off from each, in 1/16 bits.
    histories: Vec<u32>,
    backoffs: Vec<u8>,
}

/// The parsed model, shared by every engine.
#[derive(Debug)]
pub struct KeyModel {
    tables: [Table; 3],
}

/// A sequence of 1..=4 symbols as one key: its symbols in base 97, offset by its length.
pub fn key(symbols: &[u8]) -> u32 {
    let idx = symbols.iter().fold(0u32, |i, &s| i * ALPHABET + s as u32);
    (symbols.len() as u32 - 1) * ALPHABET.pow(ORDER as u32) + idx
}

/// The model's symbol for a character of the English reading (spec §10.2: each key as it types on
/// stock JIS), Shift kept: `a` is the A key, `A` the A key with Shift, `!` the 1 key with Shift.
/// None for anything else.
pub fn symbol(c: char) -> Option<u8> {
    let (base, shift) = match c {
        'A'..='Z' => (c.to_ascii_lowercase(), true),
        '!' => ('1', true),
        '"' => ('2', true),
        '#' => ('3', true),
        '$' => ('4', true),
        '%' => ('5', true),
        '&' => ('6', true),
        '\'' => ('7', true),
        '(' => ('8', true),
        ')' => ('9', true),
        '=' => ('-', true),
        '~' => ('^', true),
        // The ¥ key (stock ¥, Shift |) and the ろ key (stock \, Shift _).
        '¥' => return Some(38),
        '|' => return Some(38 + SHIFTED),
        '\\' => return Some(47),
        '_' => return Some(47 + SHIFTED),
        '`' => ('@', true),
        '{' => ('[', true),
        '+' => (';', true),
        '*' => (':', true),
        '}' => (']', true),
        '<' => (',', true),
        '>' => ('.', true),
        '?' => ('/', true),
        other => (other, false),
    };
    let i = KEYS.chars().position(|k| k == base)? as u8;
    Some(if shift { i + SHIFTED } else { i })
}

struct Reader<'a> {
    data: &'a [u8],
    at: usize,
}

impl Reader<'_> {
    fn bytes(&mut self, n: usize) -> Option<&[u8]> {
        let out = self.data.get(self.at..self.at.checked_add(n)?)?;
        self.at += n;
        Some(out)
    }

    fn u32(&mut self) -> Option<u32> {
        Some(u32::from_le_bytes(self.bytes(4)?.try_into().ok()?))
    }

    fn keys(&mut self, n: usize) -> Option<Vec<u32>> {
        let raw = self.bytes(n.checked_mul(4)?)?;
        let keys: Vec<u32> = raw.chunks_exact(4).map(|b| u32::from_le_bytes([b[0], b[1], b[2], b[3]])).collect();
        // Strictly ascending, for the binary search.
        keys.windows(2).all(|w| w[0] < w[1]).then_some(keys)
    }
}

impl KeyModel {
    /// Whether `data` is a `JKR4` model (else it may be the older two-language format).
    pub fn is_model(data: &[u8]) -> bool {
        data.starts_with(b"JKR4")
    }

    /// Parses `predict/ranker.bin` in format `JKR4`. None if it's malformed.
    pub fn parse(data: &[u8]) -> Option<KeyModel> {
        let mut r = Reader { data, at: 0 };
        if r.bytes(4)? != b"JKR4" || r.u32()? != 4 || r.u32()? != ORDER as u32 || r.u32()? != ALPHABET || r.u32()? != 3 {
            return None;
        }
        let mut tables: [Table; 3] = Default::default();
        for table in &mut tables {
            let n = r.u32()? as usize;
            table.ngrams = r.keys(n)?;
            table.costs = r.bytes(n)?.to_vec();
            let m = r.u32()? as usize;
            table.histories = r.keys(m)?;
            table.backoffs = r.bytes(m)?.to_vec();
            // Every symbol alone must be there, so a lookup always ends.
            if (0..ALPHABET as u8).any(|s| table.ngrams.binary_search(&key(&[s])).is_err()) {
                return None;
            }
        }
        (r.at == data.len()).then_some(KeyModel { tables })
    }

    /// −log2 P(`next` | `history`) in 1/16 bits, backing off to shorter histories. `history` is at
    /// most three symbols.
    pub fn cost(&self, class: Class, history: &[u8], next: u8) -> u32 {
        let table = &self.tables[class as usize];
        let mut total = 0u32;
        let mut buffer = [0u8; ORDER];
        for start in 0..=history.len() {
            let h = &history[start..];
            buffer[..h.len()].copy_from_slice(h);
            buffer[h.len()] = next;
            if let Ok(i) = table.ngrams.binary_search(&key(&buffer[..=h.len()])) {
                return total + table.costs[i] as u32;
            }
            if !h.is_empty()
                && let Ok(i) = table.histories.binary_search(&key(h))
            {
                total += table.backoffs[i] as u32;
            }
        }
        // Unreachable with every unigram present (checked in `parse`): the most a byte holds.
        total + 255
    }

    /// log2 P(`symbols` as one word) under `class`: after a space, through the symbols, to a space.
    pub fn log_probability(&self, class: Class, symbols: &[u8]) -> f64 {
        let mut sequence = Vec::with_capacity(symbols.len() + 2);
        sequence.push(SPACE);
        sequence.extend_from_slice(symbols);
        sequence.push(SPACE);
        let total: u32 = (1..sequence.len()).map(|i| self.cost(class, &sequence[i.saturating_sub(ORDER - 1)..i], sequence[i])).sum();
        -(total as f64) / 16.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A model where every n-gram is unseen but the unigrams: each symbol costs `costs[class]`.
    fn uniform(costs: [u8; 3]) -> Vec<u8> {
        let mut out = b"JKR4".to_vec();
        for v in [4u32, 4, 97, 3] {
            out.extend(v.to_le_bytes());
        }
        for cost in costs {
            out.extend(97u32.to_le_bytes());
            for s in 0..97u8 {
                out.extend(key(&[s]).to_le_bytes());
            }
            out.extend(std::iter::repeat_n(cost, 97));
            out.extend(0u32.to_le_bytes());
        }
        out
    }

    #[test]
    fn symbols_are_the_keys_with_shift() {
        assert_eq!((symbol('a'), symbol('A')), (Some(0), Some(48)));
        assert_eq!((symbol('z'), symbol('Z')), (Some(25), Some(73)));
        assert_eq!((symbol('0'), symbol('1'), symbol('!')), (Some(26), Some(27), Some(75)));
        assert_eq!((symbol('¥'), symbol('|'), symbol('\\'), symbol('_')), (Some(38), Some(86), Some(47), Some(95)));
        assert_eq!((symbol('@'), symbol('`'), symbol('/'), symbol('?')), (Some(39), Some(87), Some(46), Some(94)));
        assert_eq!(symbol(' '), None);
        assert_eq!(symbol('한'), None);
    }

    #[test]
    fn parses_and_scores_with_backoff() {
        let m = KeyModel::parse(&uniform([16, 32, 48])).unwrap();
        // Two keys and the space after them, unigram costs only: 3 × 1, 2 and 3 bits.
        assert_eq!(m.log_probability(Class::Korean, &[0, 1]), -3.0);
        assert_eq!(m.log_probability(Class::English, &[0, 1]), -6.0);
        assert_eq!(m.log_probability(Class::Kana, &[0, 1]), -9.0);
    }

    #[test]
    fn refuses_what_isnt_a_model() {
        assert!(KeyModel::parse(b"JKRK").is_none());
        let mut old = uniform([16, 16, 16]);
        old[..4].copy_from_slice(b"JKR3");
        assert!(KeyModel::parse(&old).is_none());
        let mut bad = uniform([16, 16, 16]);
        bad.pop();
        assert!(KeyModel::parse(&bad).is_none());
        // A missing unigram.
        let mut good = uniform([16, 16, 16]);
        good.truncate(good.len() - 8);
        assert!(KeyModel::parse(&good).is_none());
    }
}
