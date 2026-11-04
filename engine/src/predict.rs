//! The predictive layer (spec §10, draft): ranks readings of the staged, uncommitted text.
//!
//! Optional twice over. Built only with the `predict` cargo feature (on by default; a lite build
//! uses `--no-default-features`), and active only once a host loads a [`Predictor`] into the engine.
//! Without one, the engine is exactly the deterministic lite engine: nothing here runs.
//!
//! The engine supplies the readings ([`Readings`]); this module only scores and orders them.
//! Scores are in bits per key: positive leans Korean. The core is a likelihood comparison of the
//! same keystrokes under two key-trigram models (English spelling vs JIS Korean typing), trained by
//! `engine/tools/corpora/build_ranker.py` into `engine/predict/ranker.bin`.

use std::collections::{HashMap, HashSet};


/// Which reading a prediction is.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[repr(u8)]
pub enum Kind {
    /// JIS Korean, as composed (한글).
    Korean = 0,
    /// The same keys on JIS QWERTY (gksrmf).
    English = 1,
    /// English with a capital first letter (Gksrmf).
    EnglishCapitalized = 2,
    /// English in capitals (GKSRMF).
    EnglishUpper = 3,
    /// What plain 2-set would compose: no implied ㅇ, digits as digits (ㅠㅠ, not 유유).
    TwoSet = 4,
    /// Korean with every final kept where a vowel would take it (말이, not 마리).
    KoreanKeepFinals = 5,
    // 6 was Hanja, retired with the hanja dictionary: the other ids stay.
    /// JIS Korean with the implied ㅇ left out: bare vowels (ㅠㅠ, not 유유).
    BareVowels = 7,
    /// An English word with a Korean particle or ending attached (report에, Anthropic에서).
    Mixed = 8,
    /// Korean with an implied ㄹ after a ㄴ final (전력 for 저녁 typed lazily: ㄴ + ㄹ sounds ㄹㄹ).
    ImpliedL = 9,
    /// The keys on the JIS kana layout, in hiragana (docs/moonshot.md §5).
    Kana = 10,
    /// The kana reading in katakana, when that is the known word (コーヒー).
    Katakana = 11,
    /// The kana reading converted to kanji (今日は for きょうは), docs/moonshot.md §5.1.
    Kanji = 12,
    // 13 was Aspirated (ㄶ for ㄴ + ㅋ / ㅌ / ㅊ, 않고 for 안코), retired on 2026-11-03: ㄶ has its own
    // key (Shift+@, docs/moonshot.md §9.2). The other ids stay.
    /// A brand in its default script, suggested below the reading that spells it (Starbucks for
    /// 스타벅스, トヨタ for 도요타), docs/moonshot.md §5.4.
    Brand = 14,
}

impl Kind {
    fn language(self) -> Language {
        match self {
            Kind::English | Kind::EnglishCapitalized | Kind::EnglishUpper | Kind::Brand => Language::English,
            Kind::Kana | Kind::Katakana | Kind::Kanji => Language::Japanese,
            _ => Language::Korean,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Language {
    Korean,
    English,
    Japanese,
}

/// The readings of the staged text, from the engine.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Readings {
    pub korean: String,
    pub english: String,
    pub two_set: String,
    pub keep_finals: String,
    /// Korean with one final kept before the vowel after it, one variant per place a final moved
    /// (산이 for 사니; 어린이 for 어리니).
    pub kept: Vec<String>,
    /// Korean with an implied ㄹ after a ㄴ final, one variant per place (전력 for 저녁).
    pub implied_l: Vec<String>,
    /// JIS Korean without the implied ㅇ.
    pub bare: String,
    /// How many syllables of the Korean reading have an implied ㅇ.
    pub implied: usize,
    /// The keys split into an English start and a Korean ending, at each place near the end.
    pub splits: Vec<Split>,
    /// The pieces between punctuation typed inside the word, each with its own readings (empty
    /// when there's no such punctuation).
    pub pieces: Vec<Piece>,
    /// A Korean start then an acronym (한전KPS, 에너지ICT), at the first place lowercase keys meet
    /// two capitals.
    pub head: Option<Head>,
    /// The keys on the JIS kana layout, composed, in hiragana (docs/moonshot.md §5).
    pub kana: String,
    /// Kana-only keys (symbol keys in Korean) inside the staged keys, not counting the last one.
    pub kana_only: usize,
    /// Keys right of 0 that typed Korean (docs/moonshot.md §9: ㅢ on `,`, 다 on `^`) inside the
    /// staged keys, not counting the last one: no English word has those characters inside it.
    pub korean_only: usize,
    /// The last key was one of them: it counts too when the Korean reading is a known word (의 typed
    /// d `,`), not when it may be a comma typed out of habit after English (hello,).
    pub korean_only_last: bool,
    /// The last key was one of them and typed a consonant first (ㄴ on `]`, 가 on `^`): no habit
    /// types those characters after English, so it counts whether or not the word is known.
    pub korean_only_last_consonant: bool,
    /// The keys read with the symbol keys at the end as punctuation (안녕하세요. for 안녕하세요을,
    /// docs/moonshot.md §5.6): one option per number of such keys taken (the last one, the last
    /// two). Empty when the last key isn't one, and inside an option.
    pub punctuated: Vec<Punctuated>,
}

/// The staged keys before the symbol keys at the end, and the punctuation those keys stand for
/// in Korean and in Japanese (docs/moonshot.md §5.6).
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Punctuated {
    pub readings: Readings,
    /// The marks after Korean (. , ? /: the keys' stock characters).
    pub punctuation: String,
    /// The marks after kana (。、？), empty when a key has none.
    pub kana_punctuation: String,
}

/// A staged word read as a Korean start, an acronym and maybe more Korean (한전KPS은).
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Head {
    /// The start, as Korean (한전).
    pub korean: String,
    /// The start's keys (gkswjs).
    pub keys: String,
    /// The capitals, as typed (KPS).
    pub acronym: String,
    /// The keys after them, as Korean (은; empty if none).
    pub rest: String,
}

/// A piece of a staged word between punctuation, and the punctuation after it.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Piece {
    pub readings: Readings,
    pub separator: String,
}

/// One way to read the staged keys as English followed by Korean.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Split {
    pub english: String,
    pub korean: String,
    /// The ending's keys read as JIS kana (docs/moonshot.md §5): Claudeの, Claudeは.
    pub kana: String,
    /// The start's keys read as Korean and as kana, for brands typed in either (스타벅스에,
    /// docs/moonshot.md §5.4).
    pub korean_start: String,
    pub kana_start: String,
}

/// What the predictor knows besides the keys.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Context {
    /// The language of the last committed word: close calls lean the same way.
    pub previous: Option<Language>,
    /// The staged word is a command name (after "/"): always English.
    pub command: bool,
    /// The user's tuning.
    pub weights: Weights,
    /// How the implied-ㅇ alternatives are ranked.
    pub ranker: Ranker,
    /// F8 (docs/moonshot.md §4): kanji conversions of the kana reading on top, whatever ranks
    /// first.
    pub force_kanji: bool,
    /// The word is one of a phrase staged with spaces (docs/moonshot.md §5.5).
    pub in_phrase: bool,
}

/// How the predictor ranks the implied-ㅇ alternatives: the bare vowels (ㅠㅠ against 유유) and the
/// kept final (말이 against 마리). Everything else ranks the same either way.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub enum Ranker {
    /// Hand-written rules: one step below the Korean reading, runs of one vowel lean bare (§10.3).
    #[default]
    Rules,
    /// Effort (experimental, spec §10.7): the keys a typist who meant the alternative would have
    /// had to press and didn't, by count and finger travel.
    Travel(Effort),
    /// Route preference (experimental, spec §10.8): how strongly a typist who meant a reading
    /// would prefer the route the keys took over the other one, each jamo or the optimized route.
    Route(Preference),
}

impl Ranker {
    /// The C ABI id (`JK_RANKER_*`): 0 rules, 1 travel with the default costs.
    pub fn from_id(id: u32) -> Option<Ranker> {
        match id {
            0 => Some(Ranker::Rules),
            1 => Some(Ranker::Travel(Effort::default())),
            2 => Some(Ranker::Route(Preference::default())),
            _ => None,
        }
    }
}

/// How strongly a typist prefers JIS Korean's optimized route (implied ㅇ, dedicated keys,
/// Shift+vowel, double press) over typing each jamo: the odds of taking it are
/// σ(press × presses saved + travel × travel saved − bias). `omission`: bits per key press a reading
/// needs that the keys left out (말이 typed as 마리).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Preference {
    pub press: f64,
    pub travel: f64,
    pub bias: f64,
    pub omission: f64,
}

impl Default for Preference {
    /// Near the best of the grid in `examples/sxs.rs` (2026-10-19, retrained on the corpora):
    /// presses saved don't move the preference, travel saved slightly lowers it (dedicated keys
    /// cost travel and are taken anyway), and a typist takes JIS Korean's route about 3 times in 4.
    fn default() -> Self {
        Preference { press: 0.0, travel: -0.5, bias: -1.0, omission: 3.0 }
    }
}

impl Preference {
    /// The chance a typist takes a route that saves `presses` and `travel` over the other one
    /// (negative: costs more). Equal routes are a coin toss when `bias` is 0.
    fn odds(&self, presses: f64, travel: f64) -> f64 {
        let x = self.press * presses + self.travel * travel - self.bias;
        (1.0 / (1.0 + (-x).exp())).clamp(0.02, 0.98)
    }
}

/// What skipped effort costs, in bits: per key press left out (Shift counted), and per key unit of
/// finger travel left out. A negative cost is a credit: the more travel a reading saves (F6 is far),
/// the likelier the typist relied on it. The defaults are the best of the grid in `examples/sxs.rs`
/// (2026-10-19, retrained on the corpora): both zero, so only the lexicon decides.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Effort {
    pub press: f64,
    pub travel: f64,
}

impl Default for Effort {
    fn default() -> Self {
        Effort { press: 0.0, travel: 0.0 }
    }
}

/// A scenario the user can tune (spec §10.5). The discriminant is the C ABI id (`JK_WEIGHT_*`).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum Weight {
    Korean = 0,
    English = 1,
    /// The implied ㅇ, against the bare vowel.
    Implied = 2,
    /// English + particle words (report에).
    Mixed = 3,
    TwoSet = 4,
    KeepFinals = 5,
    /// Retired with the hanja dictionary: no reading uses it. The slot keeps the ids of the
    /// weights after it (hosts store weights by id).
    Hanja = 6,
    /// An implied ㄹ after a ㄴ final (전력). 0 turns it off.
    ImpliedL = 7,
    /// The kana reading (docs/moonshot.md §5). 0 turns it off.
    Kana = 8,
}

impl Weight {
    pub const ALL: [Weight; 9] = [
        Weight::Korean,
        Weight::English,
        Weight::Implied,
        Weight::Mixed,
        Weight::TwoSet,
        Weight::KeepFinals,
        Weight::Hanja,
        Weight::ImpliedL,
        Weight::Kana,
    ];

    pub fn from_id(id: u32) -> Option<Weight> {
        Weight::ALL.get(id as usize).copied()
    }
}

/// User tuning: one multiplier per scenario on its learned probability. 1.0 is as learned; English
/// 0.8 makes English readings 0.8 times as likely, Korean 1.2 makes Korean 1.2 times as likely.
/// In score terms each weight adds log2(weight) bits.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Weights([f64; Weight::ALL.len()]);

impl Default for Weights {
    fn default() -> Self {
        Weights([1.0; Weight::ALL.len()])
    }
}

impl Weights {
    /// Lowest and highest weight accepted; values outside are clamped.
    pub const MIN: f64 = 0.1;
    pub const MAX: f64 = 10.0;

    pub fn get(&self, which: Weight) -> f64 {
        self.0[which as usize]
    }

    /// Sets a weight, clamped to [`Weights::MIN`]–[`Weights::MAX`]. 0 turns an alternative reading
    /// off entirely (not Korean or English, which always show).
    pub fn set(&mut self, which: Weight, value: f64) {
        let off = value == 0.0 && !matches!(which, Weight::Korean | Weight::English);
        self.0[which as usize] = if off {
            0.0
        } else if value.is_finite() {
            value.clamp(Self::MIN, Self::MAX)
        } else {
            1.0
        };
    }

