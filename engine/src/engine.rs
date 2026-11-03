//! The unconfirmed segment, commit rules and F6–F10 (spec §5 and §6).

#[cfg(feature = "predict")]
use std::sync::Arc;

use crate::hangul::{self, Composer, Options};
#[cfg(feature = "predict")]
use crate::learn::{self, Event, Learner};
#[cfg(feature = "predict")]
use crate::predict::{Context, Head, Kind, Language, Piece, Prediction, Predictor, Ranker, Readings, Split, Weight, Weights};
use crate::key::{Key, Mods};
use crate::layout::{self, Layout, Mode, Out, Step};

/// Candidates per page of a list (the English modes' letter variants); Digit1–Digit9 pick on the
/// current page.
pub const PAGE_SIZE: usize = 9;

/// One row of a list the engine shows (the English modes' letter variants, spec §3.6.3).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Candidate {
    pub text: String,
    /// A note beside it. Often empty.
    pub comment: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
enum Token {
    /// A jamo, with the Latin letter or digit of the key that typed it (for F9/F10), and the
    /// key's JIS kana (the kana reading, docs/moonshot.md §5).
    Jamo { jamo: char, raw: Option<char>, kana: Option<char> },
    /// Text from a conversion or a chosen prediction.
    Text(char),
    /// A character typed by a non-letter key, with what that key types on stock JIS (for the
    /// English reading) and its JIS kana.
    Typed(char, char, Option<char>),
    /// A bare vowel marker: JIS Korean's F6 dropped an implied ㅇ with it. JIS Moonshot's F6 types
    /// hiragana instead, so nothing makes one now; the composer still reads it.
    #[allow(dead_code)]
    Bare,
    /// Shift+vowel (spec §4.2): the vowel starts its own syllable on ㅇ, leaving the previous
    /// final where it is.
    IeungVowel { jamo: char, raw: Option<char>, kana: Option<char> },
    /// ゜ (docs/moonshot.md §9.1): the syllable ends with no final. `raw` and `kana` as for a jamo
    /// when it's the key's first step (the ゜ key itself), none when it ends a longer action (로゜).
    Break { raw: Option<char>, kana: Option<char> },
    /// A freed slot's key (docs/moonshot.md §9.2): nothing in Korean, but still a key for the
    /// kana and English readings, F9/F10 and Backspace.
    Blank { raw: Option<char>, kana: Option<char> },
    /// A jamo after the first one of a key's action (the ㅏ of 다): it composes, but the key is the
    /// first token's (for the English and kana readings and F9/F10).
    Part(char),
}

/// How the staged keys are shown and committed (docs/moonshot.md §4). The C ABI id is the
/// discriminant (`JK_SCRIPT_*`).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
#[repr(u32)]
pub enum Script {
    /// The top prediction: Korean, English or kana (Mode A's default).
    #[default]
    Auto = 0,
    /// JIS Korean as composed, forced (かな / 変換 in Mode A).
    Korean = 1,
    /// The kana reading in hiragana (F6, `A_KANA`).
    Hiragana = 2,
    /// The kana reading in katakana (F7, `A_KANA`).
    Katakana = 3,
    /// JIS Korean as composed without the implied ㅇ, forced (Shift+かな / Shift+変換: ㅏ stays ㅏ).
    KoreanBare = 4,
}

impl Script {
    pub fn id(self) -> u32 {
        self as u32
    }

    /// Whether the staged keys show as kana.
    pub fn is_kana(self) -> bool {
        matches!(self, Script::Hiragana | Script::Katakana)
    }
}

pub use crate::width::Width;

/// An edit of the app's own text, which the host applies with what it can read around the caret
/// (`jk_host_edit`): 半角/全角 with nothing staged (docs/moonshot.md §4.1).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum HostEdit {
    /// The selection or the word at the caret to this width.
    Width(Width),
}

/// An F10 conversion in progress (spec §6.2).
#[derive(Clone)]
struct Conversion {
    start: usize,
    original: Vec<Token>,
    /// 0 = as typed, 1 = UPPER, 2 = Capitalized.
    case: u8,
}

/// The candidate list to show, while one is open.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Candidates {
    pub items: Vec<Candidate>,
    pub selected: usize,
}

/// The result of one engine call.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Output {
    /// Text to insert into the document, before the preedit.
    pub commit: String,
    /// The whole unconfirmed segment, to show as marked text. Empty when nothing is unconfirmed.
    pub preedit: String,
    /// `false`: after applying `commit` and `preedit`, pass the key on to the app.
    pub consumed: bool,
    /// The list to show (the English modes' letter variants), while one is open.
    pub candidates: Option<Candidates>,
    /// The same keystrokes read as English on stock JIS (see [`Engine::english`]).
    pub english: String,
    /// An edit of the app's own text for the host to apply ([`Engine::host_edit`]): 半角/全角
    /// with nothing staged (docs/moonshot.md §4.1).
    pub host_edit: Option<HostEdit>,
}

#[derive(Clone)]
pub struct Engine {
    segment: Vec<Token>,
    conversion: Option<Conversion>,
    /// The mode: which keyboard's key map (spec §3.6).
    mode: Mode,
    /// How the staged keys show (docs/moonshot.md §4): Auto in Mode A until forced.
    script: Script,
    /// The script `A_KANA` types in: hiragana or katakana, remembered across switches.
    kana_script: Script,
    /// F8 (docs/moonshot.md §4): kanji conversions on top for this segment (or the next, pressed
    /// with nothing staged), until the next commit.
    kanji: bool,
    /// Setting: Dedicated symbol keys (spec §3.6), for modes with optional symbol keys.
    symbols: bool,
    enter_commits: bool,
    /// Setting: ⌥Space types a full-width space (Korean and kana) or `_` (English). macOS only:
    /// on Windows Alt+Space is the window menu (docs/moonshot.md §4).
    option_space: bool,
    /// The English modes' ⌥ + letter list of variants, while one is open (spec §3.6.3).
    #[cfg(feature = "english")]
    held: Option<crate::hold::Held>,
    /// The predictive layer; `None` is the deterministic lite engine.
    #[cfg(feature = "predict")]
    predictor: Option<Arc<Predictor>>,
    /// The language of the last committed word, for the predictor.
    #[cfg(feature = "predict")]
    previous: Option<Language>,
    /// The user's tuning of the predictions.
    #[cfg(feature = "predict")]
    weights: Weights,
    /// How the implied-ㅇ alternatives are ranked (spec §10.7).
    #[cfg(feature = "predict")]
    ranker: Ranker,
    /// Learning from the user's choices (spec §10.11): off unless the host turns it on.
    #[cfg(feature = "predict")]
    learner: Learner,
    #[cfg(feature = "predict")]
    learning: bool,
    /// Whether choices are recorded while learning is on: off for apps the user excluded (what was
    /// learned still applies there).
    #[cfg(feature = "predict")]
    recording: bool,
    /// Scenarios the user set by hand: their weight wins over the learned one.
    #[cfg(feature = "predict")]
    manual: [bool; Weight::ALL.len()],
    /// The scenario of a top prediction just accepted, for a Backspace right after (regret).
    #[cfg(feature = "predict")]
    /// With how many characters were typed after it (Space's space, a passed-on Enter's line): a
    /// Backspace deleting those isn't regret yet.
    accepted: Option<(Weight, usize)>,
    /// The learner changed since the host last exported it.
    #[cfg(feature = "predict")]
    learned_changed: bool,
}

impl Default for Engine {
    fn default() -> Self {
        Self::new()
    }
}

