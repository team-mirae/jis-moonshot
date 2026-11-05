//! Key maps (spec §3, §3.6 and §6.1).

use crate::key::Key;

/// What a key types.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Out {
    /// A Hangul jamo, fed to the composition automaton.
    Jamo(char),
    /// A character that goes into the text as-is.
    Char(char),
    /// A short run of jamo, real-ㅇ syllables and ゜ breaks (docs/moonshot.md §9.1), as if its keys
    /// were pressed.
    Keys(&'static Action),
}

/// One step of an [`Out::Keys`] action (docs/moonshot.md §9.1).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Step {
    /// A jamo, as its 2-set key types it (a bare vowel joins the consonant before it).
    Jamo(char),
    /// A vowel on a real ㅇ, starting its own syllable (what Shift+vowel does, spec §4.2); its
    /// legend is the syllable (왜, 을), where a bare vowel's is the jamo (ㅙ).
    Ieung(char),
    /// ゜: the syllable ends with no final; the next consonant starts one, a vowel gets an implied ㅇ.
    /// No key has it since `[` types its symbol again (docs/moonshot.md §9.2, 2026-11-05); the
    /// engine still reads it.
    #[allow(dead_code)]
    Break,
    /// Nothing in Korean: a freed slot (docs/moonshot.md §9.2). The key still counts for the kana
    /// and English readings, F9/F10 and Backspace, as any key does. No key has it since 2026-11-05.
    #[allow(dead_code)]
    Blank,
}

/// What a syllable break shows as, alone (docs/moonshot.md §9.1): ㆍ (U+318D, arae-a, an archaic
/// jamo no modern text uses), so a break typed first is on screen and is a reading of its own.
pub const BREAK_MARK: char = '\u{318D}';

/// A key's action: its legend (how docs/moonshot.md §9.2 writes it) and its steps.
#[derive(Debug, PartialEq, Eq)]
pub struct Action {
    pub legend: &'static str,
    pub steps: &'static [Step],
}

impl Out {
    /// The character a key types, for legends; an action's first legend character (다 for 다).
    pub fn char(self) -> char {
        match self {
            Out::Jamo(c) | Out::Char(c) => c,
            Out::Keys(action) => action.legend.chars().next().unwrap_or(' '),
        }
    }