    /// The weight in bits (log2), the unit scores are in.
    fn bits(&self, which: Weight) -> f64 {
        self.get(which).log2()
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct Prediction {
    pub text: String,
    pub kind: Kind,
    /// Higher is likelier. Only the order matters to hosts.
    pub score: f64,
}

/// Where a candidate reading came from: finer than [`Kind`] (the learned ranker's features, spec
/// §10.13).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum Origin {
    Korean,
    English,
    Capitalized,
    Upper,
    TwoSet,
    KeepFinals,
    /// One final kept before the vowel after it (산이 for 사니).
    Kept,
    ImpliedL,
    Bare,
    /// An English start and a Korean ending (report에).
    Mixed,
    /// A Korean start and an acronym (한전KPS).
    Head,
    /// The pieces between punctuation, each read on its own.
    Pieces,
}

/// A reading before ranking: its prediction with the rule-based (legacy) score, where it came
/// from, and for an English + particle reading, the split.
#[derive(Clone, Debug, PartialEq)]
pub struct Candidate {
    pub prediction: Prediction,
    pub origin: Origin,
    pub split: Option<Split>,
}

/// At most this many predictions are offered, not counting the kana reading, which takes one more
/// slot (docs/moonshot.md §5).
pub const MAX_PREDICTIONS: usize = 4;
/// At most this many predictions with the kana reading.
pub const MAX_WITH_KANA: usize = MAX_PREDICTIONS + 1;
/// Kanji conversions join the predictions only when a Japanese reading is among the top this many
/// (docs/moonshot.md §5.1), or after F8.
pub const KANJI_TOP_N: usize = 2;
/// At most this many kanji conversions, in slots of their own.
pub const MAX_KANJI: usize = 3;
/// At most this many predictions with the kana reading and the kanji conversions.
pub const MAX_WITH_KANJI: usize = MAX_WITH_KANA + MAX_KANJI;

const SIZE: usize = 27;
const START: usize = 26;
const END: usize = 26;
/// Shift on these can be Korean other than Shift+vowel: every consonant key, which types the
/// consonant and ㅏ (docs/moonshot.md §9.2, 2026-11-03; JIS Korean had Q W E R T O P). Shift on the
/// vowel keys starts a syllable on ㅇ, mid-word too.
pub(crate) const KOREAN_SHIFTS: &str = "QWERTASDFGZXCV";
pub(crate) const IEUNG_SHIFTS: &str = "YUIOPHJKLBNM";
/// Canonical 2-set keys for the dedicated number-row jamo (0 is ㅘ, 1–5 ㅃ ㅉ ㄸ ㄲ ㅆ, 6–9 ㅟ ㅝ ㅚ ㅢ).
const DIGIT_KEYS: [&str; 10] = ["hk", "q", "w", "e", "r", "t", "nl", "nj", "hl", "ml"];

const LEXICON: f64 = 2.0;
/// The count a hand-written lexicon word stands for: as common as a frequent chat word.
const HAND_COUNT: u32 = 1000;
/// An English reading that is a noun in its normalized form (WordNet): nouns weigh more than verbs
/// and other forms, which get no bonus. A little less than the hand-written lists.
const NOUN: f64 = 1.5;
const STICKY: f64 = 0.5;
/// A word count from which a Korean word is common: the business terms' count (§10.9).
const COMMON: u32 = 20;
const BARE_JAMO: f64 = 1.5;
/// Each key right of 0 that typed Korean inside the word (docs/moonshot.md §9), toward Korean.
const KOREAN_ONLY: f64 = 3.0;
/// The highest Mozc word cost that counts as a common Japanese word (lower is more common):
/// どようび 3353, とけい 4331 in; きのと 5766, はちの 6421 out.
const COMMON_KANJI_WORD: u16 = 4500;
/// Alternatives rank below the reading they're an alternative to.
const ALTERNATIVE: f64 = 1.0;
/// A known particle after an English start (report에): strong evidence of a mixed word.
const PARTICLE: f64 = 3.0;
/// How much more English the start must read than the whole keys for a split to count (bits per
/// key). Smaller differences are noise: `small` must not become smal이.
const SPLIT_EDGE: f64 = 0.5;
/// How English the start must read on its own (bits per key), so random keys (kanx) don't count.
const SPLIT_START: f64 = 1.0;
/// A word the user never means (training file `avoid`, spec §10.12): its readings drop this far.
/// A strong one is also never on top.
pub(crate) const AVOIDED: f64 = 4.0;

/// Particles and endings that attach to English words in mixed Korean text.
/// Japanese particles and endings that follow an English word (Claudeの, Claudeは, GitHubで),
/// read from the ending's keys as JIS kana (docs/moonshot.md §5.3).
pub(crate) const KANA_PARTICLES: &[&str] = &[
    "は", "が", "を", "に", "で", "と", "も", "へ", "の", "や", "か", "ね", "よ", "から", "まで", "より",
    "には", "では", "とは", "にも", "でも", "への", "との", "での", "からの", "までの", "って", "けど",
    "だ", "です", "でした", "という", "など", "さん", "くん", "ちゃん", "さま",
];

pub(crate) const PARTICLES: &[&str] = &[
    "은", "는", "이", "가", "을", "를", "의", "에", "에서", "에게", "에는", "에서는", "에서의", "에도",
    "과", "와", "도", "만", "로", "으로", "로는", "으로는", "로도", "으로도", "까지", "부터", "까지는", "까지의", "부터는", "부터의", "부로", "처럼", "보다",
    "이나", "나", "랑", "이랑", "한테", "께서", "이다", "입니다", "이고", "이며", "이라고", "라고", "이라는",
    "라는", "인", "적", "적인", "적으로", "들", "들이", "들은", "들을", "들의", "하는", "한", "하고", "했다",
    "하다", "해서", "하면", "된", "되는", "화", "화된", "용", "형", "급", "씨", "님",
    // Business text (DART filings): FDA로부터, Dynamics와의, Licensee로서, CRO이자, Innovation이란.
    "로부터", "으로부터", "로부터의", "로서", "으로서", "로써", "으로써", "로의", "으로의", "와의", "과의", "와는",
    "과는", "와도", "과도", "와를", "과를", "에서와", "에게서", "에서도", "만의", "이자", "이란", "란", "이라", "라",
    "등", "등의", "등을", "등이", "등과", "등은", "였다", "이었다", "였으며", "이었으며", "이지만", "임", "임을",
];

/// Lowercase ends of units and names that follow capitals (MWh, MMBtu, Ltd, Inc): after an acronym
/// they're English, not a Korean word.
pub(crate) const UNIT_ENDS: &[&str] = &["h", "tu", "td", "nc", "o", "s", "b", "bps", "wh", "m", "g", "t"];

/// Korean counters and words that follow a number (2개, 3년, '23년, 3년간): after digits, Korean.
pub(crate) const COUNTERS: &[&str] = &[
    "개", "년", "월", "일", "원", "명", "차", "건", "억", "조", "만", "배", "곳", "종", "위", "대", "호", "회", "층", "주", "시",
    "분", "초", "세", "개월", "개사", "개국", "년간", "년도", "년말", "년부터", "년까지", "년에", "년의", "년은", "척", "기", "톤",
    "억원", "조원", "만원", "천", "백", "개소", "차례", "번째", "등", "위권", "주년", "분기", "반기", "회차",
];

/// After a kept final's particle or ending, what may still follow it for the final to count as
/// kept (대해서는 is 대해서 + 는; 엔진 + 이어링 isn't): nothing, or a short ending.
const ENDING_TAILS: &[&str] = &[
    "", "다", "요", "니다", "습니다", "서", "도", "는", "고", "며", "면", "나", "만", "의", "에", "를", "가", "지", "죠", "야", "라",
    "란", "로", "었", "었다", "었습니다",
];

/// `word` or `word<TAB>count` lines (`#` lines are comments) into word counts; a word without a
/// count stands for a common one ([`HAND_COUNT`]).
fn word_counts(text: &str) -> HashMap<String, u32> {
    text.lines()
        .map(str::trim)
        .filter(|l| !l.is_empty() && !l.starts_with('#'))
        .map(|l| match l.split_once('\t') {
            Some((word, count)) => (word.trim().to_string(), count.trim().parse().unwrap_or(HAND_COUNT)),
            None => (l.to_string(), HAND_COUNT),
        })
        .collect()
}

/// First syllables of 하다 / 되다 forms that turn an English word into a verb (push했어).
const VERB_STARTS: &str = "하한할함합해했되된될됨됩돼됐";

/// Particles, copula forms and endings that begin with a vowel (on ㅇ), longest first where one
/// starts another doesn't matter (prefix match). From 한글 맞춤법 제14·15·19항's examples and common
/// usage: where these follow a known word, its final is kept in spelling.
const VOWEL_PARTICLES_AND_ENDINGS: &[&str] = &[
    // 조사 and the copula
    "이", "은", "을", "의", "에", "에서", "에게", "에는", "에도", "으로", "이나", "이랑", "이며", "이야", "아", "이다",
    "입니다", "이에요", "이었", "이고", "이지만", "이면", "이니까", "인", "일", "임", "으로서", "으로써", "으로부터",
    "이라", "이라도", "이란", "이라는", "이랑",
    // 어미 and -이/-음/-히
    "아", "어", "았", "었", "아서", "어서", "아도", "어도", "아야", "어야", "아요", "어요", "으니", "으면", "으며",
    "으러", "으려", "은", "을", "음", "으세", "으시", "읍시", "으므로", "을까", "을게", "을수록", "히",
];

/// Particles that follow only a consonant (after a vowel they're 는 를 와 로 랑 나): the whole
/// rest of a word after a ㅇ final, they show the final was kept (소비량은, not 소비랴은).
/// Not 이: too many words end in a vowel and 이 (아이, 오이), typed with ㅇ (d k d l).
const AFTER_CONSONANT_ONLY: &[&str] = &["은", "을", "과", "으로", "으로는", "으로서", "으로써", "으로부터", "으로도", "이랑", "이나"];

/// The least weight an implied ㅇ after a ㄴ or ㄹ final gets (bits) when what comes before is a
/// known word (기준입니다 typed r l w n s l q s l e k, 당신이, 산이, 말이): more than an
/// alternative's one-step penalty, so a noun and its particle win when the word lists don't say
/// otherwise. Elsewhere the corpus odds stand (드리겠습니다 stays, not 들이겠습니다).
const IMPLIED_AFTER_FLOOR: f64 = 1.5;

/// The kana reading's score (docs/moonshot.md §5), in bits: it starts this far below the top
/// reading, and the evidence moves it.
const KANA_BELOW: f64 = 3.0;
/// With the three-language key model: the kana reading's step from the best other reading is its
/// key model's bits against the likelier of Korean and English, times this, plus the bias, within
/// ± the cap (tuned on tests/moonshot.rs, tests/kanji.rs and the corpora).
const KANA_KEYS: f64 = 0.5;
const KANA_KEYS_BIAS: f64 = -1.0;
const KANA_KEYS_CAP: f64 = 6.0;
/// A whole known word of 4 kana or more, or a phrase entirely of known words and particles.
const KANA_WHOLE: f64 = 5.0;
/// Coverage by known words short of all of it: 90% is +1.8, none −3.6 (times this over 6).
const KANA_COVERAGE: f64 = 6.0;
/// Each kana-only key inside the keys (a symbol key in Korean), up to three.
const KANA_ONLY_KEY: f64 = 1.5;
/// Each small kana typed with Shift inside the keys, up to two.
const KANA_SMALL: f64 = 3.0;
/// Each bare jamo in the Korean reading: Korean is unlikely, up to six.
const BARE_JAMO_FOR_KANA: f64 = 0.5;
/// A loose ゛ or ゜, or a start no Japanese word has.
const KANA_INVALID: f64 = 6.0;
/// The Korean reading is a known Korean word.
const KANA_KOREAN_WORD: f64 = 3.0;
/// The previous word was Japanese.
const KANA_STICKY: f64 = 2.0;
/// The longest Japanese word the coverage looks for, in kana.
const KANA_WORD_MAX: usize = 10;

/// How far a word-initial Shift+vowel capital leans English when it doesn't settle it (bits).
const CAPITAL_LEAN: f64 = 2.0;

/// A word read piece by piece around its punctuation ranks above the whole-word readings: the
/// pieces were each read the likeliest way.
const PIECEWISE: f64 = 1.0;

/// The trained model and word lists.
#[derive(Clone)]
pub struct Predictor {
    /// Key n-gram order of the two-language model: 3 or 4.
    order: usize,
    english: Vec<u8>,
    korean: Vec<u8>,
    /// The three-language key model (`JKR3`): the kana reading's key score. When the file is only
    /// that model, it scores Korean against English too (`keys_for_margin`); when it follows the
    /// two-language tables, those keep scoring Korean against English (the rules are tuned on them).
    keys: Option<std::sync::Arc<crate::keymodel::KeyModel>>,
    keys_for_margin: bool,
    /// Lexicon words with their counts in the training sentences (hand-written ones: [`HAND_COUNT`]).
    english_words: HashMap<String, u32>,
    korean_words: HashMap<String, u32>,
    english_nouns: HashSet<String>,
    /// Bits for an implied ㅇ after a kept final, per (final, following syllable) (spec §10.3),
    /// learned from the corpora (`#!implied-after` lines in common-ko.txt).
    implied_after: HashMap<(char, char), f64>,
    /// Words the user never means, each `true` if strong (never on top). English ones lowercase.
    avoided_korean: HashMap<String, bool>,
    avoided_english: HashMap<String, bool>,
    /// Japanese words in kana, with counts (`predict/common-ja.txt`), and the particles and endings
    /// that join them (`#!particles` lines), for the kana reading (docs/moonshot.md §5).
    japanese_words: HashMap<String, u32>,
    japanese_particles: HashSet<String>,
    avoided_japanese: HashMap<String, bool>,
    /// The learned ranker's weights (spec §10.13); none: the rules rank (legacy).
    #[cfg(not(feature = "legacy-predict"))]
    learned: Option<std::sync::Arc<crate::learned::Model>>,
    /// Kana-kanji conversion (`predict/kanji.bin`, docs/moonshot.md §5.1); none: no kanji.
    kanji: Option<std::sync::Arc<crate::kanji::Kanji>>,
}

#[derive(Debug, PartialEq, Eq)]
pub struct BadModel;

impl Predictor {
    /// `model`: `predict/ranker.bin`. `english_words` / `korean_words`: `predict/common-*.txt`.
    /// `english_nouns`: `predict/english-nouns.txt` (may be empty: no noun bonus).
    pub fn parse(model: &[u8], english_words: &str, korean_words: &str, english_nouns: &str) -> Result<Predictor, BadModel> {
        // `JKR3`: the three-language key model (Korean, English, kana; order 4, sparse), alone or
        // after the two-language tables (`JKRK` then `JKR3`).
        let three_way = crate::keymodel::KeyModel::is_model(model);
        let parse_keys = |data: &[u8]| crate::keymodel::KeyModel::parse(data).map(std::sync::Arc::new).ok_or(BadModel);
        // Version 1: key trigrams; version 2: key 4-grams. One byte per entry, English then Korean.
        let order = match model.get(4..8) {
            _ if three_way => 4,
            Some([0, 1, 0, s]) if *s as usize == SIZE => 3,
            Some([0, 2, 0, s]) if *s as usize == SIZE => 4,
            _ => return Err(BadModel),
        };
        let table = SIZE.pow(order as u32);
        let base = 8 + 2 * table;
        let keys = if three_way {
            Some(parse_keys(model)?)
        } else if model.len() > base && crate::keymodel::KeyModel::is_model(&model[base..]) {
            Some(parse_keys(&model[base..])?)
        } else {
            None
        };
        if !three_way && (keys.is_none() && model.len() != base || &model[..4] != b"JKRK") {
            return Err(BadModel);
        }
        let words = word_counts;
        Ok(Predictor {
            order,
            english: if three_way { Vec::new() } else { model[8..8 + table].to_vec() },
            korean: if three_way { Vec::new() } else { model[8 + table..base].to_vec() },
            keys,
            keys_for_margin: three_way,
            english_words: words(english_words),
            korean_words: words(korean_words),
            english_nouns: words(english_nouns).into_keys().collect(),
            implied_after: korean_words
                .lines()
                .filter_map(|l| l.strip_prefix("#!implied-after "))
                .filter_map(|rest| {
                    let mut fields = rest.split_whitespace();
                    let final_ = fields.next()?.chars().next()?;
                    let syllable = fields.next()?.chars().next()?;
                    Some(((final_, syllable), fields.next()?.parse::<f64>().ok()?))
                })
                .map(|(key, odds)| (key, odds.clamp(-2.0 * LEXICON, 2.0 * LEXICON)))
                .collect(),
            avoided_korean: HashMap::new(),
            avoided_english: HashMap::new(),
            japanese_words: HashMap::new(),
            japanese_particles: HashSet::new(),
            avoided_japanese: HashMap::new(),
            #[cfg(not(feature = "legacy-predict"))]
            learned: None,
            kanji: None,
        })
    }