impl Engine {
    pub fn new() -> Self {
        Engine {
            segment: Vec::new(),
            conversion: None,
            mode: Mode::A,
            script: Script::Auto,
            kana_script: Script::Hiragana,
            kanji: false,
            symbols: true,
            enter_commits: false,
            option_space: false,
            #[cfg(feature = "english")]
            held: None,
            #[cfg(feature = "predict")]
            predictor: None,
            #[cfg(feature = "predict")]
            previous: None,
            #[cfg(feature = "predict")]
            weights: Weights::default(),
            #[cfg(feature = "predict")]
            ranker: Ranker::default(),
            #[cfg(feature = "predict")]
            learner: Learner::default(),
            #[cfg(feature = "predict")]
            learning: false,
            #[cfg(feature = "predict")]
            recording: true,
            #[cfg(feature = "predict")]
            manual: [false; Weight::ALL.len()],
            #[cfg(feature = "predict")]
            accepted: None,
            #[cfg(feature = "predict")]
            learned_changed: false,
        }
    }

    /// Setting: plain Enter commits without passing Enter on, as Shift+Enter does (spec §5.1).
    pub fn set_enter_commits(&mut self, on: bool) {
        self.enter_commits = on;
    }

    /// Setting: ⌥Space types a full-width space in Mode A and JIS Kana (committing what's staged
    /// first) and `_` in the English modes. Off by default; the macOS shell turns it on.
    pub fn set_option_space(&mut self, on: bool) {
        self.option_space = on;
    }

    /// Setting: the mode (spec §3.6, docs/moonshot.md §2). Takes effect from the next key. Mode A
    /// starts in the Auto script, `A_KANA` in the kana script it used last.
    /// Setting the mode it's already in changes nothing: shells echo the engine's own switches
    /// back (macOS reports the input source it selected), and a forced script must survive that.
    pub fn set_mode(&mut self, mode: Mode) {
        if mode == self.mode {
            return;
        }
        self.mode = mode;
        self.script = if mode.is_kana() { self.kana_script } else { Script::Auto };
    }

    /// How the staged keys show now (docs/moonshot.md §4).
    pub fn script(&self) -> Script {
        self.script
    }

    /// The かな / 変換 key (docs/moonshot.md §4): from an English mode to Mode A; in Mode A, forces
    /// the staged keys' Korean reading (nothing staged: nothing); from `A_KANA` to Mode A with the
    /// staged keys forced Korean. Always consumed. Shells read [`Engine::mode`] after it.
    pub fn kana_key(&mut self) -> Output {
        self.kana_key_with(Script::Korean)
    }

    /// Shift+かな / Shift+変換 (docs/moonshot.md §4): [`Engine::kana_key`], but the staged keys
    /// are forced bare, with no implied ㅇ (ㅏ stays ㅏ, not 아).
    pub fn kana_key_bare(&mut self) -> Output {
        self.kana_key_with(Script::KoreanBare)
    }

    /// Applies `edit` to the app's text around the caret (docs/moonshot.md §4.1): `selected` is
    /// the selection (empty without one), `before` and `after` the text either side of it. `None`
    /// when nothing changes.
    pub fn host_edit(&mut self, edit: HostEdit, before: &str, selected: &str, after: &str) -> Option<crate::width::WidthEdit> {
        match edit {
            HostEdit::Width(width) => crate::width::width_edit(before, selected, after, width),
        }
    }

    fn kana_key_with(&mut self, forced: Script) -> Output {
        if self.mode.is_english() {
            let commit = self.take_segment();
            self.set_mode(Mode::A);
            return Output { commit, ..self.unchanged(true) };
        }
        self.restore_typed();
        self.mode = Mode::A;
        self.kanji = false;
        self.script = if self.segment.is_empty() { Script::Auto } else { forced };
        self.unchanged(true)
    }

    /// F6 / F7 (docs/moonshot.md §4): the staged keys read as kana in `script`. In Mode A, for this
    /// segment only (or the next, with nothing staged): a commit goes back to Auto. In `A_KANA`,
    /// the script it keeps.
    fn to_kana(&mut self, script: Script) -> Output {
        self.restore_typed();
        if self.mode.is_kana() {
            self.kana_script = script;
        }
        self.kanji = false;
        self.script = script;
        self.unchanged(true)
    }

    /// F8 (docs/moonshot.md §4): kanji conversions of the staged keys' kana reading on top, until
    /// the next commit (with nothing staged, for the next segment). In Mode A the script goes back
    /// to Auto (after F6 / F7 too); `A_KANA` keeps its script, which the kana reading below the
    /// conversions is shown in.
    fn to_kanji(&mut self) -> Output {
        self.restore_typed();
        self.kanji = true;
        if self.mode.is_korean() {
            self.script = Script::Auto;
        }
        self.unchanged(true)
    }

    /// Whether F8 asked for kanji conversions for the staged keys.
    pub fn kanji_forced(&self) -> bool {
        self.kanji
    }

    /// Undoes an F9/F10 conversion, so the staged keys are as typed again (forcing a script
    /// re-reads them).
    fn restore_typed(&mut self) {
        if let Some(conversion) = self.conversion.take() {
            self.segment.truncate(conversion.start);
            self.segment.extend(conversion.original);
        }
    }

    /// Setting: Dedicated symbol keys (spec §3.6), on by default. Off, the mode's optional symbol
    /// keys ([`Mode::optional_keys`], Mode C's [ ] and the ISO extra key) type the mode's stock
    /// characters (UK ISO in Mode C). Modes without optional keys ignore it.
    pub fn set_dedicated_symbols(&mut self, on: bool) {
        self.symbols = on;
    }

    /// Whether the mode's optional symbol keys are on (spec §3.6).
    pub fn dedicated_symbols(&self) -> bool {
        self.symbols
    }

    /// The mode (spec §3.6).
    pub fn mode(&self) -> Mode {
        self.mode
    }

    /// Always false: the F8 escape is retired (spec §6). Kept for hosts built against it.
    pub fn is_escaped(&self) -> bool {
        false
    }

    /// The IME was switched back to. Nothing to reset since the F8 escape was retired; kept for
    /// hosts that call it.
    pub fn activate(&mut self) {}

    /// Commits everything unconfirmed. Call on focus loss, app switch and IME switch.
    pub fn flush(&mut self) -> Output {
        // Not the user's choice: nothing is learned from it (spec §10.11).
        self.commit_quietly(String::new(), true)
    }

    /// Discards everything unconfirmed, for hosts that commit a reading of their own (the
    /// Minecraft mod picks between [`Engine::preedit`] and [`Engine::english`]).
    pub fn clear(&mut self) {
        self.segment.clear();
        self.conversion = None;
        self.kanji = false;
    }

    /// The unconfirmed keystrokes read as English: each key as it types on the stock keyboard,
    /// JIS or US ANSI (letters with their case, digits, the keyboard's symbols). Converted text stays as shown.
    pub fn english(&self) -> String {
        english_of(&self.segment)
    }

    /// The unconfirmed text without the implied ㅇ (spec §4.2): what F6 would leave of plain
    /// implied syllables (ㅠ for 유). For resolving keys (crate::resolve).
    pub fn preedit_without_implied(&self) -> String {
        compose_with(&self.segment, Options { no_implied: true, ..Options::default() })
    }

    /// The unconfirmed text, as the script shows it.
    pub fn preedit(&self) -> String {
        self.shown(&self.segment)
    }