    /// The key's legend in full (다, 왜, ㅡㄹ).
    pub fn legend(self) -> String {
        match self {
            Out::Jamo(c) | Out::Char(c) => c.to_string(),
            Out::Keys(action) => action.legend.to_string(),
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Layout {
    /// The mode's own layout.
    JisKorean,
    /// Standard 2-set on the stock keys (JIS, US ANSI in Mode B, UK ISO in Mode C): the English and
    /// standard 2-set readings. Once the F8 escape (retired, spec §6.1); the name stayed.
    Escaped,
}

/// A mode: one keyboard's key map (spec §3.6, docs/moonshot.md §2). JIS Moonshot is JIS only: the
/// predictive Mode A, the `A_` English modes, and kana typing (`A_KANA`).
///
/// Each variant carries its id as its discriminant (spec §3.6.1). Ids have gaps and a build has
/// only some modes, so nothing may assume they run from 0: look them up with [`Mode::from_id`] and
/// list them with [`Mode::ALL`]. Retired ids (1 B, 2 C, 9 B_QWERTY, 10 CJ_QWERTY) are never reused.
///
/// To add a mode (docs/modes.md has the full contract):
/// 1. Add a variant here with its id (a free one in its range, spec §3.6.1), and add it to
///    [`Mode::ALL`] and [`Mode::from_id`].
/// 2. Give it a [`Mode::code`], a [`Mode::name`] and its tables in [`Mode::korean`] and [`Mode::stock`].
/// 3. Run `cargo test`: `tests/modes.rs` checks every mode against the contract.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
#[repr(u32)]
pub enum Mode {
    /// Mode A, JIS Moonshot: 2-set on a JIS keyboard, read as Korean, English and kana at once.
    #[default]
    A = 0,
    /// `A_QWERTY`, JIS English (spec §3.6.2): not 2-set. Standard JIS QWERTY with JIS symbols on
    /// any keyboard; every key goes to the app, and ⌥ + a letter offers its accented variants
    /// (spec §3.6.3). Built with the `english` feature (on by default).
    #[cfg(feature = "english")]
    AQwerty = 3,
    /// `A_DVORAK`, JIS Dvorak (spec §3.6.4): JIS English with Dvorak's letters. Every symbol keeps its
    /// JIS key pair; the four a letter displaces go to Dvorak's punctuation slots. `english` feature.
    #[cfg(feature = "english")]
    ADvorak = 7,
    /// `A_COLEMAK`, JIS Colemak (spec §3.6.4): JIS English with Colemak's letters; `; +` moves to P.
    /// `english` feature.
    #[cfg(feature = "english")]
    AColemak = 8,
    /// `A_KANA`, JIS Kana (docs/moonshot.md §2.1): kana typing on the JIS kana layout, shown as
    /// hiragana or katakana. Its keys are recorded as Mode A's, so forcing Korean re-reads them.
    AKana = 11,
}

impl Mode {
    /// Every mode this build has, in the order menus list them. Not indexed by id.
    #[cfg(not(feature = "english"))]
    pub const ALL: &'static [Mode] = &[Mode::A, Mode::AKana];
    /// Every mode this build has, in the order menus list them. Not indexed by id.
    #[cfg(feature = "english")]
    pub const ALL: &'static [Mode] = &[Mode::A, Mode::AKana, Mode::AQwerty, Mode::ADvorak, Mode::AColemak];

    /// The stable id over the C ABI and in saved settings (spec §3.6.1). Never reuse or renumber
    /// one.
    pub fn id(self) -> u32 {
        self as u32
    }

    /// The mode with this id, or `None` for an id this build doesn't have (retired, reserved, a
    /// mode left out of the build, or no mode at all). Callers decide what to fall back to.
    pub fn from_id(id: u32) -> Option<Mode> {
        match id {
            0 => Some(Mode::A),
            #[cfg(feature = "english")]
            3 => Some(Mode::AQwerty),
            #[cfg(feature = "english")]
            7 => Some(Mode::ADvorak),
            #[cfg(feature = "english")]
            8 => Some(Mode::AColemak),
            11 => Some(Mode::AKana),
            _ => None,
        }
    }

    /// The mode name (spec §3.6.1): what tools, settings and the public API use. Upper-case snake
    /// case; Mode A keeps its one-letter name.
    pub fn code(self) -> &'static str {
        match self {
            Mode::A => "A",
            #[cfg(feature = "english")]
            Mode::AQwerty => "A_QWERTY",
            #[cfg(feature = "english")]
            Mode::ADvorak => "A_DVORAK",
            #[cfg(feature = "english")]
            Mode::AColemak => "A_COLEMAK",
            Mode::AKana => "A_KANA",
        }
    }

    /// Whether the mode types Korean (2-set, predictive). Everything about Korean, the core
    /// principle included, is about this mode; JIS English and JIS Kana aren't.
    pub fn is_korean(self) -> bool {
        self == Mode::A
    }

    /// Whether the mode types kana (`A_KANA`).
    pub fn is_kana(self) -> bool {
        self == Mode::AKana
    }

    /// Whether the mode is an English one (spec §3.6.2): every key goes to the app.
    pub fn is_english(self) -> bool {
        !self.is_korean() && !self.is_kana()
    }

    /// How a kana mode turns keys into kana (`A_KANA`: the JIS kana layout). Mode A's kana reading
    /// always uses the JIS layout.
    pub fn kana_scheme(self) -> crate::kana::KanaScheme {
        crate::kana::KanaScheme::Jis
    }

    /// The display name shells show (a settings menu, the playground).
    pub fn name(self) -> &'static str {
        match self {
            Mode::A => "JIS Moonshot",
            #[cfg(feature = "english")]
            Mode::AQwerty => "JIS English",
            #[cfg(feature = "english")]
            Mode::ADvorak => "JIS Dvorak",
            #[cfg(feature = "english")]
            Mode::AColemak => "JIS Colemak",
            Mode::AKana => "JIS Kana",
        }
    }

    /// The symbol keys the Dedicated symbol keys setting turns off (spec §3.6). No JIS mode has
    /// any; kept for hosts built against it.
    pub fn optional_keys(self) -> &'static [Key] {
        &[]
    }