    /// This predictor ranking with the learned model `model` (`predict/learned.bin`, spec §10.13).
    /// The legacy build (`legacy-predict` feature) ignores it and keeps the rules: the same
    /// predictor back.
    pub fn with_learned(&self, model: &[u8]) -> Result<Predictor, BadModel> {
        #[cfg(not(feature = "legacy-predict"))]
        {
            let mut next = self.clone();
            next.learned = Some(std::sync::Arc::new(crate::learned::Model::parse(model)?));
            Ok(next)
        }
        #[cfg(feature = "legacy-predict")]
        {
            let _ = model;
            Ok(self.clone())
        }
    }

    /// This predictor with kana-kanji conversion (`predict/kanji.bin`, docs/moonshot.md §5.1).
    pub fn with_kanji(&self, data: &[u8]) -> Result<Predictor, BadModel> {
        let mut next = self.clone();
        next.kanji = Some(std::sync::Arc::new(crate::kanji::Kanji::parse(data)?));
        Ok(next)
    }

    /// Whether kana-kanji conversion is loaded.
    pub fn has_kanji(&self) -> bool {
        self.kanji.is_some()
    }

    /// Up to [`MAX_KANJI`] kanji conversions of `kana` (hiragana or katakana), cheapest first,
    /// without the kana itself. Empty without [`Predictor::with_kanji`].
    pub fn kanji_of(&self, kana: &str) -> Vec<String> {
        let Some(kanji) = &self.kanji else { return Vec::new() };
        let hiragana = crate::kana::hiragana(kana);
        let katakana = crate::kana::katakana(&hiragana);
        // One more than shown: the all-kana path is often among them.
        kanji
            .convert(&hiragana, MAX_KANJI + 2)
            .into_iter()
            .map(|(text, _)| text)
            .filter(|t| *t != hiragana && *t != katakana && *t != kana)
            .take(MAX_KANJI)
            .collect()
    }

    /// Adds the kanji conversions (docs/moonshot.md §5.1): right after the Japanese reading when
    /// one is among the top [`KANJI_TOP_N`]; on top, followed by the kana reading, after F8.
    fn with_kanji_conversions(&self, mut out: Vec<Prediction>, readings: &Readings, context: Context) -> Vec<Prediction> {
        let japanese = |p: &Prediction| matches!(p.kind, Kind::Kana | Kind::Katakana);
        if context.force_kanji {
            if readings.kana.is_empty() {
                return out;
            }
            // The kana reading, the predictor's (with its っ and katakana) or the keys' own.
            let kana = match out.iter().position(japanese) {
                Some(i) => out.remove(i),
                None => Prediction { text: readings.kana.clone(), kind: Kind::Kana, score: f64::NEG_INFINITY },
            };
            let top = out.first().map_or(0.0, |p| p.score).max(kana.score);
            let mut head: Vec<Prediction> = self
                .kanji_of(&kana.text)
                .into_iter()
                .enumerate()
                .map(|(i, text)| Prediction { text, kind: Kind::Kanji, score: top + (MAX_KANJI + 1 - i) as f64 })
                .collect();
            head.push(Prediction { score: top + 0.5, ..kana });
            out.retain(|p| !head.iter().any(|h| h.text == p.text));
            head.extend(out);
            head.truncate(MAX_WITH_KANJI);
            return head;
        }
        let Some(at) = out.iter().take(KANJI_TOP_N).position(japanese) else { return out };
        let kana = out[at].clone();
        let conversions: Vec<Prediction> = self
            .kanji_of(&kana.text)
            .into_iter()
            .filter(|t| !out.iter().any(|p| p.text == *t))
            .enumerate()
            .map(|(i, text)| Prediction { text, kind: Kind::Kanji, score: kana.score - 0.01 * (i + 1) as f64 })
            .collect();
        for (i, p) in conversions.into_iter().enumerate() {
            out.insert(at + 1 + i, p);
        }
        out.truncate(MAX_WITH_KANJI);
        out
    }

    /// This predictor with Japanese words for the kana reading (`predict/common-ja.txt`, or a
    /// training file's `vocabulary.japanese`): lines of `word` or `word<TAB>count` in hiragana or
    /// katakana, and `#!particles` lines of particles and endings. Merged into what's there.
    pub fn with_japanese_words(&self, text: &str) -> Predictor {
        let mut next = self.clone();
        for (word, count) in word_counts(text) {
            let entry = next.japanese_words.entry(word).or_insert(0);
            *entry = (*entry).max(count);
        }
        for line in text.lines().filter_map(|l| l.trim().strip_prefix("#!particles ")) {
            next.japanese_particles.extend(line.split_whitespace().map(|p| crate::kana::hiragana(p)));
        }
        next
    }

    /// This predictor with Japanese words the user never means (a training file's
    /// `avoid.japanese`): lines of `word` or `word<TAB>strong`, in kana.
    pub fn with_avoided_japanese(&self, text: &str) -> Predictor {
        let mut next = self.clone();
        for line in text.lines().map(str::trim).filter(|l| !l.is_empty() && !l.starts_with('#')) {
            let (word, strength) = line.split_once('\t').unwrap_or((line, ""));
            let word = crate::kana::hiragana(word.trim());
            let strong = next.avoided_japanese.get(&word).copied().unwrap_or(false) || strength.trim() == "strong";
            next.avoided_japanese.insert(word, strong);
        }
        next
    }

    /// Whether a kana word is in the Japanese lexicon, in hiragana or katakana.
    fn japanese_count(&self, word: &str) -> Option<u32> {
        self.japanese_words.get(word).copied().or_else(|| self.japanese_words.get(&crate::kana::katakana(word)).copied()).or_else(|| {
            // A common whole word in the kanji dictionary (どようび, 土曜日), from three kana: shorter
            // ones are nearly all some word there, and rare ones (きのと, 乙) aren't a typist's.
            // Counted like a hand-written word.
            let common = |k: &crate::kanji::Kanji| k.word_cost(&crate::kana::hiragana(word)).is_some_and(|cost| cost <= COMMON_KANJI_WORD);
            (word.chars().count() >= 3 && self.kanji.as_deref().is_some_and(common)).then_some(HAND_COUNT)
        })
    }

    /// The share of `kana` (hiragana) covered by known words and particles, by a longest-match
    /// segmentation (Japanese has no spaces, so a staged segment is often a whole phrase), and the
    /// kana written that way: a word the lexicon knows only in katakana is in katakana
    /// (あなたのぎたー → あなたのギター).
    fn kana_coverage(&self, kana: &str) -> (f64, String) {
        let chars: Vec<char> = kana.chars().collect();
        let n = chars.len();
        if n == 0 {
            return (0.0, String::new());
        }
        // best[i]: the most kana covered in chars[..i], and the word that ends there (its length).
        let mut best = vec![(0usize, 0usize); n + 1];
        for i in 0..n {
            // Kana i left uncovered (length 0 means one kana as typed).
            if best[i].0 > best[i + 1].0 {
                best[i + 1] = (best[i].0, 0);
            }
            for len in 1..=KANA_WORD_MAX.min(n - i) {
                let word: String = chars[i..i + len].iter().collect();
                let known = (len >= 2 && self.japanese_count(&word).is_some()) || self.japanese_particles.contains(&word);
                if known && best[i].0 + len > best[i + len].0 {
                    best[i + len] = (best[i].0 + len, len);
                }
            }
        }
        // Back through the segmentation: known words, and the kana between them as typed.
        let mut pieces = Vec::new();
        let mut end = n;
        while end > 0 {
            let len = best[end].1.max(1);
            let word: String = chars[end - len..end].iter().collect();
            let katakana = crate::kana::katakana(&word);
            let only_katakana = best[end].1 > 0
                && !self.japanese_words.contains_key(&word)
                && self.japanese_words.contains_key(&katakana);
            pieces.push(if only_katakana { katakana } else { word });
            end -= len;
        }
        pieces.reverse();
        (best[n].0 as f64 / n as f64, pieces.concat())
    }