    /// Staged tokens as the script shows them: kana in a kana script, else Korean.
    fn shown(&self, segment: &[Token]) -> String {
        match self.script {
            Script::Hiragana => kana_of(segment),
            // Only the typed keys in katakana: text already chosen (a kanji prediction's 私は, a
            // pick) stays as it is.
            Script::Katakana => segment
                .chunk_by(|a, b| matches!(a, Token::Text(_)) == matches!(b, Token::Text(_)))
                .map(|run| if matches!(run[0], Token::Text(_)) { kana_of(run) } else { crate::kana::katakana(&kana_of(run)) })
                .collect(),
            Script::Auto | Script::Korean => compose(segment),
            Script::KoreanBare => compose_with(segment, Options { no_implied: true, ..Options::default() }),
        }
    }

    /// The staged keys read as kana, in hiragana (docs/moonshot.md §5), whatever the script.
    pub fn kana(&self) -> String {
        kana_of(&self.segment)
    }

    pub fn key(&mut self, key: Key, mods: Mods) -> Output {
        if key == Key::Modifier {
            return self.unchanged(false);
        }
        // JIS English (spec §3.6.2): nothing Korean applies; keys go to the app.
        #[cfg(feature = "english")]
        if self.mode.is_english() {
            return self.english_key(key, mods);
        }
        // A variants list still open from an English mode (the mode changed under it): what it
        // shows stays, as plain text before what's typed now.
        #[cfg(feature = "english")]
        if let Some(held) = self.held.take() {
            self.segment.insert(0, Token::Text(held.shown()));
        }
        // Backspace right after accepting a top prediction: a weak sign it was wrong.
        #[cfg(feature = "predict")]
        if let Some((kind, after)) = self.accepted.take()
            && key == Key::Backspace
            && self.segment.is_empty()
        {
            if after == 0 {
                self.learn(&Event::Regret(kind));
            } else {
                self.accepted = Some((kind, after - 1));
            }
        }
        // ⌥Space (the setting): commit, then a full-width space (docs/moonshot.md §4).
        if key == Key::Space && self.option_space && mods.alt && !mods.ctrl && !mods.meta {
            return self.commit("\u{3000}".into(), true);
        }
        // ⌥ + a symbol or digit key (the same setting): commit, then the key's JIS legend in
        // full-width, as a Japanese IME types it (⇧⌥1 → ！, ⌥, → 、); in Mode A ⌥[ ⌥] are the
        // double brackets 『』. Letters stay shortcuts.
        if self.option_space
            && mods.alt
            && !mods.ctrl
            && !mods.meta
            && key.letter().is_none()
            && key != Key::Backquote
            && let Some(Out::Char(c)) = layout::output(Layout::Escaped, key, mods.shift)
            && zenkaku(c) != c
        {
            let typed = match c {
                '[' if self.mode.is_korean() => '『',
                ']' if self.mode.is_korean() => '』',
                _ => zenkaku(c),
            };
            // Staged, not committed (docs/moonshot.md §4), so 半角/全角 can still change it.
            self.conversion = None;
            self.segment.push(Token::Text(typed));
            return self.unchanged(true);
        }
        // 半角/全角 (docs/moonshot.md §4.1): what's staged, as it shows, to full-width; with Shift
        // to half-width. With nothing staged, the app's text (the host applies it).
        if key == Key::Backquote && !mods.is_shortcut() {
            let width = if mods.shift { Width::Half } else { Width::Full };
            if self.segment.is_empty() {
                return Output { consumed: true, host_edit: Some(HostEdit::Width(width)), ..Output::default() };
            }
            let shown = self.preedit();
            let converted = crate::width::convert(&shown, width);
            if converted != shown {
                self.conversion = None;
                self.segment = converted.chars().map(Token::Text).collect();
            }
            return self.unchanged(true);
        }
        if mods.is_shortcut() {
            return self.commit(String::new(), false);
        }
        // F6 hiragana, F7 katakana: the staged keys read as kana, until the next commit in Mode A
        // (docs/moonshot.md §4). Taken even with nothing staged: the next segment is kana.
        if matches!(key, Key::F6 | Key::F7) && !mods.shift {
            return self.to_kana(if key == Key::F6 { Script::Hiragana } else { Script::Katakana });
        }
        // F8 (and Shift+F8, the same since the hanja list is gone): kanji conversions on top, until
        // the next commit (docs/moonshot.md §4). Taken with nothing staged too: the next segment
        // converts.
        if key == Key::F8 {
            return self.to_kanji();
        }
        // With nothing unconfirmed, the other function keys are the app's: spec §6.
        if matches!(key, Key::F6 | Key::F7 | Key::F8 | Key::F9 | Key::F10) && self.segment.is_empty() {
            self.conversion = None;
            return self.unchanged(false);
        }
        // F9 (docs/moonshot.md §4): the Korean reading forced, as 変換 / かな; Shift+F9 bare.
        if key == Key::F9 {
            return self.kana_key_with(if mods.shift { Script::KoreanBare } else { Script::Korean });
        }
        // Space (docs/moonshot.md §4): a space, staged. The words around it stay unconfirmed and
        // are read together; with nothing staged it's the app's. Shift+Space commits.
        if key == Key::Space && !mods.shift {
            if self.segment.is_empty() {
                return self.unchanged(false);
            }
            self.conversion = None;
            self.segment.push(Token::Typed(' ', ' ', Some('\u{3000}')));
            return self.unchanged(true);
        }
        if key != Key::F10 {
            self.conversion = None;
        }
        match key {
            // Shift+F6, Shift+F7: nothing (reserved).
            Key::F6 | Key::F7 => self.unchanged(true),
            Key::F10 => {
                self.to_english();
                self.unchanged(true)
            }
            // Shift+Space commits, then a space; kana a full-width one, as Japanese IMEs do.
            Key::Space if self.script.is_kana() => self.commit("\u{3000}".into(), true),
            Key::Space => self.commit(" ".into(), true),
            Key::Enter if self.segment.is_empty() => self.unchanged(false),
            Key::Enter => {
                // Kana confirms with Enter, without a new line.
                let commit_only = mods.shift || self.enter_commits || self.script.is_kana();
                self.commit(String::new(), commit_only)
            }
            Key::Backspace if self.segment.is_empty() => self.unchanged(false),
            Key::Backspace => {
                // A key comes off whole: an action's parts (the ㅏ of 다, the ゜ of 로゜) go with the
                // token that carries its key, as the kana reading counts keys (docs/moonshot.md §9).
                while let Some(Token::Part(_) | Token::Break { raw: None, .. }) = self.segment.pop() {}
                self.unchanged(true)
            }
            _ => match layout::output_with(self.mode, self.symbols, Layout::JisKorean, key, mods.shift) {
                Some(Out::Jamo(jamo)) => {
                    let kana = self.mode.kana_scheme().kana(key, mods.shift);
                    let raw = key.letter().map(|c| if mods.shift { c.to_ascii_uppercase() } else { c });
                    // A jamo on a key right of 0 (docs/moonshot.md §9) is that key's stock JIS character.
                    let raw = raw.or(key.digit()).or_else(|| {
                        layout::output_with(self.mode, self.symbols, Layout::Escaped, key, mods.shift).map(Out::char)
                    });
                    // Shift on a vowel key (all of them, O and P too, docs/moonshot.md §9): the vowel
                    // starts its own syllable on ㅇ (spec §4.2).
                    let ieung = mods.shift
                        && hangul::is_medial(jamo)
                        && layout::output_with(self.mode, self.symbols, Layout::JisKorean, key, false) == Some(Out::Jamo(jamo));
                    self.segment.push(if ieung { Token::IeungVowel { jamo, raw, kana } } else { Token::Jamo { jamo, raw, kana } });
                    self.unchanged(true)
                }
                Some(Out::Keys(action)) => {
                    // An action (docs/moonshot.md §9): its first step carries the key, like any
                    // other key's token; the rest are parts of it.
                    let kana = self.mode.kana_scheme().kana(key, mods.shift);
                    let raw = key.letter().map(|c| if mods.shift { c.to_ascii_uppercase() } else { c });
                    let raw = raw.or_else(|| layout::output_with(self.mode, self.symbols, Layout::Escaped, key, mods.shift).map(Out::char));
                    for (i, step) in action.steps.iter().enumerate() {
                        let (raw, kana) = if i == 0 { (raw, kana) } else { (None, None) };
                        self.segment.push(match (*step, i) {
                            (Step::Jamo(jamo), 0) => Token::Jamo { jamo, raw, kana },
                            (Step::Jamo(jamo), _) => Token::Part(jamo),
                            (Step::Ieung(jamo), _) => Token::IeungVowel { jamo, raw, kana },
                            (Step::Break, _) => Token::Break { raw, kana },
                            (Step::Blank, _) => Token::Blank { raw, kana },
                        });
                    }
                    self.unchanged(true)
                }
                Some(Out::Char(c)) => {
                    let kana = self.mode.kana_scheme().kana(key, mods.shift);
                    // The English reading is the stock keyboard's character, so predictions never
                    // imply JIS on US ANSI (Shift+2 reads @ there, " on JIS).
                    let english = layout::output_with(self.mode, self.symbols, Layout::Escaped, key, mods.shift).map_or(c, Out::char);
                    self.segment.push(Token::Typed(c, english, kana));
                    self.unchanged(true)
                }
                None => self.commit(String::new(), false),
            },
        }
    }