    /// What the non-letter keys type: (unshifted, shifted). `None` for keys the mode leaves out:
    /// they commit and pass through to the app. `symbols` is the Dedicated symbol keys setting.
    fn korean(self, key: Key, symbols: bool) -> Option<(Out, Out)> {
        if !symbols && self.optional_keys().contains(&key) {
            return self.stock(key).map(|(base, shifted)| (Out::Char(base), Out::Char(shifted)));
        }
        match self {
            // JIS Kana records its keys as Mode A's (docs/moonshot.md §4).
            Mode::A | Mode::AKana => jis_korean(key),
            // Not Korean: the JIS symbols as they are.
            #[cfg(feature = "english")]
            Mode::AQwerty | Mode::ADvorak | Mode::AColemak => {
                self.stock(key).map(|(base, shifted)| (Out::Char(base), Out::Char(shifted)))
            }
        }
    }

    /// The stock keyboard's characters on the non-letter keys: standard 2-set on the stock keys
    /// and the English reading of every key (predictions, F9/F10's symbols).
    /// It must cover every key [`Mode::korean`] maps.
    fn stock(self, key: Key) -> Option<(char, char)> {
        match self {
            Mode::A | Mode::AKana => stock_jis(key),
            #[cfg(feature = "english")]
            Mode::AQwerty => stock_jis(key),
            #[cfg(feature = "english")]
            Mode::ADvorak | Mode::AColemak => stock_jis(self.qwerty_key(key)),
        }
    }

    /// The JIS English key whose characters `key` types in this mode: itself, except in JIS Dvorak
    /// and JIS Colemak, which move keys around (spec §3.6.4). ⌘ and ⌥ still use `key` as it is.
    pub fn qwerty_key(self, key: Key) -> Key {
        match self {
            #[cfg(feature = "english")]
            Mode::ADvorak => dvorak(key),
            #[cfg(feature = "english")]
            Mode::AColemak => colemak(key),
            _ => key,
        }
    }

    /// The Latin letter `key` types in an English mode (lower case), or `None` for a key that types
    /// something else there. Korean and kana modes have none.
    pub fn letter(self, key: Key) -> Option<char> {
        if !self.is_english() { None } else { self.qwerty_key(key).letter() }
    }
}

/// What `key` types in `layout` in Mode A. `None` for keys outside the map (F-keys, Space, …).
pub fn output(layout: Layout, key: Key, shift: bool) -> Option<Out> {
    output_in(Mode::A, layout, key, shift)
}

/// What `key` types in `layout` in `mode`, with the mode's symbol keys on (the default).
pub fn output_in(mode: Mode, layout: Layout, key: Key, shift: bool) -> Option<Out> {
    output_with(mode, true, layout, key, shift)
}

/// What `key` types in `layout` in `mode`, with the Dedicated symbol keys setting `symbols`
/// (spec §3.6; only modes with [`Mode::optional_keys`] read it).
pub fn output_with(mode: Mode, symbols: bool, layout: Layout, key: Key, shift: bool) -> Option<Out> {
    if mode.is_english() {
        // JIS English: the Latin letters, upper-case with Shift.
        if let Some(letter) = mode.letter(key) {
            return Some(Out::Char(if shift { letter.to_ascii_uppercase() } else { letter }));
        }
    } else if layout == Layout::JisKorean
        && let Some((base, shifted)) = moonshot_letter(key)
    {
        return Some(if shift { shifted } else { base });
    } else if let Some((base, shifted)) = two_set(key) {
        return Some(Out::Jamo(if shift { shifted } else { base }));
    }
    let (base, shifted) = match layout {
        Layout::JisKorean => mode.korean(key, symbols)?,
        Layout::Escaped => {
            let (base, shifted) = mode.stock(key)?;
            (Out::Char(base), Out::Char(shifted))
        }
    };
    Some(if shift { shifted } else { base })
}