    /// The kana reading, scored against `others` (the other readings, ranked): docs/moonshot.md §5.
    fn kana_prediction(&self, readings: &Readings, others: &[Prediction], context: Context) -> Option<Prediction> {
        let w = context.weights;
        // No Japanese lexicon loaded (`with_japanese_words`): no kana reading, as in JIS Korean.
        if readings.kana.is_empty() || w.get(Weight::Kana) == 0.0 || self.japanese_words.is_empty() {
            return None;
        }
        let mut texts = vec![readings.kana.clone()];
        texts.extend(crate::kana::sokuon_variants(&readings.kana, 3));
        let mut best: Option<(f64, String)> = None;
        for (i, text) in texts.iter().enumerate() {
            let n = text.chars().count();
            let mut e = 0.0;
            let exact = if n >= 2 { self.japanese_count(text) } else { None };
            let whole = match (exact, n) {
                (Some(_), 2) => 2.0,
                (Some(_), 3) => 3.5,
                (Some(_), _) => KANA_WHOLE,
                (None, _) => f64::NEG_INFINITY,
            };
            // A phrase made entirely of known words and particles counts as a known word.
            let (coverage, _) = self.kana_coverage(text);
            let covered = match n {
                _ if n < 3 => f64::NEG_INFINITY,
                _ if coverage >= 1.0 => KANA_WHOLE,
                _ => KANA_COVERAGE * (coverage - 0.6),
            };
            let words = whole.max(covered);
            if words.is_finite() {
                e += words;
            }
            // A variant the keys didn't type: a little below the reading itself.
            if i > 0 {
                e -= ALTERNATIVE;
            }
            // Without the kana key model, kana-only keys stand in for it (with it, they're in its
            // score below).
            if self.keys.is_none() {
                e += KANA_ONLY_KEY * readings.kana_only.min(3) as f64;
            }
            // The keys right of 0 that typed Korean (docs/moonshot.md §9) lifted the Korean reading
            // over English (`korean_only`), but they're kana-only keys as much: the same lift here,
            // so it doesn't push Korean past kana (どようび typed with ゛ on @). Only for a known
            // Japanese word: junk kana (ねにすひみち for 의약품) gets none of it.
            if exact.is_some() || coverage >= 1.0 {
                e += KOREAN_ONLY * readings.korean_only as f64;
            }
            // A small kana typed with Shift after the first key (っ ゃ ゅ ょ ぁ…): Korean never
            // shifts Z, and Shift+digits rarely sit inside a word.
            let small = readings.kana.chars().skip(1).filter(|&c| "ぁぃぅぇぉゃゅょっ".contains(c)).count();
            let misplaced = crate::kana::misplaced_small(text);
            e += KANA_SMALL * small.saturating_sub(misplaced).min(2) as f64;
            e -= KANA_INVALID * misplaced as f64;
            e += BARE_JAMO_FOR_KANA * readings.korean.chars().filter(|&c| is_jamo(c)).count().min(6) as f64;
            e -= KANA_INVALID * crate::kana::loose_marks(text) as f64;
            if crate::kana::bad_start(text) {
                e -= KANA_INVALID;
            }
            if self.korean_words.contains_key(readings.korean.as_str()) {
                e -= KANA_KOREAN_WORD;
            }
            match context.previous {
                Some(Language::Japanese) => e += KANA_STICKY,
                Some(_) => e -= STICKY,
                None => {}
            }
            if let Some(strong) = self.avoided_japanese.get(text) {
                e -= if *strong { 99.0 } else { AVOIDED };
            }
            if best.as_ref().is_none_or(|(b, _)| e > *b) {
                best = Some((e, text.clone()));
            }
        }
        let (e, text) = best?;
        let reference = others.first().map_or(0.0, |p| p.score);
        // How much better the kana key model explains the keys than the likelier of the Korean and
        // English ones (spec §10.3), bounded; without the model, a fixed step below.
        // Not with digits typed as digits (2026, ㅅ7): the Korean and English models never saw the
        // number row as digits, so the kana model would win them by default.
        let digits = readings.korean.chars().any(|c| c.is_ascii_digit());
        let keys = match self.kana_bits(&readings.english).filter(|_| !digits) {
            Some(bits) => (KANA_KEYS * bits + KANA_KEYS_BIAS).clamp(-KANA_KEYS_CAP, KANA_KEYS_CAP),
            None => -KANA_BELOW,
        };
        let score = reference + keys + e + w.bits(Weight::Kana);
        // Katakana where that's the word the lexicon knows: the whole (コーヒー), or words inside a
        // phrase (あなたのギターになりたい).
        let katakana = crate::kana::katakana(&text);
        let (text, kind) = if !self.japanese_words.contains_key(&text) && self.japanese_words.contains_key(&katakana) {
            (katakana, Kind::Katakana)
        } else {
            (self.kana_coverage(&text).1, Kind::Kana)
        };
        Some(Prediction { text, kind, score })
    }

    /// Merges the kana reading into ranked predictions: by score, in a slot of its own, never
    /// replacing another reading, and never a copy of one.
    fn with_kana(&self, mut out: Vec<Prediction>, readings: &Readings, context: Context) -> Vec<Prediction> {
        let Some(kana) = self.kana_prediction(readings, &out, context) else { return out };
        if !kana.score.is_finite() || out.iter().any(|p| p.text == kana.text) {
            return out;
        }
        let at = out.iter().position(|p| p.score < kana.score).unwrap_or(out.len());
        out.insert(at, kana);
        out.truncate(MAX_WITH_KANA);
        out
    }

    /// A Korean word's count in the lexicon (hand-written words count [`HAND_COUNT`]).
    #[cfg_attr(feature = "legacy-predict", allow(dead_code))]
    pub(crate) fn korean_count(&self, word: &str) -> Option<u32> {
        self.korean_words.get(word).copied()
    }

    /// An English word's count in the lexicon, by its lowercase form.
    #[cfg_attr(feature = "legacy-predict", allow(dead_code))]
    pub(crate) fn english_count(&self, lowercase: &str) -> Option<u32> {
        self.english_words.get(lowercase).copied()
    }

    /// Whether a lowercase English word is a noun (WordNet).
    #[cfg_attr(feature = "legacy-predict", allow(dead_code))]
    pub(crate) fn is_noun(&self, lowercase: &str) -> bool {
        self.english_nouns.contains(lowercase)
    }

    /// Whether the learned ranker ranks (a model is loaded, and this isn't the legacy build).
    pub fn is_learned(&self) -> bool {
        #[cfg(not(feature = "legacy-predict"))]
        return self.learned.is_some();
        #[cfg(feature = "legacy-predict")]
        false
    }

    /// This predictor ranking by the rules only (the learned model, if any, dropped).
    pub fn without_learned(&self) -> Predictor {
        #[allow(unused_mut)]
        let mut next = self.clone();
        #[cfg(not(feature = "legacy-predict"))]
        {
            next.learned = None;
        }
        next
    }

    /// This predictor with the user's own words added (spec §10.12, the training contract): lines of
    /// `word` or `word<TAB>count` as in common-*.txt. A word already known keeps the larger count;
    /// a word without a count counts as a common one ([`HAND_COUNT`]).
    pub fn with_user_words(&self, korean: &str, english: &str) -> Predictor {
        let mut next = self.clone();
        let merge = |into: &mut HashMap<String, u32>, text: &str, lowercase: bool| {
            for (word, count) in word_counts(text) {
                let entry = into.entry(if lowercase { word.to_lowercase() } else { word }).or_insert(0);
                *entry = (*entry).max(count);
            }
        };
        merge(&mut next.korean_words, korean, false);
        merge(&mut next.english_words, english, true);
        next
    }

    /// This predictor with words the user never means (spec §10.12, the training contract's
    /// `avoid`): lines of `word` or `word<TAB>strong`. Wherever a reading is an avoided word, alone
    /// or with a particle after it (god, god을, 마리가), it drops [`AVOIDED`] bits; a strong one is
    /// also never on top. English matches ignore case, but keys starting with a capital are taken as
    /// asked for (God typed with Shift+G stays). An avoided word wins over the same word in the vocabulary.
    pub fn with_avoided(&self, korean: &str, english: &str) -> Predictor {
        let mut next = self.clone();
        let merge = |into: &mut HashMap<String, bool>, text: &str, lowercase: bool| {
            for line in text.lines().map(str::trim).filter(|l| !l.is_empty() && !l.starts_with('#')) {
                let (word, strength) = line.split_once('\t').unwrap_or((line, ""));
                let word = if lowercase { word.trim().to_lowercase() } else { word.trim().to_string() };
                let strong = into.get(&word).copied().unwrap_or(false) || strength.trim() == "strong";
                into.insert(word, strong);
            }
        };
        merge(&mut next.avoided_korean, korean, false);
        merge(&mut next.avoided_english, english, true);
        next
    }

    /// Whether a prediction is a word the user never means, and if so whether strongly. `keys`: the
    /// English reading, the keys as typed.
    pub(crate) fn avoided(&self, p: &Prediction, keys: &str) -> Option<bool> {
        // A word, alone or followed by a particle.
        let find = |list: &HashMap<String, bool>, text: &str| {
            list.get(text).copied().or_else(|| {
                PARTICLES.iter().find_map(|particle| text.strip_suffix(particle).and_then(|word| list.get(word).copied()))
            })
        };
        if p.kind.language() == Language::Korean && p.kind != Kind::Mixed {
            return if self.avoided_korean.is_empty() { None } else { find(&self.avoided_korean, &p.text) };
        }
        // English, or the English start of a mixed word. A capital at the start settled English
        // (spec §10.3 rule 2): that spelling was asked for.
        if self.avoided_english.is_empty() || keys.starts_with(|c: char| c.is_ascii_uppercase()) {
            return None;
        }
        let start = p.text.trim_end_matches(|c: char| !c.is_ascii());
        if start.is_empty() || !start.chars().all(|c| c.is_ascii_alphabetic()) {
            return None;
        }
        self.avoided_english.get(&start.to_lowercase()).copied()
    }

    /// The readings, ranked, without duplicates, at most [`MAX_PREDICTIONS`].
    /// any) is only used for stems in front of 曰. With a learned model (spec §10.13) and the
    /// rules ranker, the model ranks; otherwise the rules do.
    pub fn predict(&self, readings: &Readings, context: Context) -> Vec<Prediction> {
        // A phrase staged with spaces (docs/moonshot.md §5.5): decoded word by word.
        if readings.pieces.len() > 1 && readings.pieces.iter().any(|p| p.separator.chars().any(char::is_whitespace)) {
            return self.predict_phrase(readings, context);
        }
        #[cfg(not(feature = "legacy-predict"))]
        if let Some(model) = &self.learned
            && context.ranker == Ranker::Rules
        {
            let ranked = crate::learned::rank(self, model, readings, context);
            let ranked = self.with_kana(ranked, readings, context);
            let ranked = self.with_kanji_conversions(ranked, readings, context);
            return self.with_punctuation(ranked, readings, context);
        }
        let ranked = self.predict_by_rules(readings, context);
        let ranked = self.with_kana(ranked, readings, context);
        let ranked = self.with_kanji_conversions(ranked, readings, context);
        self.with_punctuation(ranked, readings, context)
    }

    /// The Korean and Japanese readings of the keys before the symbol keys at the end, with those
    /// keys as the punctuation they stand for (docs/moonshot.md §5.6: 안녕하세요. for 안녕하세요을,
    /// ありがとう。 for ありがとうる), placed among `ranked` by probability. Each list's scores are
    /// read as log2 odds; a punctuated row's share of its list counts times how often its ending
    /// takes the mark, against how often what the key types follows the same reading
    /// (`punctuation.tsv`, from the corpora: 요 takes . far more than 을, 책 the other way; ます
    /// takes 。, たべ る). The order of `ranked` stays.
    fn with_punctuation(&self, ranked: Vec<Prediction>, readings: &Readings, context: Context) -> Vec<Prediction> {
        use crate::punctuation::{Language as Marks, any_next_bits, follow_bits};
        const PER_OPTION: usize = 3;
        const NEAR_BEST: f64 = 3.0;
        const EXTRA_MARK: f64 = 6.0;
        const MAX_LIFT: f64 = 8.0;
        if readings.punctuated.is_empty() || ranked.is_empty() {
            return ranked;
        }
        // Each row's log2 share of its list; the base list kept in its order (a row never above
        // one ranked over it).
        let mut base = shares(&ranked);
        for i in 1..base.len() {
            base[i] = base[i].min(base[i - 1]);
        }
        let mut extra: Vec<Prediction> = Vec::new();
        for option in &readings.punctuated {
            let before = self.predict(&option.readings, context);
            let shared: Vec<(&Prediction, f64)> = before.iter().zip(shares(&before)).filter(|(_, share)| share.is_finite()).collect();
            for (language, marks) in [(Language::Korean, &option.punctuation), (Language::Japanese, &option.kana_punctuation)] {
                let Some(mark) = marks.chars().next() else { continue };
                let (table, whole, part) = match language {
                    Language::Japanese => (Marks::Japanese, &readings.kana, &option.readings.kana),
                    _ => (Marks::Korean, &readings.korean, &option.readings.korean),
                };
                // What the key typed instead, after the same keys' reading (을 after 안녕하세요, る
                // after ありがとう); when the key's vowel took the final before it (프로젝틍 + ㅢ is
                // 프로젝트의), the whole reading's last character after the rest of it.
                let last_character = || {
                    let last = whole.chars().last()?;
                    follow_bits(table, whole.strip_suffix(last)?, last)
                };
                let typed = whole
                    .strip_prefix(part.as_str())
                    .and_then(|rest| rest.chars().next())
                    .and_then(|next| follow_bits(table, part, next))
                    .or_else(last_character)
                    .unwrap_or_else(|| any_next_bits(table));
                // A second mark (?!, ..) is rare: a cost of its own.
                let more = EXTRA_MARK * (marks.chars().count() - 1) as f64;
                let rows: Vec<&(&Prediction, f64)> = shared.iter().filter(|(p, _)| p.kind.language() == language).collect();
                // Only readings that count among the keys' readings (an eighth of them or more): a
                // mark doesn't make a stray reading (안녕핫에요, のととなしき… for 안녕하세요) worth a
                // row.
                // In the order the readings came: the mark places them, it doesn't reorder them.
                let mut floor = f64::INFINITY;
                for (p, share) in rows.into_iter().filter(|(_, share)| *share >= -NEAR_BEST).take(PER_OPTION) {
                    // Kana rows by the kana typed (a kanji row's 今日 is read きょう).
                    let ending = if language == Language::Japanese { part } else { &p.text };
                    let Some(bits) = follow_bits(table, ending, mark) else { continue };
                    // At most MAX_LIFT, however unlikely the typed character is.
                    let score = (share + (bits - typed).min(MAX_LIFT) - more).min(floor);
                    floor = score;
                    extra.push(Prediction { text: format!("{}{marks}", p.text), kind: p.kind, score });
                }
            }
        }
        extra.sort_by(|a, b| b.score.total_cmp(&a.score));
        let mut out: Vec<Prediction> = Vec::with_capacity(ranked.len() + extra.len());
        let mut extra = extra.into_iter().peekable();
        for (p, share) in ranked.iter().zip(&base) {
            while let Some(e) = extra.next_if(|e| e.score > *share) {
                out.push(e);
            }
            out.push(Prediction { score: *share, ..p.clone() });
        }
        out.extend(extra);
        let mut seen = HashSet::new();
        out.retain(|p| seen.insert(p.text.clone()));
        out.truncate(ranked.len().max(MAX_PREDICTIONS));
        out
    }