    /// F10 (docs/moonshot.md §4): the whole staged text as its keys, half-width (the English
    /// reading as typed), staged; pressed again it cycles case: as typed → UPPER → Capitalized.
    /// Symbols stay as typed, with their half-width forms (spec §5.3). F9, F6 and F7 undo it.
    fn to_english(&mut self) {
        match &mut self.conversion {
            Some(conversion) => conversion.case = (conversion.case + 1) % 3,
            None => {
                if self.segment.is_empty() {
                    return;
                }
                let original = std::mem::take(&mut self.segment);
                self.conversion = Some(Conversion { start: 0, original, case: 0 });
            }
        }
        let Some(conversion) = &self.conversion else { return };
        let raw: String = conversion
            .original
            .iter()
            .filter_map(|t| match t {
                Token::Jamo { raw, .. } | Token::IeungVowel { raw, .. } | Token::Break { raw, .. } | Token::Blank { raw, .. } => *raw,
                Token::Part(_) => None,
                // In kana, a symbol key converts to its stock JIS character (ほ → -).
                Token::Typed(_, english, Some(_)) if self.script.is_kana() => Some(*english),
                Token::Text(c) | Token::Typed(c, ..) => Some(*c),
                Token::Bare => None,
            })
            .collect();
        let cased = match conversion.case {
            0 => raw,
            1 => raw.to_uppercase(),
            _ => {
                let mut chars = raw.chars();
                chars
                    .next()
                    .map(|first| first.to_uppercase().chain(chars.as_str().to_lowercase().chars()).collect())
                    .unwrap_or_default()
            }
        };
        let start = conversion.start;
        self.segment.truncate(start);
        self.segment.extend(cased.chars().map(|c| Token::Text(half_width(c))));
    }

    /// Turns the predictive layer on (`Some`) or off (`None`, the lite engine). Engines can share one.
    #[cfg(feature = "predict")]
    pub fn set_predictor(&mut self, predictor: Option<Arc<Predictor>>) {
        self.predictor = predictor;
    }

    /// Tunes one scenario of the predictions (spec §10.5): a multiplier on its learned probability.
    /// Sets a scenario's weight by hand; it then wins over the learned one.
    #[cfg(feature = "predict")]
    pub fn set_prediction_weight(&mut self, which: Weight, value: f64) {
        self.weights.set(which, value);
        self.manual[which as usize] = true;
    }

    /// Hands a scenario's weight back to learning (or 1.0 when learning is off).
    #[cfg(feature = "predict")]
    pub fn clear_prediction_weight(&mut self, which: Weight) {
        self.weights.set(which, 1.0);
        self.manual[which as usize] = false;
    }

    /// Turns learning from the user's choices on or off (spec §10.11). Off keeps what was learned
    /// but stops using and adding to it.
    #[cfg(feature = "predict")]
    pub fn set_learning(&mut self, on: bool) {
        self.learning = on;
        self.accepted = None;
    }

    /// Whether choices are recorded (spec §10.11): off in an app the user excluded from learning,
    /// where what was learned still applies. On by default.
    #[cfg(feature = "predict")]
    pub fn set_recording(&mut self, on: bool) {
        self.recording = on;
        self.accepted = None;
    }

    #[cfg(feature = "predict")]
    pub fn learner(&self) -> &Learner {
        &self.learner
    }

    /// Whether a predictor is loaded (the full engine with predictions on).
    #[cfg(feature = "predict")]
    pub fn predictions_available(&self) -> bool {
        self.predictor.is_some()
    }

    /// Learns from one example of what the user meant (spec §10.12, the training contract): types
    /// `keys` (or, without them, the cheapest keys for `intended`) into a scratch engine with this
    /// one's predictor and weights, and where `intended` is among the predictions, records that it
    /// was chosen, as learning does. Returns where it was listed (0 = on top), or None when it
    /// wasn't (or there's no predictor). Learns even with learning off: the user asked for it.
    #[cfg(feature = "predict")]
    pub fn learn_example(&mut self, keys: Option<&[crate::plan::Press]>, intended: &str) -> Option<usize> {
        let planned;
        let keys = match keys {
            Some(keys) => keys,
            None => {
                planned = crate::plan::keys_for_word(intended)?;
                &planned
            }
        };
        let mut scratch = Engine::new();
        scratch.predictor = Some(self.predictor.clone()?);
        scratch.weights = self.weights;
        scratch.manual = self.manual;
        scratch.ranker = self.ranker;
        scratch.learner = self.learner.clone();
        scratch.learning = true;
        for press in keys {
            scratch.key(press.key, if press.shift { Mods::SHIFTED } else { Mods::NONE });
        }
        let at = scratch.predictions().iter().position(|p| p.text == intended)?;
        scratch.commit_prediction(at);
        self.learner = scratch.learner;
        self.learned_changed = true;
        Some(at)
    }

    /// Replaces what was learned (from the host's storage, or a reset with the default).
    #[cfg(feature = "predict")]
    pub fn set_learner(&mut self, learner: Learner) {
        self.learner = learner;
        self.learned_changed = false;
    }

    /// Whether the learner changed since the last call (the host then stores it).
    #[cfg(feature = "predict")]
    pub fn take_learned_changed(&mut self) -> bool {
        std::mem::take(&mut self.learned_changed)
    }

    #[cfg(feature = "predict")]
    fn learn(&mut self, event: &Event) {
        if self.learning && self.recording {
            self.learner = self.learner.after(event);
            self.learned_changed = true;
        }
    }