/// Standard 2-set (KS X 5002) on the 26 letter keys: (unshifted, shifted).
fn two_set(key: Key) -> Option<(char, char)> {
    use Key::*;
    let pair = match key {
        KeyQ => ('ㅂ', 'ㅃ'),
        KeyW => ('ㅈ', 'ㅉ'),
        KeyE => ('ㄷ', 'ㄸ'),
        KeyR => ('ㄱ', 'ㄲ'),
        KeyT => ('ㅅ', 'ㅆ'),
        KeyO => ('ㅐ', 'ㅒ'),
        KeyP => ('ㅔ', 'ㅖ'),
        _ => {
            let j = match key {
                KeyY => 'ㅛ',
                KeyU => 'ㅕ',
                KeyI => 'ㅑ',
                KeyA => 'ㅁ',
                KeyS => 'ㄴ',
                KeyD => 'ㅇ',
                KeyF => 'ㄹ',
                KeyG => 'ㅎ',
                KeyH => 'ㅗ',
                KeyJ => 'ㅓ',
                KeyK => 'ㅏ',
                KeyL => 'ㅣ',
                KeyZ => 'ㅋ',
                KeyX => 'ㅌ',
                KeyC => 'ㅊ',
                KeyV => 'ㅍ',
                KeyB => 'ㅠ',
                KeyN => 'ㅜ',
                KeyM => 'ㅡ',
                _ => return None,
            };
            (j, j)
        }
    };
    Some(pair)
}

/// JIS Moonshot's letter keys (docs/moonshot.md §9.2): standard 2-set. Shift on a consonant key
/// types the consonant and ㅏ (Shift+R → 가; ㅃ ㅉ ㄸ ㄲ ㅆ are on 1–5), and Shift on every vowel key,
/// O and P included, starts a syllable on ㅇ (spec §4.2; ㅒ ㅖ are on : and ;). The standard 2-set
/// reading (`Layout::Escaped`) keeps KS X 5002.
fn moonshot_letter(key: Key) -> Option<(Out, Out)> {
    let (base, _) = two_set(key)?;
    let shifted = consonant_a(base).map_or(Out::Jamo(base), Out::Keys);
    Some((Out::Jamo(base), shifted))
}

/// Shift+consonant (docs/moonshot.md §9.2): the consonant, then ㅏ.
fn consonant_a(consonant: char) -> Option<&'static Action> {
    use Step::Jamo as S;
    macro_rules! ca {
        ($legend:literal, $c:literal) => {
            &Action { legend: $legend, steps: &[S($c), S('ㅏ')] }
        };
    }
    Some(match consonant {
        'ㄱ' => ca!("가", 'ㄱ'),
        'ㄴ' => ca!("나", 'ㄴ'),
        'ㄷ' => ca!("다", 'ㄷ'),
        'ㄹ' => ca!("라", 'ㄹ'),
        'ㅁ' => ca!("마", 'ㅁ'),
        'ㅂ' => ca!("바", 'ㅂ'),
        'ㅅ' => ca!("사", 'ㅅ'),
        'ㅇ' => ca!("아", 'ㅇ'),
        'ㅈ' => ca!("자", 'ㅈ'),
        'ㅊ' => ca!("차", 'ㅊ'),
        'ㅋ' => ca!("카", 'ㅋ'),
        'ㅌ' => ca!("타", 'ㅌ'),
        'ㅍ' => ca!("파", 'ㅍ'),
        'ㅎ' => ca!("하", 'ㅎ'),
        _ => return None,
    })
}

/// Mode A's non-letter keys: the number row (spec §3.1) and the 12 keys right of 0 (docs/moonshot.md
/// §9.2): (unshifted, shifted). Since 2026-11-05 only − ; : , type Korean there.
fn jis_korean(key: Key) -> Option<(Out, Out)> {
    use Key::*;
    use Out::{Char as C, Jamo as J, Keys as K};
    use Step::Ieung as I;
    let pair = match key {
        Digit1 => (J('ㅃ'), C('1')),
        Digit2 => (J('ㅉ'), C('2')),
        Digit3 => (J('ㄸ'), C('3')),
        Digit4 => (J('ㄲ'), C('4')),
        Digit5 => (J('ㅆ'), C('5')),
        Digit6 => (J('ㅟ'), C('6')),
        Digit7 => (J('ㅝ'), C('7')),
        Digit8 => (J('ㅚ'), C('8')),
        Digit9 => (J('ㅢ'), C('9')),
        Digit0 => (J('ㅘ'), C('0')),
        Minus => (J('ㅙ'), K(&Action { legend: "왜", steps: &[I('ㅙ')] })),
        // docs/moonshot.md §9.2 (2026-11-05, the owner's call): ^ ¥ @ [ ] . / ろ type their stock JIS
        // symbols again, on both layers, so punctuation has its keys.
        Equal | IntlYen | BracketLeft | BracketRight | Backslash | Slash | IntlRo => {
            let (alone, shifted) = stock_jis(key)?;
            (C(alone), C(shifted))
        }
        // Shift+`.` is a period too (2026-11-06), as in JIS Kana (。), the way Shift+`,` is a comma.
        Period => (C('.'), C('.')),
        Semicolon => (J('ㅖ'), K(&Action { legend: "예", steps: &[I('ㅖ')] })),
        Quote => (J('ㅒ'), K(&Action { legend: "얘", steps: &[I('ㅒ')] })),
        Comma => (J('ㅢ'), C(',')),
        _ => return None,
    };
    Some(pair)
}