    /// A phrase staged with spaces between its words (docs/moonshot.md §5.5, spec §10.10): each
    /// piece is ranked on its own, with the language of the piece before it as its context (as a
    /// committed word gives the next one), its scores turned into probabilities, and a beam search
    /// keeps the likeliest phrases. The whole keys read as Korean and as English, the two phrases
    /// typing straight through would give, stay listed after them.
    fn predict_phrase(&self, readings: &Readings, context: Context) -> Vec<Prediction> {
        const PER_PIECE: usize = 4;
        const BEAM: usize = 8;
        // A piece's readings under the previous piece's language: log2-probabilities, ranked.
        let mut cache: HashMap<(usize, Option<Language>), Vec<(String, Kind, f64)>> = HashMap::new();
        let mut options = |i: usize, previous: Option<Language>| -> Vec<(String, Kind, f64)> {
            cache
                .entry((i, previous))
                .or_insert_with(|| {
                    let ranked = self.predict(&readings.pieces[i].readings, Context { previous, in_phrase: true, ..context });
                    let top: Vec<Prediction> = ranked.into_iter().filter(|p| p.score.is_finite()).take(PER_PIECE).collect();
                    let max = top.iter().map(|p| p.score).fold(f64::NEG_INFINITY, f64::max);
                    let sum: f64 = top.iter().map(|p| (p.score - max).exp2()).sum();
                    top.into_iter().map(|p| (p.text, p.kind, p.score - max - sum.log2())).collect()
                })
                .clone()
        };
        // (text so far, the last piece's language, total log2-probability, every piece Korean)
        let mut beam: Vec<(String, Option<Language>, f64, bool)> = vec![(String::new(), context.previous, 0.0, true)];
        for (i, piece) in readings.pieces.iter().enumerate() {
            let mut next = Vec::new();
            for (text, previous, total, korean) in &beam {
                let choices = options(i, *previous);
                if choices.is_empty() {
                    next.push((format!("{text}{}", piece.separator), *previous, *total, *korean));
                    continue;
                }
                // No bonus for neighbours sharing a language: code-switching text switches often (a
                // bonus of 1 bit or more cost words on the long-context benchmark).
                for (word, kind, lp) in choices {
                    let language = kind.language();
                    next.push((format!("{text}{word}{}", piece.separator), Some(language), total + lp, *korean && language == Language::Korean));
                }
            }
            next.sort_by(|a, b| b.2.total_cmp(&a.2));
            next.dedup_by(|a, b| a.0 == b.0);
            next.truncate(BEAM);
            beam = next;
        }
        let mut out: Vec<Prediction> = Vec::new();
        for (text, _, total, korean) in beam {
            if out.iter().any(|p| p.text == text) {
                continue;
            }
            // Korean words throughout (안녕하세요. 반가워요. with the marks read, §5.6): Korean.
            let kind = if text == readings.korean || korean {
                Kind::Korean
            } else if text == readings.english {
                Kind::English
            } else {
                Kind::Mixed
            };
            out.push(Prediction { text, kind, score: total });
        }
        // The literal readings, typed straight through, as the last options.
        let floor = out.last().map_or(0.0, |p| p.score) - 1.0;
        for (text, kind) in [(&readings.korean, Kind::Korean), (&readings.english, Kind::English)] {
            if !out.iter().any(|p| &p.text == text) {
                out.push(Prediction { text: text.clone(), kind, score: floor });
            }
        }
        out.truncate(MAX_PREDICTIONS.max(4));
        out
    }

    /// [`Predictor::predict`] by the rules only (the legacy ranker), whatever model is loaded.
    pub fn predict_by_rules(&self, readings: &Readings, context: Context) -> Vec<Prediction> {
        let candidates = self.candidates(readings, context);
        self.finish_by_rules(candidates.into_iter().map(|c| c.prediction).collect(), &readings.english)
    }

    /// Every reading of the staged text with its rule-based score, before the avoid list, the
    /// hard guards, duplicates and the cut to [`MAX_PREDICTIONS`]. A scenario weighed 0 scores
    /// −∞ (or isn't generated).
    pub fn candidates(&self, readings: &Readings, context: Context) -> Vec<Candidate> {
        if readings.korean.is_empty() && readings.english.is_empty() {
            return Vec::new();
        }
        let w = context.weights;
        let margin = self.margin(readings, context);
        let english = &readings.english;
        let mut out = vec![
            Prediction { text: readings.korean.clone(), kind: Kind::Korean, score: margin },
            Prediction { text: english.clone(), kind: Kind::English, score: -margin },
        ];
        let mut origins: Vec<(Origin, Option<Split>)> = vec![(Origin::Korean, None), (Origin::English, None)];
        // Case variants only when the word reads as English: otherwise they'd just be clutter.
        let lowercase = english.chars().any(|c| c.is_ascii_lowercase());
        if margin < 0.0 && lowercase {
            out.push(Prediction { text: capitalized(english), kind: Kind::EnglishCapitalized, score: -margin - ALTERNATIVE });
            out.push(Prediction { text: english.to_uppercase(), kind: Kind::EnglishUpper, score: -margin - 2.0 * ALTERNATIVE });
            origins.extend([(Origin::Capitalized, None), (Origin::Upper, None)]);
        }
        // The bare-vowel reading is usually the 2-set one too (ㅏㅣ for k l): then it's one row,
        // weighed by both, so the implied ㅇ weight moves it either way.
        let bare = match context.ranker {
            Ranker::Rules => self.bare(readings, margin),
            Ranker::Travel(costs) => self.bare_by_effort(readings, margin, costs),
            Ranker::Route(preference) => self.bare_by_route(readings, margin, preference),
        };
        let bare = bare.map(|mut bare| {
            // Each implied ㅇ is weight times as likely as its bare vowel.
            bare.score -= readings.implied as f64 * w.bits(Weight::Implied);
            if bare.text == readings.two_set {
                bare.score += w.bits(Weight::TwoSet);
            }
            bare
        });
        for (text, kind) in [(&readings.two_set, Kind::TwoSet), (&readings.keep_finals, Kind::KoreanKeepFinals)] {
            if text.is_empty() || *text == readings.korean || bare.as_ref().is_some_and(|b| b.text == *text) {
                continue;
            }
            // An alternative Korean reading that is a known word (ㅠㅠ, 말이) can win. Not the plain
            // 2-set one when the keys have a Shift+vowel: that Shift asked for the ㅇ (열어 typed
            // d u f ⇧J isn't 여러).
            let shift_vowel = kind == Kind::TwoSet && has_shift_vowel(english);
            let known = if self.korean_words.contains_key(text.as_str()) && !shift_vowel {
                self.lexicon_odds(readings, text)
            } else {
                0.0
            };
            let weight = if kind == Kind::TwoSet { Weight::TwoSet } else { Weight::KeepFinals };
            let score = match context.ranker {
                Ranker::Travel(costs) if kind == Kind::KoreanKeepFinals => {
                    let Some(skipped) = self.skipped(readings, text, costs) else { continue };
                    margin + self.lexicon_odds(readings, text) - skipped
                }
                Ranker::Route(preference) => {
                    let Some(route) = self.route_odds(readings, text, preference) else { continue };
                    margin + self.lexicon_odds(readings, text) + jamo_prior(text, &readings.korean) + route
                }
                _ => margin - ALTERNATIVE + known,
            };
            out.push(Prediction { text: text.clone(), kind, score: score + w.bits(weight) });
            origins.push((if kind == Kind::TwoSet { Origin::TwoSet } else { Origin::KeepFinals }, None));
        }
        // A ㄴ or ㄹ final kept before 이 / 의 / 와 / 에 / 아 on an implied ㅇ (산이 against 사니), by the
        // odds learned from the corpora. Same row as the kept finals; the higher copy stays.
        for kept in &readings.kept {
            // Only what it keeps beyond the typed reading counts (술의 typed with d is in both).
            let bits = self.implied_after_bits(kept) - self.implied_after_bits(&readings.korean);
            // Both ways: a common typed word (아니) keeps beating a rare kept reading (안이).
            let known = self.lexicon_odds(readings, kept);
            let score = margin - ALTERNATIVE + bits + known + w.bits(Weight::KeepFinals);
            out.push(Prediction { text: kept.clone(), kind: Kind::KoreanKeepFinals, score });
            origins.push((Origin::Kept, None));
        }
        // An implied ㄹ after a ㄴ final (전력 for 저녁): an alternative, one step below unless a
        // known word; the counts decide between two known words.
        if w.get(Weight::ImpliedL) > 0.0 {
            for text in &readings.implied_l {
                let score = margin - ALTERNATIVE + self.lexicon_odds(readings, text) + w.bits(Weight::ImpliedL);
                out.push(Prediction { text: text.clone(), kind: Kind::ImpliedL, score });
                origins.push((Origin::ImpliedL, None));
            }
        }
        if let Some(bare) = bare {
            out.push(bare);
            origins.push((Origin::Bare, None));
        }
        if let Some((mut mixed, split)) = self.mixed(readings, -margin, context) {
            // When the whole keys make a Korean word (kanxms is 아무튼; a word in the lexicon or in
            // the hanja dictionary), a split is only an alternative to it, never ahead of it. Junk
            // Korean (slack에 as 님차에) doesn't hold it back.
            // A word started with Shift+vowel (오픈 as H v m s) counts as Korean the same way.
            let word = self.korean_words.contains_key(readings.korean.as_str())
                || self.shift_vowel_start(&readings.english);
            if margin > 0.0 && word {
                mixed.score = mixed.score.min(margin - ALTERNATIVE);
            }
            mixed.score += w.bits(Weight::Mixed);
            out.push(mixed);
            origins.push((Origin::Mixed, Some(split)));
        }
        if let Some(mut head) = self.korean_head(readings, margin) {
            head.score += w.bits(Weight::Mixed);
            out.push(head);
            origins.push((Origin::Head, None));
        }

        // Punctuation inside the word: each piece read on its own (Research(이하, AI)를).
        if readings.pieces.len() > 1 {
            let tops: Vec<String> = readings
                .pieces
                .iter()
                .map(|piece| self.predict(&piece.readings, context).into_iter().next().map_or(String::new(), |p| p.text))
                .collect();
            let latin = |text: &str| text.starts_with(|c: char| c.is_ascii_alphabetic());
            let text: String = readings
                .pieces
                .iter()
                .enumerate()
                .map(|(i, piece)| {
                    // One capital (maybe with digits) next to a Latin piece is a letter, not the jamo
                    // its key types: M&A, O&M, CT-P13, K-IFRS. A lowercase one stays Korean ((USD)에).
                    let keys = &piece.readings.english;
                    // Not a Shift+vowel particle ((Software-as-a-Service)에 as ⇧P): Korean.
                    let particle = keys.chars().count() == 1
                        && keys.starts_with(|c: char| IEUNG_SHIFTS.contains(c))
                        && PARTICLES.contains(&tops[i].as_str());
                    let letter = keys.starts_with(|c: char| c.is_ascii_uppercase())
                        && keys.chars().all(|c| c.is_ascii_alphanumeric())
                        && keys.chars().filter(char::is_ascii_alphabetic).count() == 1
                        && !particle;
                    let beside_latin = (i > 0 && latin(&tops[i - 1])) || tops.get(i + 1).is_some_and(|t| latin(t));
                    let top = if letter && beside_latin { keys } else { &tops[i] };
                    top.clone() + &piece.separator
                })
                .collect();
            // Also when it's a whole-word reading (work), 있습니다.): there the punctuation drags
            // that reading's score down, and this copy, ranked higher, is the one kept.
            let kind = if text == readings.korean {
                Kind::Korean
            } else if text == *english {
                Kind::English
            } else {
                Kind::Mixed
            };
            out.push(Prediction { text, kind, score: margin.abs() + PIECEWISE });
            origins.push((Origin::Pieces, None));
        }
        out.into_iter().zip(origins).map(|(prediction, (origin, split))| Candidate { prediction, origin, split }).collect()
    }