    /// The weights the predictor uses: the user's where set by hand, else the learned ones while
    /// learning is on, else 1.0.
    #[cfg(feature = "predict")]
    pub fn effective_weights(&self) -> Weights {
        let mut weights = self.weights;
        for which in Weight::ALL {
            if !self.manual[which as usize] && self.learning {
                weights.set(which, self.learner.weight(which));
            }
        }
        weights
    }

    /// Chooses how the implied-ㅇ alternatives are ranked (spec §10.7).
    #[cfg(feature = "predict")]
    pub fn set_ranker(&mut self, ranker: Ranker) {
        self.ranker = ranker;
    }

    #[cfg(feature = "predict")]
    pub fn prediction_weights(&self) -> Weights {
        self.weights
    }

    /// The readings of the staged text (spec §10), with its pieces between punctuation when there's
    /// any (Research(이하 is Research, then 이하).
    #[cfg(feature = "predict")]
    pub fn readings(&self) -> Readings {
        let mut readings = self.readings_of(&self.segment);
        let mut start = 0;
        let mut pieces = Vec::new();
        for (i, token) in self.segment.iter().enumerate() {
            if let Token::Typed(c, ..) = *token
                && is_separator(c)
            {
                pieces.push(Piece { readings: without_kana(self.readings_of(&self.segment[start..i])), separator: c.to_string() });
                start = i + 1;
            }
        }
        if !pieces.is_empty() {
            pieces.push(Piece { readings: without_kana(self.readings_of(&self.segment[start..])), separator: String::new() });
            readings.pieces = pieces;
        }
        readings
    }

    #[cfg(feature = "predict")]
    fn readings_of(&self, segment: &[Token]) -> Readings {
        let korean = compose(segment);
        // Where each key's tokens start: an action's parts (docs/moonshot.md §9) belong to its key,
        // and every split and case change below falls between keys.
        let starts: Vec<usize> = (0..segment.len()).filter(|&k| !is_part(&segment[k])).collect();
        let at = |i: usize| starts.get(i).copied().unwrap_or(segment.len());
        let count = starts.len();
        // Every way to read the keys as an English start plus a Korean ending (report에), ending
        // short: the predictor keeps the ones whose ending is a particle.
        let first = count.saturating_sub(MAX_ENDING_KEYS).max(1);
        // Not after a number-row jamo (ㅘ as 0 in Dynamics와의): that key is the ending's, and an
        // English start's digits are typed as digits (APR1400은).
        let jamo_digit = |t: &Token| matches!(*t, Token::Jamo { raw: Some(c), .. } | Token::IeungVowel { raw: Some(c), .. } if c.is_ascii_digit());
        let mut splits: Vec<Split> = (first..count)
            .map(at)
            .filter(|&k| !segment[..k].iter().any(jamo_digit))
            .map(|k| Split {
                english: typed_of(&segment[..k]),
                korean: compose(&segment[k..]),
                kana: kana_of(&segment[k..]),
                korean_start: compose(&segment[..k]),
                kana_start: kana_of(&segment[..k]),
            })
            .collect();
        // The keys one by one, to find where the case changes.
        let keys: Vec<Option<char>> = starts.iter().map(|&k| typed_key(&segment[k])).collect();
        let upper = |i: usize| keys[i].is_some_and(|c| c.is_ascii_uppercase());
        let lower = |i: usize| keys[i].is_some_and(|c| c.is_ascii_lowercase());
        // A digit typed as a digit, not a number-row jamo (ㅢ as 9).
        let digit = |i: usize| matches!(segment[at(i)], Token::Text(c) | Token::Typed(c, ..) if c.is_ascii_digit());
        let capital_run = |from: usize| (from..count).take_while(|&i| upper(i) || digit(i)).count();
        // An acronym start then lowercase keys (LG에너지솔루션은, KB국민카드의): also split where the
        // case changes, however far from the end, so the rest can be read as Korean.
        let acronym = capital_run(0);
        if acronym >= 2 && acronym < first && lower(acronym) {
            splits.push(Split {
                english: typed_of(&segment[..at(acronym)]),
                korean: compose(&segment[at(acronym)..]),
                kana: kana_of(&segment[at(acronym)..]),
                korean_start: compose(&segment[..at(acronym)]),
                kana_start: kana_of(&segment[..at(acronym)]),
            });
        }
        // A Korean start then an acronym (한전KPS, 에너지ICT): the first place lowercase keys are
        // followed by two capitals.
        let head = (1..count.saturating_sub(1))
            .find(|&i| (lower(i - 1) || keys[i - 1].is_some_and(|c| c.is_ascii_digit())) && upper(i) && upper(i + 1))
            .map(|i| {
                let end = i + capital_run(i);
                Head {
                    korean: compose(&segment[..at(i)]),
                    keys: typed_of(&segment[..at(i)]),
                    acronym: typed_of(&segment[at(i)..at(end)]),
                    rest: compose(&segment[at(end)..]),
                }
            });
        Readings {
            english: typed_of(segment),
            two_set: compose_with(&two_set_tokens(segment), Options { standard: true, ..Options::default() }),
            keep_finals: compose_with(segment, Options { keep_finals: true, ..Options::default() }),
            kept: kept_variants(segment),
            implied_l: implied_l_variants(segment),
            bare: compose_with(segment, Options { no_implied: true, ..Options::default() }),
            implied: composer_for(segment).implied_count(),
            splits,
            korean,
            pieces: Vec::new(),
            head,
            kana: kana_of(segment),
            // A key right of 0 that typed Korean (docs/moonshot.md §9), before the last key; and
            // whether the last key is one.
            korean_only: segment[..at(count.saturating_sub(1))].iter().filter(|t| korean_only_key(t)).count(),
            korean_only_last: count > 0 && korean_only_key(&segment[at(count - 1)]),
            korean_only_last_consonant: count > 0
                && korean_only_key(&segment[at(count - 1)])
                && matches!(segment[at(count - 1)], Token::Jamo { jamo, .. } if !hangul::is_medial(jamo)),
            // A symbol key (not a digit) typed before the last key: punctuation in Korean, kana here.
            kana_only: segment
                .iter()
                .take(at(count.saturating_sub(1)))
                .filter(|t| match **t {
                    Token::Typed(c, _, Some(_)) => !c.is_ascii_digit(),
                    // The keys right of 0 type Korean in JIS Moonshot (docs/moonshot.md §9): they're
                    // still kana-only keys, by the key (its stock character isn't a letter or digit).
                    Token::Jamo { raw: Some(c), kana: Some(_), .. }
                    | Token::IeungVowel { raw: Some(c), kana: Some(_), .. }
                    | Token::Break { raw: Some(c), kana: Some(_) }
                    | Token::Blank { raw: Some(c), kana: Some(_) } => !c.is_alphanumeric(),
                    _ => false,
                })
                .count(),
        }
    }

    /// The ranked predictions for the staged text; empty without a predictor (the lite engine).
    #[cfg(feature = "predict")]
    pub fn predictions(&self) -> Vec<Prediction> {
        match &self.predictor {
            Some(p) if !self.segment.is_empty() && self.script == Script::Auto && self.mode.is_korean() => {
                let readings = self.readings();
                let predicted = p.predict(&readings, self.prediction_context());
                with_symbol_variants(crate::brands::with_suggestions(predicted, &readings, |k| p.is_ending(k)))
            }
            // JIS Kana after F8: the conversions, then the kana as typed (docs/moonshot.md §4).
            Some(p) if !self.segment.is_empty() && self.kanji && self.mode.is_kana() => {
                let kanji = p.kanji_of(&kana_of(&self.segment));
                if kanji.is_empty() {
                    return Vec::new();
                }
                let kind = if self.script == Script::Katakana { Kind::Katakana } else { Kind::Kana };
                let n = kanji.len();
                let mut out: Vec<Prediction> =
                    kanji.into_iter().enumerate().map(|(i, text)| Prediction { text, kind: Kind::Kanji, score: (n - i) as f64 }).collect();
                out.push(Prediction { text: self.preedit(), kind, score: 0.0 });
                out
            }
            _ => Vec::new(),
        }
    }