/// JIS Dvorak's keys (spec §3.6.4): Dvorak's letters, each key typing the JIS English key named.
/// The JIS symbol keys a letter displaces go where Dvorak has punctuation: `, <` to W, `. >` to E,
/// `; +` to Z, and `/ ?` to Q (Dvorak's `' "`, which JIS has on Shift+7 and Shift+2).
#[cfg(feature = "english")]
fn dvorak(key: Key) -> Key {
    use Key::*;
    match key {
        KeyQ => Slash,
        KeyW => Comma,
        KeyE => Period,
        KeyR => KeyP,
        KeyT => KeyY,
        KeyY => KeyF,
        KeyU => KeyG,
        KeyI => KeyC,
        KeyO => KeyR,
        KeyP => KeyL,
        KeyS => KeyO,
        KeyD => KeyE,
        KeyF => KeyU,
        KeyG => KeyI,
        KeyH => KeyD,
        KeyJ => KeyH,
        KeyK => KeyT,
        KeyL => KeyN,
        Semicolon => KeyS,
        KeyZ => Semicolon,
        KeyX => KeyQ,
        KeyC => KeyJ,
        KeyV => KeyK,
        KeyB => KeyX,
        KeyN => KeyB,
        Comma => KeyW,
        Period => KeyV,
        Slash => KeyZ,
        _ => key,
    }
}

/// JIS Colemak's keys (spec §3.6.4): Colemak's letters, each key typing the JIS English key named.
/// `; +`, displaced by o, goes to P, Colemak's semicolon.
#[cfg(feature = "english")]
fn colemak(key: Key) -> Key {
    use Key::*;
    match key {
        KeyE => KeyF,
        KeyR => KeyP,
        KeyT => KeyG,
        KeyY => KeyJ,
        KeyU => KeyL,
        KeyI => KeyU,
        KeyO => KeyY,
        KeyP => Semicolon,
        KeyS => KeyR,
        KeyD => KeyS,
        KeyF => KeyT,
        KeyG => KeyD,
        KeyJ => KeyN,
        KeyK => KeyE,
        KeyL => KeyI,
        Semicolon => KeyO,
        KeyN => KeyK,
        _ => key,
    }
}

/// The stock JIS characters on the non-letter keys: Mode A's stock keyboard and English readings.
fn stock_jis(key: Key) -> Option<(char, char)> {
    use Key::*;
    let pair = match key {
        Digit1 => ('1', '!'),
        Digit2 => ('2', '"'),
        Digit3 => ('3', '#'),
        Digit4 => ('4', '$'),
        Digit5 => ('5', '%'),
        Digit6 => ('6', '&'),
        Digit7 => ('7', '\''),
        Digit8 => ('8', '('),
        Digit9 => ('9', ')'),
        Digit0 => ('0', '0'),
        Minus => ('-', '='),
        Equal => ('^', '~'),
        IntlYen => ('¥', '|'),
        BracketLeft => ('@', '`'),
        BracketRight => ('[', '{'),
        Semicolon => (';', '+'),
        Quote => (':', '*'),
        Backslash => (']', '}'),
        Comma => (',', '<'),
        Period => ('.', '>'),
        Slash => ('/', '?'),
        IntlRo => ('\\', '_'),
        // Not on a full JIS board (半角/全角 is there), but the HHKB sends it: ` and ~.
        Backquote => ('`', '~'),
        _ => return None,
    };
    Some(pair)
}