    /// The rules' last steps: the avoid list, a scenario weighed 0 off, ranked, duplicates
    /// removed, the hard guards, at most [`MAX_PREDICTIONS`]. `english`: the keys as typed.
    fn finish_by_rules(&self, mut out: Vec<Prediction>, english: &str) -> Vec<Prediction> {
        // Words the user never means drop, and a strong one is never on top (below).
        let mut strong = HashSet::new();
        for p in &mut out {
            if let Some(is_strong) = self.avoided(p, english) {
                p.score -= AVOIDED;
                if is_strong {
                    strong.insert(p.text.clone());
                }
            }
        }
        // A scenario weighed 0 is off: its readings go.
        out.retain(|p| p.score.is_finite());
        out.sort_by(|a, b| b.score.total_cmp(&a.score));
        let mut seen = HashSet::new();
        out.retain(|p| !p.text.is_empty() && seen.insert(p.text.clone()));
        // Hard guard: a strongly avoided word is never on top, whatever the weights (a user's 10×,
        // or learning). The first other reading moves up.
        let held = |p: &Prediction| strong.contains(&p.text);
        if out.first().is_some_and(held)
            && let Some(i) = out.iter().position(|p| !held(p))
        {
            let other = out.remove(i);
            out.insert(0, other);
        }
        out.truncate(MAX_PREDICTIONS);
        out
    }

    /// The bare-vowel reading, weighed by how likely each implied ㅇ is: likely in real words
    /// (아이, 이거), unlikely in runs of one vowel (유유 → ㅠㅠ) and in chat words.
    fn bare(&self, readings: &Readings, margin: f64) -> Option<Prediction> {
        if readings.implied == 0 || readings.bare.is_empty() || readings.bare == readings.korean {
            return None;
        }
        let mut odds = 0.0;
        if self.korean_words.contains_key(readings.bare.as_str()) {
            odds += self.lexicon_odds(readings, &readings.bare);
        }
        if repeated_implied(&readings.korean).is_some_and(|syllable| self.chat_run(syllable)) {
            odds += LEXICON;
        }
        if self.korean_words.contains_key(readings.korean.as_str()) && !self.korean_words.contains_key(readings.bare.as_str()) {
            odds -= LEXICON;
        }
        Some(Prediction { text: readings.bare.clone(), kind: Kind::BareVowels, score: margin - ALTERNATIVE + odds })
    }

    /// The bare-vowel reading by effort (spec §10.7): the lexicon's evidence, less the effort a
    /// typist who meant the bare vowels skipped (the F6 after each).
    fn bare_by_effort(&self, readings: &Readings, margin: f64, costs: Effort) -> Option<Prediction> {
        if readings.implied == 0 || readings.bare.is_empty() || readings.bare == readings.korean {
            return None;
        }
        let skipped = self.skipped(readings, &readings.bare, costs)?;
        let score = margin + self.lexicon_odds(readings, &readings.bare) - skipped;
        Some(Prediction { text: readings.bare.clone(), kind: Kind::BareVowels, score })
    }

    /// The bare-vowel reading by route preference (spec §10.8).
    fn bare_by_route(&self, readings: &Readings, margin: f64, preference: Preference) -> Option<Prediction> {
        if readings.implied == 0 || readings.bare.is_empty() || readings.bare == readings.korean {
            return None;
        }
        let route = self.route_odds(readings, &readings.bare, preference)?;
        let score = margin + self.lexicon_odds(readings, &readings.bare) + jamo_prior(&readings.bare, &readings.korean) + route;
        Some(Prediction { text: readings.bare.clone(), kind: Kind::BareVowels, score })
    }

    /// log2 P(keys | alternative) − log2 P(keys | Korean reading), in bits, by route preference.
    ///
    /// The Korean reading: the keys are its each-jamo route (taken with the chance the optimized
    /// one wasn't preferred) or a route with shortcuts (taken with the chance they were, for what
    /// they saved over each jamo). An alternative: the keys are its each-jamo route (ㅠㅠ as b b,
    /// taken unless its optimized route saves effort, which for bare vowels it doesn't: F6 is
    /// extra), or they leave out keys it needs, at `omission` bits per press.
    fn route_odds(&self, readings: &Readings, alternative: &str, preference: Preference) -> Option<f64> {
        let keys = crate::plan::keys_of(&readings.english);
        let typed = crate::plan::effort(&keys);
        let log2 = |p: f64| p.clamp(0.02, 1.0).log2();
        // Taking the each-jamo route `jamo` when `best` was there to be had. No cheaper route at
        // all: the each-jamo route is the only one (마리, ㅠㅠ), taken for sure.
        let chose_jamo = |jamo: crate::plan::Effort, best: crate::plan::Effort| {
            if best.presses >= jamo.presses && best.travel >= jamo.travel - 1e-9 {
                return 0.0;
            }
            log2(1.0 - preference.odds(jamo.presses as f64 - best.presses as f64, jamo.travel - best.travel))
        };
        let korean = {
            let jamo_keys = crate::plan::each_jamo(&readings.korean)?;
            let jamo = crate::plan::effort(&jamo_keys);
            if jamo_keys == keys {
                chose_jamo(jamo, crate::plan::cheapest(&readings.korean)?)
            } else {
                log2(preference.odds(jamo.presses as f64 - typed.presses as f64, jamo.travel - typed.travel))
            }
        };
        let jamo_keys = crate::plan::each_jamo(alternative)?;
        // A bare vowel has no JIS Moonshot route (F6 no longer drops the implied ㅇ): its each-jamo
        // keys are the only way, as for 마리.
        let (best, best_keys) = crate::plan::cheapest_route(alternative)
            .unwrap_or_else(|| (crate::plan::effort(&jamo_keys), jamo_keys.clone()));
        let alt = if jamo_keys == keys {
            chose_jamo(crate::plan::effort(&jamo_keys), best)
        } else {
            // Keys left out (or extra): the fewest edits from the typed keys to either route.
            let edits = crate::plan::edits(&keys, &jamo_keys).min(crate::plan::edits(&keys, &best_keys));
            -preference.omission * edits as f64
        };
        Some(alt - korean)
    }

    /// Whether chat doubles this vowel as bare jamo more than as syllables: ㅠㅠ is more common than
    /// 유유, while 워워 beats ㅝㅝ (the lexicon's counts decide).
    fn chat_run(&self, syllable: char) -> bool {
        let medial = ((syllable as u32 - 0xAC00) % 588 / 28) as usize;
        let Some(vowel) = char::from_u32(0x314F + medial as u32) else { return false };
        let count = |s: String| self.korean_words.get(&s).copied().unwrap_or(0);
        let bare = count(format!("{vowel}{vowel}"));
        bare > 0 && bare > count(format!("{syllable}{syllable}"))
    }

    /// The kept-final bits of a reading: for each final followed by a syllable on ㅇ (by its vowel:
    /// 입 as 이), at least [`IMPLIED_AFTER_FLOOR`] where the reading up to it is a known word of two
    /// syllables or more and the rest begins with a particle or ending (기준 + 입니다); else the
    /// corpus odds for ㄴ and ㄹ (산이 against 사니), and a step against for other finals. A double
    /// final always counts as kept (값이).
    fn implied_after_bits(&self, text: &str) -> f64 {
        const FINALS: &str = " ㄱㄲㄳㄴㄵㄶㄷㄹㄺㄻㄼㄽㄾㄿㅀㅁㅂㅄㅅㅆㅇㅈㅊㅋㅌㅍㅎ";
        let chars: Vec<char> = text.chars().collect();
        // A word ending in a 하다 / 되다 modifier (반영한, 사용될) is a verb form, not a noun a
        // particle follows: its final is an ending's (반영하는, not 반영한은).
        let known = |word: &str| {
            word.chars().count() >= 2
                && self.korean_words.contains_key(word)
                && !(word.chars().count() >= 3 && word.ends_with(['한', '할', '된', '될']))
        };
        (1..chars.len())
            .filter_map(|i| {
                let n = (chars[i - 1] as u32).checked_sub(0xAC00).filter(|&n| n < 11172)?;
                let final_ = FINALS.chars().nth((n % 28) as usize)?;
                // The next syllable counts by its vowel, whatever its final (입 as 이).
                let next = (chars[i] as u32).checked_sub(0xAC00).filter(|&n| n < 11172)?;
                let open = char::from_u32(0xAC00 + next - next % 28)?;
                if (open as u32 - 0xAC00) / 588 != 11 {
                    return None; // not a syllable on ㅇ: nothing kept here
                }
                let stem: String = chars[..i].iter().collect();
                // A double final is spelled whole before a vowel (값이, 많이): only its second half
                // moves, in pronunciation (표준 발음법 제14항). When the stem is a word: two keys
                // can also just fall together (지난주 isn't 지낝우).
                if "ㄳㄵㄶㄺㄻㄼㄽㄾㄿㅀㅄ".contains(final_) {
                    return Some(if self.korean_words.contains_key(stem.as_str()) { LEXICON } else { -LEXICON });
                }
                let rest: String = chars[i..].iter().collect();
                // 한글 맞춤법 제14·15항: a final is spelled where it is when a particle or ending
                // follows the word it ends (기준 + 입니다, 당신 + 이, 바둑 + 이). Within one morpheme
                // it moves (사람, 지난주).
                // The rest must be that particle or ending, maybe with a short ending after it
                // (대해서는): 엔진 + 이어링 only starts like one.
                let boundary = known(&stem)
                    && VOWEL_PARTICLES_AND_ENDINGS
                        .iter()
                        .any(|p| rest.strip_prefix(p).is_some_and(|after| ENDING_TAILS.contains(&after)));
                // A ㅇ final "moved" leaves the next syllable as it is (량은 → 랴은), so a particle
                // that only follows consonants (은 을 이 과 으로 …) proves it was kept: 소비량은.
                if final_ == 'ㅇ' && AFTER_CONSONANT_ONLY.contains(&rest.as_str()) {
                    return Some(LEXICON);
                }
                // The corpus odds where learned (ㄴ, ㄹ); else a step against keeping it.
                let odds = self.implied_after.get(&(final_, open)).copied().unwrap_or(-LEXICON);
                Some(if boundary { odds.max(IMPLIED_AFTER_FLOOR) } else { odds })
            })
            .sum()
    }

    /// Lexicon evidence for an alternative Korean reading against the Korean one: a known word
    /// against an unknown one leans its way; when both are words, their counts decide, within two
    /// lexicon steps (ㅠㅠ is far more common than 유유; 저녁 and 전역 are both common, so the
    /// reading the keys spell keeps its place).
    pub(crate) fn lexicon_odds(&self, readings: &Readings, alternative: &str) -> f64 {
        match (self.korean_words.get(alternative), self.korean_words.get(readings.korean.as_str())) {
            (Some(_), None) => 2.0 * LEXICON,
            (None, Some(_)) => -LEXICON,
            (Some(&a), Some(&k)) => (a as f64 / k as f64).log2().clamp(-2.0 * LEXICON, 2.0 * LEXICON),
            (None, None) => 0.0,
        }
    }

    /// The effort, in bits, that the staged keys leave out of the cheapest way to type
    /// `alternative`. None when the alternative can't be typed.
    fn skipped(&self, readings: &Readings, alternative: &str, costs: Effort) -> Option<f64> {
        let full = crate::plan::cheapest(alternative)?;
        let typed = crate::plan::effort_of_keys(&readings.english);
        let presses = full.presses.saturating_sub(typed.presses) as f64;
        let travel = (full.travel - typed.travel).max(0.0);
        Some(costs.press * presses + costs.travel * travel)
    }