    #[cfg(feature = "predict")]
    fn prediction_context(&self) -> Context {
        Context { previous: self.previous, command: false, weights: self.effective_weights(), ranker: self.ranker, force_kanji: self.kanji }
    }

    /// The staged text's distinct readings with the learned ranker's features (spec §10.13), in a
    /// table of 2^`bits`: what the training data records. Empty without a predictor.
    #[cfg(all(feature = "predict", not(feature = "legacy-predict")))]
    pub fn prediction_features(&self, bits: u32) -> Vec<crate::learned::Featured> {
        match &self.predictor {
            Some(p) if !self.segment.is_empty() => {
                crate::learned::featurize(p, &self.readings(), self.prediction_context(), bits)
            }
            _ => Vec::new(),
        }
    }

    /// Commits prediction `index` (from [`Engine::predictions`]) instead of the Korean reading.
    #[cfg(feature = "predict")]
    pub fn commit_prediction(&mut self, index: usize) -> Output {
        let shown = self.predictions();
        let Some(chosen) = shown.get(index).cloned() else { return self.unchanged(true) };
        let kinds = shown.iter().map(|p| learn::scenario(p.kind)).collect();
        self.learn(&Event::Chose { shown: kinds, chosen: index });
        self.accepted = (index == 0 && self.learning).then(|| (learn::scenario(chosen.kind), 0));
        self.previous = Some(Language::of(chosen.kind));
        self.take_segment();
        Output { commit: chosen.text, consumed: true, ..Output::default() }
    }

    /// A key that commits (Space, Enter, a shortcut, …) while prediction `index` is what shows:
    /// that prediction stands in for the unconfirmed text, then the key does what it does to
    /// it (spec §10): Space adds a space, Enter passes on unless it only commits (Shift+Enter or
    /// the setting), and so on. Learns the choice as [`Engine::commit_prediction`] does.
    #[cfg(feature = "predict")]
    pub fn key_after_prediction(&mut self, index: usize, key: Key, mods: Mods) -> Output {
        let shown = self.predictions();
        let Some(chosen) = shown.get(index).cloned() else { return self.key(key, mods) };
        let kinds = shown.iter().map(|p| learn::scenario(p.kind)).collect();
        self.learn(&Event::Chose { shown: kinds, chosen: index });
        self.previous = Some(Language::of(chosen.kind));
        self.conversion = None;
        self.segment = chosen.text.chars().map(Token::Text).collect();
        // The choice is learned above: the key's own commit learns nothing more.
        let learning = std::mem::replace(&mut self.learning, false);
        let out = self.key(key, mods);
        self.learning = learning;
        // After the key, which clears what was accepted before it and sets the context. What the
        // key typed after the word (Space's space, a passed-on Enter) goes before regret can.
        let after = out.commit.chars().count().saturating_sub(chosen.text.chars().count()) + !out.consumed as usize;
        self.accepted = (index == 0 && self.learning).then(|| (learn::scenario(chosen.kind), after));
        self.previous = Some(Language::of(chosen.kind));
        out
    }

    /// Whether `key` commits the unconfirmed text as it stands: a shortcut, Space, Enter, Esc, an
    /// arrow, a key outside the mode's map; or takes it as it shows and keeps it staged (⌥ + a
    /// symbol, 半角/全角: docs/moonshot.md §4, §4.1). While a prediction shows instead of the Korean reading,
    /// hosts send these through [`Engine::key_after_prediction`] so what's shown is what commits;
    /// other keys (jamo, symbols, Backspace, F6–F10) go to [`Engine::key`]. Keeping the list here
    /// keeps the shells from deciding what a key does.
    pub fn commits(&self, key: Key, mods: Mods) -> bool {
        if matches!(key, Key::Modifier) {
            return false;
        }
        if mods.is_shortcut() {
            return true;
        }
        match key {
            // 半角/全角 converts the reading on screen (docs/moonshot.md §4.1).
            Key::Backquote => true,
            // Space stages a space (docs/moonshot.md §4): what shows stays as typed. Shift+Space
            // commits it.
            Key::Space if !mods.shift => false,
            Key::Space | Key::Enter | Key::Escape | Key::ArrowUp | Key::ArrowDown | Key::ArrowLeft | Key::ArrowRight | Key::Other => true,
            Key::Backspace | Key::F6 | Key::F7 | Key::F8 | Key::F9 | Key::F10 => false,
            _ => layout::output_with(self.mode, self.symbols, Layout::JisKorean, key, mods.shift).is_none(),
        }
    }

    /// Commits prediction `index` because the host must (focus loss, a click elsewhere): like
    /// [`Engine::flush`], it isn't a choice, so nothing is learned.
    #[cfg(feature = "predict")]
    pub fn flush_prediction(&mut self, index: usize) -> Output {
        let Some(chosen) = self.predictions().into_iter().nth(index) else { return self.flush() };
        self.previous = Some(Language::of(chosen.kind));
        self.take_segment();
        Output { commit: chosen.text, consumed: true, ..Output::default() }
    }

    fn take_segment(&mut self) -> String {
        // JIS English stages nothing but a press-and-hold letter: what it shows is what commits,
        // before anything typed after switching to a Korean mode with the list still open.
        #[cfg(feature = "english")]
        let held = self.held.take().map(|held| held.shown().to_string()).unwrap_or_default();
        #[cfg(not(feature = "english"))]
        let held = String::new();
        self.conversion = None;
        let segment = std::mem::take(&mut self.segment);
        let text = self.shown(&segment);
        // F8 lasts until the commit. Mode A's next segment starts in Auto again; JIS Kana stays in
        // its script.
        self.kanji = false;
        if self.mode.is_korean() {
            self.script = Script::Auto;
        }
        held + &text
    }