    /// The best English-start + particle split (report에, Anthropic에서, RLHF에), if any. It's
    /// offered when the English start reads clearly more English than the whole keys
    /// ([`SPLIT_EDGE`]; a capitalized name always qualifies) and the whole isn't a known English
    /// word. Then the particle is the better explanation for the last keys, and the split ranks
    /// above the whole-word English reading (`english_score`).
    fn mixed(&self, readings: &Readings, english_score: f64, context: Context) -> Option<(Prediction, Split)> {
        // A Japanese particle only after a name (a capitalized start: Claudeの, Tokyoの), in
        // Japanese context or typed on its own: elsewhere the last letter of an English word reads
        // as one (PowerPoint as PowerPoin + t か, JavaScript as JavaScrip + t か), docs/moonshot.md §5.3.
        let japanese = context.previous == Some(Language::Japanese) || (context.previous.is_none() && !context.in_phrase);
        let kana_ok = |s: &Split| s.english.starts_with(|c: char| c.is_ascii_uppercase()) && japanese;
        // Measured the way the starts are, so a Shift+vowel capital weighs the same on both.
        let whole = -self.start_margin(&readings.english);
        let lower = readings.english.to_lowercase();
        if self.english_words.contains_key(&lower) || self.english_nouns.contains(&lower) {
            return None;
        }
        readings
            .splits
            .iter()
            // A particle or ending, or after an acronym a Korean word (LG에너지솔루션은); or a
            // Japanese particle read as kana (Claudeの, docs/moonshot.md §5.3).
            .filter(|s| self.is_ending(&s.korean) || self.acronym_tail(readings, s) || (KANA_PARTICLES.contains(&s.kana.as_str()) && kana_ok(s)))
            // An ending typed in full, its ㅇ included (Claude + d p → Claude에), over a name that
            // takes the d (Clauded + p), unless that start is a known English word (Android에). Only
            // after a capitalized start: a lowercase word keeps its d (frontend에서, method를).
            .filter(|s| {
                if !s.english.starts_with(|c: char| c.is_ascii_uppercase()) {
                    return true;
                }
                let Some(shorter) = s.english.strip_suffix('d') else { return true };
                let lower = s.english.to_lowercase();
                let known = self.english_words.contains_key(&lower) || self.english_nouns.contains(&lower);
                known || !readings.splits.iter().any(|o| o.english == shorter && o.korean == s.korean && self.is_ending(&o.korean))
            })
            // Letters, or letters and digits with a capital (APR1400은, GWh의).
            .filter(|s| {
                s.english.len() >= 2
                    && (s.english.chars().all(|c| c.is_ascii_alphabetic())
                        || (s.english.chars().all(|c| c.is_ascii_alphanumeric()) && s.english.chars().any(|c| c.is_ascii_uppercase())))
            })
            .filter(|s| !self.known_head(readings, s))
            // A Shift+vowel start that isn't an English word (Hv as ⇧ㅗ ㅍ) doesn't split the word
            // when it's two letters or the Korean reading is a known word (오픈 typed ⇧H v m s isn't
            // Hv은). A longer unknown name still splits (Kotlin은).
            .filter(|s| {
                let mut letters = s.english.chars();
                let ieung_start = letters.next().is_some_and(|c| IEUNG_SHIFTS.contains(c)) && letters.all(|c| c.is_ascii_lowercase());
                let lower = s.english.to_lowercase();
                let english = self.english_words.contains_key(&lower) || self.english_nouns.contains(&lower);
                let short = s.english.len() <= 2;
                !(ieung_start && !english && (short || self.korean_words.contains_key(readings.korean.as_str())))
            })
            // Not capitals after capitals: the acronym goes on (4CCL, not 4CC이).
            .filter(|s| {
                let rest = &readings.english[s.english.len()..];
                !(s.english.ends_with(|c: char| c.is_ascii_uppercase()) && rest.chars().all(|c| c.is_ascii_uppercase()))
            })
            // Among starts that are surely English (capitals), a known English word wins, else the
            // longer start (Android에, not Androi에 with d p as 에; SSD에, Docker와), except an acronym that trails off in
            // lowercase keys: those belong to the ending (API를, not APIf을), and among such
            // starts, digits before them or not, the shorter wins (GWh를 and 791GWh를, not GWhf을).
            .map(|s| {
                let trailing = is_acronym(s.english.trim_start_matches(|c: char| c.is_ascii_digit()))
                    && s.english.chars().any(|c| c.is_ascii_lowercase());
                let length = s.english.len() as isize;
                // A start that is a known English word beats a longer one that isn't (Frontier를,
                // not Frontierf을).
                let lower = s.english.to_lowercase();
                let known = self.english_words.contains_key(&lower) || self.english_nouns.contains(&lower);
                // A Korean particle beats a kana one (Claude + r k is Claude가, not Clauder + k as
                // の): a kana particle is read only where no Korean one fits (docs/moonshot.md §5.3).
                let korean = self.is_ending(&s.korean) || self.acronym_tail(readings, s);
                let rank = (korean, !trailing, known, if trailing { -length } else { length });
                (-self.start_margin(&s.english), rank, s)
            })
            // A kana particle's keys (w @ for で) needn't look Korean: after a capitalized start
            // that reads as English it's enough (GitHubで).
            .filter(|(start, _, s)| {
                let kana = !self.is_ending(&s.korean) && !self.acronym_tail(readings, s);
                let capitalized = s.english.starts_with(|c: char| c.is_ascii_uppercase());
                *start >= 99.0
                    || (*start >= SPLIT_START && *start - whole >= SPLIT_EDGE)
                    || (kana && capitalized && *start >= SPLIT_START)
            })
            // A Korean particle first, then the most English start, then the rank.
            .max_by(|a, b| a.1.0.cmp(&b.1.0).then(a.0.min(99.0).total_cmp(&b.0.min(99.0))).then(a.1.cmp(&b.1)))
            .map(|(start, _, s)| {
                // The Korean ending when it is one, else the kana particle.
                let korean = self.is_ending(&s.korean) || self.acronym_tail(readings, s);
                let text = format!("{}{}", s.english, if korean { &s.korean } else { &s.kana });
                (Prediction { text, kind: Kind::Mixed, score: start.max(english_score) + PARTICLE }, s.clone())
            })
    }

    /// After an acronym start (two capitals or more), the rest of the keys read as a Korean word
    /// (LG에너지솔루션은, KB국민카드의, IT서비스): two lowercase keys or more, no bare jamo, two
    /// syllables or an ending, reading as Korean (or a known word), and not the lowercase end of a
    /// unit or name (MWh, MMBtu, Ltd).
    pub(crate) fn acronym_tail(&self, readings: &Readings, split: &Split) -> bool {
        if !split.english.chars().all(|c| c.is_ascii_uppercase() || c.is_ascii_digit()) || !is_acronym(&split.english) {
            return false;
        }
        let Some(tail) = readings.english.get(split.english.len()..) else { return false };
        // A unit, alone or with an ending (MMBtu로: MMBtu + 로, not MMB + 셔로).
        let unit = UNIT_ENDS.contains(&tail)
            || readings.splits.iter().any(|other| {
                other.english.strip_prefix(split.english.as_str()).is_some_and(|end| UNIT_ENDS.contains(&end)) && self.is_ending(&other.korean)
            });
        // Lowercase keys, then any Korean typing shifts too (IT계열사 with ㅖ as ⇧P, LNG수입 with ⇧L).
        let korean_keys = tail.starts_with(|c: char| c.is_ascii_lowercase())
            && tail.chars().all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || KOREAN_SHIFTS.contains(c) || IEUNG_SHIFTS.contains(c));
        tail.len() >= 2
            && !unit
            && korean_keys
            && (split.korean.chars().count() >= 2 || self.is_ending(&split.korean))
            && !split.korean.chars().any(is_jamo)
            && (self.key_margin(tail) > 0.0 || self.korean_words.contains_key(split.korean.as_str()))
    }

    /// The Korean reading is a known word and the split's ending: a split with a lowercase start
    /// then isn't offered (소요되는 is 소요 + 되는, not thy되는).
    fn known_head(&self, readings: &Readings, split: &Split) -> bool {
        if split.english.chars().any(|c| c.is_ascii_uppercase()) {
            return false;
        }
        let Some(head) = readings.korean.strip_suffix(split.korean.as_str()) else { return false };
        !head.is_empty() && self.korean_words.contains_key(head)
    }

    /// A Korean start, then an acronym, maybe with Korean after it (한전KPS, 에너지ICT, 삼성SDI의):
    /// one step above a mixed split of the same keys (gkswjsKPS은), since it explains both scripts.
    /// The start must read as Korean (or be a known word), and the rest be an ending or a Korean
    /// word. Not when every capital is one Korean typing shifts too (있 as ⇧L ⇧T in
    /// 등록되어있으며) and the Korean reading has no bare jamo: that's Korean.
    fn korean_head(&self, readings: &Readings, margin: f64) -> Option<Prediction> {
        let head = readings.head.as_ref()?;
        let acronym: Vec<char> = head.acronym.chars().collect();
        let korean_capitals = acronym.iter().all(|&c| KOREAN_SHIFTS.contains(c) || IEUNG_SHIFTS.contains(c) || c.is_ascii_digit())
            && !acronym.windows(2).any(|w| acronym_pair(w[0], w[1]))
            && !readings.korean.chars().any(is_jamo);
        let start = head.keys.starts_with(|c: char| c.is_ascii_lowercase())
            && head.keys.chars().all(|c| c.is_ascii_lowercase() || c.is_ascii_digit())
            && !head.korean.chars().any(is_jamo)
            && (self.key_margin(&head.keys) > 0.0 || self.korean_words.contains_key(head.korean.as_str()));
        let rest = head.rest.is_empty() || self.is_ending(&head.rest) || (head.rest.chars().count() >= 2 && !head.rest.chars().any(is_jamo));
        if korean_capitals || !start || !rest {
            return None;
        }
        let mut score = margin.abs() + PARTICLE + ALTERNATIVE;
        // A known Korean word as a whole still comes first, as for a mixed split.
        if margin > 0.0 && self.korean_words.contains_key(readings.korean.as_str()) {
            score = score.min(margin - ALTERNATIVE);
        }
        Some(Prediction { text: format!("{}{}{}", head.korean, head.acronym, head.rest), kind: Kind::Mixed, score })
    }

    /// [`Predictor::english_margin`] for the English start of a split: a Shift+vowel capital at the
    /// start (오픈 as H v m s) only leans English, unless the start is a known English word, so it
    /// doesn't make the split certain the way Anthropic's A does.
    pub(crate) fn start_margin(&self, start: &str) -> f64 {
        let margin = self.english_margin(start);
        let lower = start.to_lowercase();
        // Two capitals in a row is an acronym (KPI와, UI가): that settles it as before.
        let acronym = start.chars().take(2).filter(char::is_ascii_uppercase).count() == 2;
        if margin <= -99.0
            && !acronym
            && start.starts_with(|c: char| IEUNG_SHIFTS.contains(c))
            && !self.english_words.contains_key(&lower)
            && !self.english_nouns.contains(&lower)
        {
            return self.english_margin_with(start, true) - CAPITAL_LEAN;
        }
        margin
    }

    /// The keys start with a Shift+vowel capital that doesn't settle English: not an acronym, not a
    /// known English word.
    pub(crate) fn shift_vowel_start(&self, english: &str) -> bool {
        let lower = english.to_lowercase();
        english.starts_with(|c: char| IEUNG_SHIFTS.contains(c))
            && english.chars().take(2).filter(char::is_ascii_uppercase).count() < 2
            && !self.english_words.contains_key(&lower)
            && !self.english_nouns.contains(&lower)
    }

    /// A Korean ending that can follow an English word: a particle, or a known word made of 하다 /
    /// 되다 forms (push했어, deploy했는데).
    pub(crate) fn is_ending(&self, korean: &str) -> bool {
        PARTICLES.contains(&korean)
            || (korean.starts_with(|c: char| VERB_STARTS.contains(c)) && self.korean_words.contains_key(korean))
            || stacked_particles(korean)
    }

    /// Korean minus English, with the rules on top of the model.
    pub(crate) fn margin(&self, readings: &Readings, context: Context) -> f64 {
        let margin = self.margin_of_keys(readings, context);
        // A key right of 0 that typed Korean inside the word (ㅢ on `,`, docs/moonshot.md §9): no
        // English word has that character inside it, so the English reading's capitals and shape
        // don't count against Korean (의약품 typed `,` ⇧I r v n a isn't ,Irvna).
        let last = readings.korean_only_last
            && (readings.korean_only_last_consonant || self.korean_words.contains_key(readings.korean.as_str()));
        let keys = readings.korean_only + last as usize;
        if keys == 0 || context.command {
            return margin;
        }
        margin.max(0.0) + KOREAN_ONLY * keys as f64
    }

    /// [`Predictor::margin`] from the keys' shapes and the lexicons alone.
    fn margin_of_keys(&self, readings: &Readings, context: Context) -> f64 {
        let (korean, english) = (&readings.korean, &readings.english);
        if context.command {
            return -99.0;
        }
        if korean == english {
            return 0.0;
        }
        let mut margin = self.english_margin(english);
        // A word can start with Shift+vowel too (있습니다 as L T t m q s l e k). Unless the keys
        // spell a known English word, that capital leans English instead of settling it, and a
        // known Korean word cancels the lean.
        let lower = english.to_lowercase();
        if margin <= -99.0
            && english.starts_with(|c: char| IEUNG_SHIFTS.contains(c))
            && !self.english_words.contains_key(&lower)
            && !self.english_nouns.contains(&lower)
        {
            let known = self.korean_words.contains_key(korean.as_str());
            margin = self.english_margin_with(english, true) - if known { 0.0 } else { CAPITAL_LEAN };
        }
        // Capitals in a row typed as Shift+consonant and Shift+vowel (가나 as ⇧R ⇧S): a known Korean
        // word, not an acronym.
        if margin <= -99.0
            && self.korean_words.contains_key(korean.as_str())
            && english.chars().filter(char::is_ascii_uppercase).all(|c| KOREAN_SHIFTS.contains(c) || IEUNG_SHIFTS.contains(c))
        {
            margin = self.key_margin(english);
        }
        if self.korean_words.contains_key(korean.as_str()) {
            margin += LEXICON;
        } else {
            margin -= BARE_JAMO * korean.chars().filter(|&c| is_jamo(c)).count() as f64;
        }
        // Digits then a Korean counter or known word (2개, '23년, 3년간): Korean.
        if margin > -99.0 && english.starts_with(|c: char| c.is_ascii_digit() || c == '\'') {
            let rest = korean.trim_start_matches(|c: char| c.is_ascii_digit() || ".,'~".contains(c));
            let counter = COUNTERS.contains(&rest)
                || COUNTERS.iter().any(|c| rest.strip_prefix(c).is_some_and(|p| PARTICLES.contains(&p)))
                || self.korean_words.contains_key(rest);
            if !rest.is_empty() && rest.len() < korean.len() && !rest.chars().any(is_jamo) && counter {
                margin = margin.max(ALTERNATIVE);
            }
        }
        // A word in capitals only (KB, OEM, IT) is an acronym: English, unless its Korean reading is
        // a common word (있 typed ⇧L ⇧T).
        let letters: Vec<char> = english.chars().filter(char::is_ascii_alphabetic).collect();
        if letters.len() >= 2
            && letters.iter().all(char::is_ascii_uppercase)
            && self.korean_words.get(korean.as_str()).copied().unwrap_or(0) < COMMON
        {
            return -99.0;
        }
        match context.previous {
            Some(Language::Korean) => margin += STICKY,
            // A particle right after an English word is the English + particle pattern (report에),
            // typed apart: Service then 에 as ⇧P.
            Some(Language::English) if PARTICLES.contains(&korean.as_str()) => margin += PARTICLE,
            Some(Language::English) => margin -= STICKY,
            Some(Language::Japanese) | None => {}
        }
        // One vowel key is a syllable on an implied ㅇ (ㅗ → 오), and a lone lowercase letter
        // is hardly ever an English word: Korean keeps it, or the first key of every word
        // starting with ㅗ ㅕ ㅐ ㅜ … would show as h u o n.
        if readings.implied > 0 && english.len() == 1 && english.chars().all(|c| c.is_ascii_lowercase()) {
            margin = margin.max(ALTERNATIVE);
        }
        // A known Korean word against keys that spell no English word (몇 against auc, 어디야
        // against jeli, ㅇㅈ against dw): Korean, unless the Shift rule settled it.
        let lower = english.to_lowercase();
        if margin > -99.0
            && self.korean_words.contains_key(korean.as_str())
            && !self.english_words.contains_key(&lower)
            && !self.english_nouns.contains(&lower)
        {
            margin = margin.max(ALTERNATIVE);
        }
        // The user's tuning: per-key bits for each language.
        margin + context.weights.bits(Weight::Korean) - context.weights.bits(Weight::English)
    }

    /// Korean minus English for keys read as English: the Shift rule, the model and the English
    /// word list (the Korean-side rules need the Korean reading).
    pub(crate) fn english_margin(&self, english: &str) -> f64 {
        self.english_margin_with(english, false)
    }

    /// [`Predictor::english_margin`]; `initial_ieung`: a Shift+vowel capital at the start doesn't
    /// settle it either (see `margin`).
    fn english_margin_with(&self, english: &str, initial_ieung: bool) -> f64 {
        let chars: Vec<char> = english.chars().collect();
        for (i, &c) in chars.iter().enumerate() {
            // After any key of the word (a number-row key too: 있 as l 5, then Shift+ㅡ), a Shift
            // on a vowel key is Shift+vowel, starting a syllable on ㅇ (spec §4.2).
            let ieung_ok = (i > 0 && chars[i - 1].is_alphanumeric()) || (i == 0 && initial_ieung);
            if c.is_ascii_uppercase() && !KOREAN_SHIFTS.contains(c) && !(ieung_ok && IEUNG_SHIFTS.contains(c)) {
                return -99.0;
            }
            // Two capitals in a row are an acronym (ESG, KPS), though Shift on any letter key
            // types Korean (docs/moonshot.md §9.2): margin_of_keys lets a known Korean word back.
            if i > 0 && acronym_pair(chars[i - 1], c) {
                return -99.0;
            }
        }
        let mut margin = self.key_margin(english);
        let lower = english.to_lowercase();
        if self.english_words.contains_key(&lower) {
            margin -= LEXICON;
        } else if self.english_nouns.contains(&lower) {
            margin -= NOUN;
        }
        margin
    }

    /// Korean minus English in average log2 probability per key, over the letter runs (the keys
    /// runs, with the three-language model).
    pub(crate) fn key_margin(&self, english: &str) -> f64 {
        // Korean reads a Shift+consonant as the consonant and ㅏ (docs/moonshot.md §9.2): R is r k.
        // English reads the keys as they are. Both totals are per key pressed.
        let korean: String = english
            .chars()
            .flat_map(|c| {
                let a = KOREAN_SHIFTS.contains(c) && c.is_ascii_uppercase();
                [Some(c.to_ascii_lowercase()), a.then_some('k')]
            })
            .flatten()
            .collect();
        let pressed = keys_of(english);
        if pressed == 0 {
            return 0.0;
        }
        if let Some(model) = self.keys.as_ref().filter(|_| self.keys_for_margin) {
            use crate::keymodel::Class;
            let ko = total_of(&korean, |run| model.log_probability(Class::Korean, run));
            let en = total_of(english, |run| model.log_probability(Class::English, run));
            return (ko - en) / pressed as f64;
        }
        let legacy = |text: &str, table: &[u8]| {
            let mut total = 0.0;
            let mut run = String::new();
            for c in text.chars().chain([' ']) {
                if c.is_ascii_alphabetic() {
                    run.push(c.to_ascii_lowercase());
                } else if let Some(d) = c.to_digit(10) {
                    run.push_str(DIGIT_KEYS[d as usize]);
                } else if !run.is_empty() {
                    total += log_probability(table, self.order, &run);
                    run.clear();
                }
            }
            total
        };
        (legacy(&korean, &self.korean) - legacy(english, &self.english)) / pressed as f64
    }

    /// Kana minus the likelier of Korean and English, in log2 probability over the whole staged
    /// word: how much better the JIS kana model explains the keys (docs/moonshot.md §5). None
    /// without the three-language model.
    pub(crate) fn kana_bits(&self, english: &str) -> Option<f64> {
        use crate::keymodel::Class;
        let model = self.keys.as_ref()?;
        let symbols: Vec<u8> = english.chars().filter_map(crate::keymodel::symbol).collect();
        if symbols.is_empty() {
            return None;
        }
        let other = model.log_probability(Class::Korean, &symbols).max(model.log_probability(Class::English, &symbols));
        Some(model.log_probability(Class::Kana, &symbols) - other)
    }
}

/// `score` summed over the runs of keys in `english` (the characters a key types, `keymodel::symbol`),
/// per key (each run's END counts as one). 0 with no keys.
/// Particles stacked two or three deep (들 + 에게, 부터 + 가, 들 + 에게 + 는): an ending after an
/// English word as much as one particle is (client들에게, naming부터가).
fn stacked_particles(korean: &str) -> bool {
    // Particles that can come first in a stack; 이 가 을 를 은 는 의 end one.
    const FIRST: &[&str] = &["들", "에", "에서", "에게", "한테", "께", "부터", "까지", "로", "으로", "만", "처럼", "보다", "과", "와", "랑", "이랑", "하고", "께서"];
    let chars: Vec<char> = korean.chars().collect();
    let text = |a: usize, b: usize| chars[a..b].iter().collect::<String>();
    let piece = |a: usize, b: usize| {
        let t = text(a, b);
        PARTICLES.contains(&t.as_str()) && (b == chars.len() || FIRST.contains(&t.as_str()))
    };
    // can[i]: the first i characters are 1 to 3 particles; parts[i]: how many.
    let n = chars.len();
    let mut parts = vec![usize::MAX; n + 1];
    parts[0] = 0;
    for end in 1..=n {
        for start in 0..end {
            if parts[start] != usize::MAX && parts[start] < 3 && piece(start, end) {
                parts[end] = parts[end].min(parts[start] + 1);
            }
        }
    }
    (2..=3).contains(&parts[n])
}

/// Two capitals in a row with a consonant among them: an acronym's (ESG, KPS), not Korean, though
/// Shift+consonant types Korean (docs/moonshot.md §9.2). Shift+vowels in a row (분야의 as q n s ⇧I
/// ⇧M l) are still Korean.
fn acronym_pair(a: char, b: char) -> bool {
    a.is_ascii_uppercase() && b.is_ascii_uppercase() && (KOREAN_SHIFTS.contains(a) || KOREAN_SHIFTS.contains(b))
}

/// The keys `english` presses, counted per run of keys (each run and its end).
fn keys_of(english: &str) -> usize {
    let mut keys = 0;
    let mut run = 0;
    for c in english.chars().map(Some).chain([None]) {
        match c.and_then(crate::keymodel::symbol) {
            Some(_) => run += 1,
            None if run > 0 => {
                keys += run + 1;
                run = 0;
            }
            None => {}
        }
    }
    keys
}

/// The summed score of each run of keys in `text`.
fn total_of(text: &str, score: impl Fn(&[u8]) -> f64) -> f64 {
    let mut total = 0.0;
    let mut run: Vec<u8> = Vec::new();
    for c in text.chars().map(Some).chain([None]) {
        match c.and_then(crate::keymodel::symbol) {
            Some(s) => run.push(s),
            None if !run.is_empty() => {
                total += score(&run);
                run.clear();
            }
            None => {}
        }
    }
    total
}

/// Each prediction's log2 share of the list, its scores read as log2 odds (as a phrase's pieces
/// are, [`Predictor::predict_phrase`]); −∞ for a row scored −∞.
fn shares(list: &[Prediction]) -> Vec<f64> {
    let max = list.iter().map(|p| p.score).filter(|s| s.is_finite()).fold(f64::NEG_INFINITY, f64::max);
    if !max.is_finite() {
        return vec![f64::NEG_INFINITY; list.len()];
    }
    let sum: f64 = list.iter().filter(|p| p.score.is_finite()).map(|p| (p.score - max).exp2()).sum();
    list.iter().map(|p| if p.score.is_finite() { p.score - max - sum.log2() } else { f64::NEG_INFINITY }).collect()
}

fn log_probability(table: &[u8], order: usize, keys: &str) -> f64 {
    // The keys before this one, oldest first, START-padded: order − 1 of them.
    let mut history = [START; 3];
    let history = &mut history[..order - 1];
    let mut total = 0.0;
    for c in keys.bytes().map(|k| (k - b'a') as usize).chain([END]) {
        let index = history.iter().fold(0, |i, &k| i * SIZE + k) * SIZE + c;
        total -= table[index] as f64 / 16.0;
        history.rotate_left(1);
        if let Some(last) = history.last_mut() {
            *last = c;
        }
    }
    total
}

/// The prior against bare jamo, relative to the Korean reading's own: one step per jamo inside a
/// word (ㅗㅗ백년), one step for a word of jamo only (ㅓㅏㅂ), none for a run of one jamo, the
/// chat genre (ㅠㅠㅠ, ㅋㅋ). A property of the text, not of the keys.
fn jamo_prior(alternative: &str, korean: &str) -> f64 {
    let jamo = |s: &str| s.chars().filter(|&c| is_jamo(c)).count() as f64;
    let mut chars = alternative.chars();
    let first = chars.next();
    let own = if alternative.chars().all(is_jamo) {
        if chars.all(|c| Some(c) == first) { 0.0 } else { 1.0 }
    } else {
        jamo(alternative)
    };
    -BARE_JAMO * (own - jamo(korean))
}

/// Two identical syllables in a row that start on ㅇ and have no final (유유, 우우): the
/// syllable, if any. They may be bare vowels typed for chat (ㅠㅠ); see `chat_run`.
fn repeated_implied(korean: &str) -> Option<char> {
    let chars: Vec<char> = korean.chars().collect();
    chars
        .windows(2)
        .find(|w| {
            let n = w[0] as u32;
            w[0] == w[1] && (0xC544..=0xC7A3).contains(&n) && (n - 0xAC00).is_multiple_of(28)
        })
        .map(|w| w[0])
}

/// Whether the keys have a Shift+vowel after another key (a capital on a vowel key mid-word).
pub(crate) fn has_shift_vowel(english: &str) -> bool {
    let chars: Vec<char> = english.chars().collect();
    (1..chars.len()).any(|i| IEUNG_SHIFTS.contains(chars[i]) && chars[i - 1].is_alphanumeric())
}

fn capitalized(s: &str) -> String {
    let mut chars = s.chars();
    chars.next().map(|c| c.to_uppercase().chain(chars.as_str().to_lowercase().chars()).collect()).unwrap_or_default()
}

/// Two capitals at the start, before any lowercase letter (KB, LG, K2O): an acronym.
pub(crate) fn is_acronym(s: &str) -> bool {
    s.starts_with(|c: char| c.is_ascii_uppercase())
        && s.chars().filter(char::is_ascii_alphabetic).take(2).filter(char::is_ascii_uppercase).count() == 2
}

pub(crate) fn is_jamo(c: char) -> bool {
    ('\u{3131}'..='\u{318E}').contains(&c)
}

impl Language {
    pub fn of(kind: Kind) -> Language {
        kind.language()
    }
}