    /// A key in an English mode (spec §3.6.2, §3.6.3): every key goes to the app, except ⌥ + a
    /// letter with variants, which opens their list, and the keys that work the list while it's open.
    #[cfg(feature = "english")]
    fn english_key(&mut self, key: Key, mods: Mods) -> Output {
        // Whatever a Korean mode left unconfirmed commits before the key, as on any switch.
        let mut commit = if self.held.is_none() { self.take_segment() } else { String::new() };
        // ⌥ alone (Shift allowed): ⌘ and Ctrl make it a shortcut.
        let option = mods.alt && !mods.ctrl && !mods.meta;
        if let Some(held) = self.held.as_mut() {
            let count = held.variants.len();
            if option && key == held.key {
                // The same ⌥ + letter again: the next variant. Its auto-repeat does nothing.
                if !mods.repeat {
                    held.selected = Some(held.selected.map_or(0, |i| (i + 1) % count));
                }
                return self.held_output();
            }
            if !mods.is_shortcut() {
                // 1–9 and 0 pick the first ten; a digit past the list is typed after the letter.
                let pick = key.digit().and_then(|d| d.to_digit(10)).map(|d| if d == 0 { 10 } else { d as usize });
                if let Some(n) = pick.filter(|&n| n <= count) {
                    held.selected = Some(n - 1);
                    return self.commit(String::new(), true);
                }
                match key {
                    Key::ArrowRight | Key::ArrowDown => {
                        held.selected = Some(held.selected.map_or(0, |i| (i + 1) % count));
                        return self.held_output();
                    }
                    Key::ArrowLeft | Key::ArrowUp => {
                        held.selected = Some(held.selected.map_or(count - 1, |i| (i + count - 1) % count));
                        return self.held_output();
                    }
                    // The choice, or the letter with none.
                    Key::Enter => return self.commit(String::new(), true),
                    // Nothing was typed yet: both leave nothing.
                    Key::Escape | Key::Backspace => {
                        self.held = None;
                        return Output { consumed: true, ..Output::default() };
                    }
                    _ => {}
                }
            }
            // Any other key keeps what's shown, then goes on (another ⌥ + letter opens its list).
            commit = self.take_segment();
        }
        // 半角/全角 (docs/moonshot.md §4.1): nothing is staged, so the app's text converts.
        if key == Key::Backquote && !mods.is_shortcut() {
            let width = if mods.shift { Width::Half } else { Width::Full };
            return Output { commit, consumed: true, host_edit: Some(HostEdit::Width(width)), ..Output::default() };
        }
        // ⌥Space (the setting): `_` (docs/moonshot.md §4).
        if option && key == Key::Space && self.option_space {
            commit.push('_');
            return Output { commit, consumed: true, ..Output::default() };
        }
        // ⌥[ ⌥] (the setting): the double angle quotes « », wherever the mode puts [ and ].
        if option
            && self.option_space
            && !mods.shift
            && let Some(Out::Char(c @ ('[' | ']'))) = layout::output(Layout::Escaped, self.mode.qwerty_key(key), false)
        {
            commit.push(if c == '[' { '«' } else { '»' });
            return Output { commit, consumed: true, ..Output::default() };
        }
        let upper = mods.shift || mods.caps;
        if option
            && !mods.repeat
            && let Some(letter) = self.mode.letter(key)
            && let Some(variants) = crate::hold::variants(letter, upper)
        {
            let letter = if upper { letter.to_ascii_uppercase() } else { letter };
            self.held = Some(crate::hold::Held { key, letter, variants, selected: None });
            return Output { commit, ..self.held_output() };
        }
        Output { commit, consumed: false, ..Output::default() }
    }

    /// The open variants list: the shown character as the preedit, the variants as the list
    /// (none selected is `selected` = the count).
    #[cfg(feature = "english")]
    fn held_output(&self) -> Output {
        let Some(held) = &self.held else { return Output { consumed: true, ..Output::default() } };
        let items = held.variants.iter().map(|v| Candidate { text: v.to_string(), comment: String::new() }).collect();
        Output {
            preedit: held.shown().to_string(),
            consumed: true,
            candidates: Some(Candidates { items, selected: held.selected.unwrap_or(held.variants.len()) }),
            ..Output::default()
        }
    }

    /// Commits the segment followed by `suffix`, for a key that commits: the Korean reading was
    /// the user's choice (spec §10.11 Accept, when it was the top prediction with others shown),
    /// and the context for the next word (§10.3).
    fn commit(&mut self, suffix: String, consumed: bool) -> Output {
        #[cfg(feature = "predict")]
        if !self.segment.is_empty() && self.script == Script::Auto {
            let shown = self.predictions();
            if let Some(top) = shown.first()
                && top.kind == crate::predict::Kind::Korean
                && shown.len() > 1
            {
                let kinds = shown.iter().map(|p| learn::scenario(p.kind)).collect();
                self.learn(&Event::Chose { shown: kinds, chosen: 0 });
                let after = suffix.chars().count() + !consumed as usize;
                self.accepted = self.learning.then_some((Weight::Korean, after));
            }
            self.previous = Some(Language::Korean);
        }
        self.commit_quietly(suffix, consumed)
    }

    /// Commits the segment followed by `suffix`, learning nothing.
    fn commit_quietly(&mut self, suffix: String, consumed: bool) -> Output {
        let mut commit = self.take_segment();
        commit.push_str(&suffix);
        Output { commit, consumed, ..Output::default() }
    }

    fn unchanged(&self, consumed: bool) -> Output {
        Output { preedit: self.preedit(), consumed, english: self.english(), ..Output::default() }
    }
}

/// A piece between punctuation is read as Korean or English only: the kana reading is of the whole
/// staged text, where the symbol keys are kana (docs/moonshot.md §5).
#[cfg(feature = "predict")]
fn without_kana(mut readings: Readings) -> Readings {
    readings.kana.clear();
    readings
}

/// Each reading that ends in a symbol, followed by the same reading with that symbol's variants
/// (docs/moonshot.md §5.2: 、 and ， after a reading ending in a comma, … after one ending in a
/// period, 「 『 after one ending in [), each just below it and of its kind. Duplicates go.
#[cfg(feature = "predict")]
fn with_symbol_variants(predictions: Vec<Prediction>) -> Vec<Prediction> {
    let mut out: Vec<Prediction> = Vec::with_capacity(predictions.len());
    let mut seen = std::collections::HashSet::new();
    for p in predictions {
        // Not the standard 2-set reading's: its symbols are the stock keys' (< for Shift+,).
        let variants = if p.kind == Kind::TwoSet { Vec::new() } else { crate::punctuation::variants(&p.text) };
        let (kind, score) = (p.kind, p.score);
        if seen.insert(p.text.clone()) {
            out.push(p);
        }
        for (i, text) in variants.into_iter().enumerate() {
            if seen.insert(text.clone()) {
                out.push(Prediction { text, kind, score: score - 0.001 * (i + 1) as f64 });
            }
        }
    }
    out
}

/// Staged tokens read as kana (docs/moonshot.md §3.1, §5): each key's JIS kana, composed; text
/// from a conversion or a pick as it is.
fn kana_of(segment: &[Token]) -> String {
    crate::kana::compose(segment.iter().filter_map(|t| match *t {
        Token::Jamo { kana, .. }
        | Token::IeungVowel { kana, .. }
        | Token::Break { kana, .. }
        | Token::Blank { kana, .. }
        | Token::Typed(_, _, kana) => kana,
        Token::Text(c) => Some(c),
        Token::Bare | Token::Part(_) => None,
    }))
}

/// Keystrokes read as English on the mode's stock keyboard (see [`Engine::english`]).
fn english_of(segment: &[Token]) -> String {
    segment
        .iter()
        .filter_map(|t| match *t {
            Token::Jamo { jamo, raw, .. } | Token::IeungVowel { jamo, raw, .. } => Some(raw.unwrap_or(jamo)),
            Token::Break { raw, .. } | Token::Blank { raw, .. } => raw,
            Token::Text(c) => Some(c),
            Token::Typed(_, english, _) => Some(english),
            Token::Bare | Token::Part(_) => None,
        })
        .collect()
}

/// The staged keys composed with one final kept before the vowel after it (산이 for 사니, 어린이
/// for 어리니), one variant per place the composition moves a final: the vowel there starts its
/// own syllable on ㅇ, as Shift+vowel types it (spec §4.2, §10.3).
#[cfg(feature = "predict")]
fn kept_variants(segment: &[Token]) -> Vec<String> {
    let moved = compose(segment);
    let mut out: Vec<String> = Vec::new();
    for (k, token) in segment.iter().enumerate() {
        let Token::Jamo { jamo, raw, kana } = *token else { continue };
        if k == 0 || !hangul::is_medial(jamo) {
            continue;
        }
        let mut variant = segment.to_vec();
        variant[k] = Token::IeungVowel { jamo, raw, kana };
        let kept = compose(&variant);
        if kept != moved && kept.chars().count() == moved.chars().count() && !out.contains(&kept) {
            out.push(kept);
        }
    }
    out
}

/// The staged keys composed with an implied ㄹ after a ㄴ final (전력 for 저녁 typed lazily: ㄴ
/// then ㄹ sounds ㄹㄹ, so the ㄹ is easy to leave out), one variant per place a ㄴ final moved
/// onto a vowel (spec §10.5).
#[cfg(feature = "predict")]
fn implied_l_variants(segment: &[Token]) -> Vec<String> {
    let moved = compose(segment);
    let mut out: Vec<String> = Vec::new();
    for (k, token) in segment.iter().enumerate() {
        let Token::Jamo { jamo, .. } = *token else { continue };
        let after_n = k > 0 && matches!(segment[k - 1], Token::Jamo { jamo: 'ㄴ', .. });
        if !after_n || !hangul::is_medial(jamo) {
            continue;
        }
        let mut variant = segment.to_vec();
        variant.insert(k, Token::Jamo { jamo: 'ㄹ', raw: None, kana: None });
        let text = compose(&variant);
        if text != moved && text.chars().count() == moved.chars().count() && !out.contains(&text) {
            out.push(text);
        }
    }
    out
}

/// The keys read as English for predictions: letters with their case, and non-letter keys as they
/// typed (a ; typed is ;, where F10 would give the stock JIS key's character).
#[cfg(feature = "predict")]
fn typed_of(segment: &[Token]) -> String {
    segment.iter().filter_map(typed_key).collect()
}

/// Whether a token is a key right of 0 that typed Korean (docs/moonshot.md §9: its stock character
/// isn't a letter or digit).
#[cfg(feature = "predict")]
fn korean_only_key(token: &Token) -> bool {
    match *token {
        Token::Jamo { raw: Some(c), .. } | Token::IeungVowel { raw: Some(c), .. } | Token::Break { raw: Some(c), .. } => !c.is_alphanumeric(),
        _ => false,
    }
}

/// Whether a token is part of the key before it (the ㅏ of 다, the ゜ of 로゜), docs/moonshot.md §9.
fn is_part(token: &Token) -> bool {
    matches!(token, Token::Part(_) | Token::Break { raw: None, .. })
}

/// One token's key as [`typed_of`] reads it; none for a bare-vowel marker.
#[cfg(feature = "predict")]
fn typed_key(token: &Token) -> Option<char> {
    match *token {
        Token::Jamo { jamo, raw, .. } | Token::IeungVowel { jamo, raw, .. } => Some(raw.unwrap_or(jamo)),
        Token::Break { raw, .. } | Token::Blank { raw, .. } => raw,
        Token::Text(c) | Token::Typed(c, ..) => Some(c),
        Token::Bare | Token::Part(_) => None,
    }
}

/// Punctuation that separates the pieces of a staged word (spec §10.10): anything typed that isn't
/// a letter or digit.
#[cfg(feature = "predict")]
fn is_separator(c: char) -> bool {
    !c.is_alphanumeric()
}

/// The longest Korean ending considered when splitting into English + Korean (에서는 is 6 keys).
#[cfg(feature = "predict")]
const MAX_ENDING_KEYS: usize = 8;

fn composer_for(segment: &[Token]) -> Composer {
    composer_with(segment, Options::default())
}

fn composer_with(segment: &[Token], options: Options) -> Composer {
    let mut composer = Composer::with(options);
    // Breaks with nothing else staged (the [ key pressed first) show as ㆍ, so they're on screen
    // and a reading of their own (docs/moonshot.md §9.1).
    if !segment.is_empty() && segment.iter().all(|t| matches!(t, Token::Break { .. })) {
        for _ in segment {
            composer.text(crate::layout::BREAK_MARK);
        }
        return composer;
    }
    for token in segment {
        match *token {
            Token::Jamo { jamo, .. } => composer.jamo(jamo),
            Token::Part(jamo) => composer.part(jamo),
            Token::Break { .. } => composer.syllable_break(),
            Token::Blank { .. } => {}
            Token::Text(c) | Token::Typed(c, ..) => composer.text(c),
            Token::Bare => composer.bare(),
            Token::IeungVowel { jamo, .. } => composer.ieung_vowel(jamo),
        }
    }
    composer
}

fn compose(segment: &[Token]) -> String {
    composer_for(segment).finish()
}

fn compose_with(segment: &[Token], options: Options) -> String {
    composer_with(segment, options).finish()
}

/// The segment as plain 2-set on stock JIS would have it: number-row keys type digits, symbol keys
/// their stock JIS characters (composing then happens with `Options::standard`).
#[cfg(feature = "predict")]
fn two_set_tokens(segment: &[Token]) -> Vec<Token> {
    let mut out = Vec::new();
    // A key right of 0 (docs/moonshot.md §9) is its stock JIS character on plain 2-set: its first
    // token becomes that character and the rest of its action goes.
    let mut symbol_key = false;
    for t in segment {
        let keyed_symbol = |raw: Option<char>| raw.is_some_and(|c| !c.is_alphanumeric());
        match *t {
            Token::Jamo { raw: Some(d), .. } if d.is_ascii_digit() => out.push(Token::Text(d)),
            Token::Jamo { raw, .. } | Token::IeungVowel { raw, .. } | Token::Break { raw, .. } | Token::Blank { raw, .. }
                if keyed_symbol(raw) =>
            {
                symbol_key = true;
                out.push(Token::Text(raw.unwrap_or_default()));
                continue;
            }
            Token::Part(_) | Token::Break { raw: None, .. } if symbol_key => continue,
            Token::Typed(_, english, _) => out.push(Token::Text(english)),
            ref other => out.push(other.clone()),
        }
        symbol_key = false;
    }
    out
}

/// Characters with a full-width and a half-width form beyond printable ASCII (Unicode's
/// Halfwidth and Fullwidth Forms block): (half, full).
const WIDTH_PAIRS: &[(char, char)] = &[
    ('¢', '￠'), ('£', '￡'), ('¬', '￢'), ('¯', '￣'), ('¦', '￤'), ('¥', '￥'), ('₩', '￦'),
    ('｡', '。'), ('｢', '「'), ('｣', '」'), ('､', '、'), ('･', '・'),
];

/// The full-width form of a character, or the character itself when it has none (spec §5.3).
pub fn full_width(c: char) -> char {
    match c {
        '!'..='~' => char::from_u32(c as u32 + 0xFEE0).unwrap_or(c),
        ' ' => '\u{3000}',
        _ => WIDTH_PAIRS.iter().find(|&&(half, _)| half == c).map_or(c, |&(_, full)| full),
    }
}

/// What ⌥ + a key types (docs/moonshot.md §4): Japanese punctuation for , . / [ ] (、。・「」, as a
/// Japanese IME types them), else the full-width form.
fn zenkaku(c: char) -> char {
    match c {
        ',' => '、',
        '.' => '。',
        '/' => '・',
        '[' => '「',
        ']' => '」',
        _ => full_width(c),
    }
}

/// The half-width form of a character, or the character itself when it has none (spec §5.3).
pub fn half_width(c: char) -> char {
    match c {
        '\u{FF01}'..='\u{FF5E}' => char::from_u32(c as u32 - 0xFEE0).unwrap_or(c),
        '\u{3000}' => ' ',
        _ => WIDTH_PAIRS.iter().find(|&&(_, full)| full == c).map_or(c, |&(half, _)| half),
    }
}
