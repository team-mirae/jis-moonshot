# Typing speed as a signal of lazy typing

Research note, 2026-11-02. No keystroke timing data exists in the repo; the estimate below uses the
engine's own typing of `corpora/heldout-ko.txt` (held-out chat words with counts) with a throwaway
program outside the repo (method below), and `cargo run --release --example sxs -- --no-grid` and
`cargo test --release --test corpora -- --nocapture` as they stand. Not used by the engine.

## Question

Is keystrokes per second (typing speed, inter-key intervals) a useful sign of how "lazy" the user is
typing (implied ㅇ, Shift+vowel, the dedicated keys, skipping the double press so 말이 is typed with
마리's keys, spelling by sound as in 안코 for 않고, trusting the predictions), and can it be paired
with a predictor to rank readings better?

## What's known

- **Faster typists are more careful, not sloppier.** In 136 million keystrokes from 168,000 people
  ([Dhakal et al., CHI 2018](https://www.pokristensson.com/pubs/DhakalEtAlCHI2018.pdf)), words per
  minute correlates *negatively* with uncorrected errors (r = −0.21) and with corrections
  (r = −0.36). Fast typists (about 120 ms between keys) leave 0.71% of errors uncorrected and slow
  ones (about 480 ms) 1.78%. Across people, speed measures skill. It doesn't measure carelessness.
- **Within a person, speed hardly predicts errors either.** Touch typists are no faster before an
  error. Their timing is only more *variable*, and the error itself and the key after it are slower
  ([Kalfaoğlu & Stafford, QJEP 2014](https://eprints.whiterose.ac.uk/85855/)). Ten-finger typists
  slow down a little before errors they notice; hunt-and-peck typists don't
  ([Dahm & Rieger, Open Psychology 2022](https://doaj.org/article/4ee1ef22dbf44710806539380c5aea30)).
  Both effects are small, so they only show up when averaged over many keys.
- **Timing mostly reflects how predictable the next key is.** Familiar words start sooner (first-key
  latency), but the intervals inside a word barely change
  ([Baus, Strijkers & Costa 2013](https://pmc.ncbi.nlm.nih.gov/articles/PMC3870946/)). Pauses
  before a letter follow how uncertain that letter is in context: typists slow down at the first
  letter and mid-word
  ([Crump, Lai & Brosowsky 2019](https://www.crumplab.com/EntropyTyping/)). The key models and the
  lexicon already know most of that.
- **Korean timing follows syllable structure.** Inter-key times in Korean typing depend on how
  syllables and finals are perceived and on same-hand repetition
  ([이창환·이만영, 인지과학 1994](https://koreascience.kr/article/JAKO199411920276705.page?&lang=en)).
  The finals the kept-final decision is about would leave their own timing marks either way, which
  makes timing a confound rather than a clean signal.
- **Slowing down can be a choice.** Expert typists slow down when errors cost more
  ([Banovic et al., CHI 2017](https://make4all.org/portfolio/quantifying-aversion-to-costly-typing-errors-in-expert-mobile-text-entry/)).
  That supports "slow means careful" across conditions (an important email against chat), but not
  key by key.
- **Prior art:** Apple's application
  [US 2012/0167009](https://patents.justia.com/patent/20120167009) compares keystroke timing with a
  per-user baseline speed to detect typing errors on touch keyboards. I found no published
  evaluation of it, and no study of timing as a sign of shortcut use or of IME reading choice
  (Korean, Japanese or Chinese).
- **Stability:** keystroke timing drifts over weeks (template ageing in keystroke biometrics, for
  example [this 8-week study](https://arxiv.org/pdf/2609.29851)) and with time of day and fatigue. Any
  baseline has to be the user's own and keep updating.
- **Privacy:** inter-key timing is a biometric. It identifies people (keystroke-dynamics
  authentication), leaks what was typed (Song, Wagner & Tian, "Timing Analysis of Keystrokes and
  Timing Attacks on SSH", USENIX Security 2001), and has been used to infer emotional state (Epp,
  Lippold & Mandryk, CHI 2011) and early Parkinson's disease (Giancardo et al., Sci. Rep. 2016).

**In short:** speed is a stable trait of each user that tracks skill, not laziness. From moment to
moment it mostly echoes how predictable the text is. Nothing in the literature links it to taking
shortcuts.

## Where it could plug in

Each decision below is one a style signal would move, with what speed would plausibly predict.

| Decision (code) | What speed might say | Direction (guess) | Plug point |
|---|---|---|---|
| Kept final vs moved (`Kind::KoreanKeepFinals`, `Origin::Kept`; 말이 / 마리, `predict.rs` `candidates`) | a fast run over the final + vowel means the double press was skipped on purpose | fast → kept final more likely | `w.bits(Weight::KeepFinals)` × a factor; or the route ranker's `Preference::bias` per word (`route_odds`) |
| ㄶ by sound, implied ㄹ (`Kind::Aspirated`, `Kind::ImpliedL`) | typing by ear in a fast chat burst | fast → sound reading | `Weight::ImpliedL` bits |
| Implied ㅇ vs bare vowel (`bare`, `Weight::Implied`; ㅠㅠ / 유유) | a run of one key repeated fast is chat jamo | fast repetition → bare | the chat-run rule already decides this from the keys alone |
| Shift+vowel vs capital (`shift_vowel_start`, `has_shift_vowel`, the Shift rule in §10.3) | how long Shift leads the letter; a pause before a deliberate Shift+vowel | none known | the shell would need Shift-down events, which the engine never sees |
| Korean vs English margin (`margin`, the previous word's language) | a fluent burst is a practised word in one language | per user, unknown | a feature crossed with the reading's class in `learned.rs` |
| Route preference (`Ranker::Route`, spec §10.8) | p(optimized route) per typist: bias −1 means about 3 in 4 | slower → each-jamo | `Preference::bias` as a function of the user's speed relative to their own baseline |
| Learned ranker (`learned.rs` `features`) | bucketed timing features (z-scored log interval before the word, word speed against the baseline, longest pause inside) crossed with `class(origin, kind)` | learned | `FEATURES` version bump and retraining, which needs labelled timing data |

## Feasibility

- **API:** today it's `jk_key(engine, key, mods)`. A timed form, `jk_key_at(engine, key, mods, ms)`
  (`Engine::key_at`), would take the event's time from the shell (macOS `NSEvent.timestamp`,
  Windows `GetMessageTime`, web `KeyboardEvent.timeStamp`). The engine stays deterministic: the time
  is just one more input, and the same inputs give the same outputs. `jk_key` would mean "no time
  known", and every timing feature then falls back to neutral, so tests and the lite build don't
  change. Auto-repeat and IME-reprocessed events need their own handling.
- **Features:** the interval before each staged key, kept per staged segment and cleared on commit;
  the pause before the word; word speed and the longest pause in it, each against a running median
  of the user's own intervals, kept in memory only (a few numbers, never the intervals themselves).
- **Data:** there is none.
  1. **Self-recorded (cheapest, decisive):** an opt-in log in the web playground (key, time,
     committed text) of a few thousand words of the owner's own chat and business typing. The commit
     labels each word: whether it was typed lazily (an ㅇ after a final left out, the kept reading
     picked) or diligently. That's enough to measure whether timing separates the two.
  2. **On-device opt-in logging** through the learning framework (§10.11): possible, but spec
     §10.11 promises "no text, no keys, no times", so it would change the notice (moonshot.md §6).
     It should keep running statistics only, never a log.
  3. **Synthetic timing** from published distributions (Dhakal's dataset is English transcription
     on QWERTY) can test the plumbing. It can't test the hypothesis: it carries no laziness labels,
     so any effect would be one we put in.
- **Privacy:** timing must never be stored or exported. Only the in-memory baseline, reset with the
  learner. Even a learned weight conditioned on speed says how fast the user types; that's mild,
  but it's more than the counts the learner stores today.

## Estimate (no timing data)

**How much the style decision matters.** The program types every word of 2+ syllables in
`heldout-ko.txt` (60,622 types, 148,114 tokens, weighted by count) through the full engine (learned
ranker) with Mode A's keys: each jamo, with the doubles on 1–5 and ㅒ ㅖ on `:` `;`. It types each
word diligently (ㅇ typed) and lazily (an ㅇ after a final left out), and checks the top reading at
five `KeepFinals` / `ImpliedL` weights:

| Typed (top-1 errors, token-weighted) | ×0.25 | ×0.5 | ×1 | ×2 | ×4 |
|---|---|---|---|---|---|
| ㅇ after a final (28% of tokens), diligently | 0.01% | 0.01% | **0.16%** | 3.1% | 70% |
| the same words, lazily (마리's keys for 말이) | 100% | 78% | **46%** | 28% | 22% |
| words spelled differently by sound (0.6%), diligently | 0.1% | 0.1% | 0.1% | 1.6% | 14% |
| the same, by sound (안코 for 않고) | 100% | 100% | **67%** | 45% | 16% |

The prior over style moves a lot. A diligent typist wants the weight at ×1 or lower, and a lazy one
wants ×4 (the learner's clamp of 2× stops at 28%). In the typical lazy miss the right word is second,
a ↓ away (있어요 → 이써요, 좋았어요 → 조하써요).

**What a per-word speed signal could add on top of a per-user prior.** Suppose the signal says
"lazy" or "diligent" for each word with accuracy q, and the weight switches on it. Errors on the
ㅇ-after-final words:

| Share of words typed lazily | best constant weight | q = 0.7 | q = 0.8 | q = 0.9 | q = 1 (oracle) |
|---|---|---|---|---|---|
| 20% | 8.0% (×2) | 7.4% | 6.8% | 6.2% | 4.5% |
| 50% | 15.3% (×2) | 15.3% | 15.3% | 14.9% | 11.2% |
| 80% | 22.6% (×2) | 22.6% | 22.0% | 20.2% | 17.9% |

Even a perfect per-word oracle removes only a quarter to a half of these errors. A signal right 4
times in 5, which the literature gives no reason to expect from timing, is worth under a point
unless the typist is rarely lazy. Most of the gain is already in choosing the constant for the user,
and the keys alone show that: whether they type the ㅇ after finals, and what they pick.

**On the evaluation texts there is little left to win.** `tests/corpora.rs` (trusting typist, mixed
styles): 2–3 of 2,008 AI-extended words fail per seed, 0 of 115 of the owner's. The misses are a
Shift+vowel read as a capital (연 → Us, 옮기는 → Hfarls은, 이용료를 → LYdfy를) and one punctuation case.
Timing only bears on the Shift ones, and only through Shift-lead time, which the engine doesn't
receive. In `examples/sxs` the implied-ㅇ alternatives take the top from the right word in 4 of
1,426 (A), 1 of 996 (B) and 1 of 3,000 (D). The style-ambiguous set B′ (lazy keys that also spell a
word: 공유/고유, 앞을/아플, 전역/저녁) has 10 of 113 wrong, and those are ambiguities of intent that no
speed reading resolves.

**Side finding:** `plan::each_jamo` (used by `route_odds` and the travel ranker) still types doubles
with Shift+Q W E R T and ㅒ ㅖ with Shift+O / P (stock 2-set, `Layout::Escaped`). In Mode A since
moonshot.md §9 those keys type ㄱ…ㅅ and 애 / 에, so the route ranker's "each-jamo route" for
있, 했, 까 is no longer a route Mode A types. The estimate above remapped them by hand.

## Recommendation

**Don't add timing now.** The literature ties speed to skill and to how predictable the text is,
not to shortcut use, and finds the typist's own speed stable but drifting. The stakes per word are
real (lazily typed words with an ㅇ after a final are wrong 46% of the time at the default weight),
but they come from the user's habit. A per-user prior captures that habit, and the keys already
show it. A per-word speed signal would have to be right about 9 times in 10 to add a point. It
would also need an API change, retraining, and a change to the privacy notice for a biometric.
Do the cheap things first:

1. A **key-based style estimate**: how often, in committed words with an ㅇ after a final, the user
   kept it in the keys (`d` or the double press) and how often they picked a kept reading the keys
   didn't make. Feed it into the `KeepFinals` / `ImpliedL` weights or the route ranker's `bias`,
   and consider widening the learner's 2× clamp for these two scenarios. The table puts a lazy
   typist's best weight at ×4.
2. **Test timing only with a self-recorded session** (option 1 above). If the timing features,
   relative to the user's own baseline and controlling for word frequency, separate lazily from
   diligently typed words with AUC ≥ 0.9, then design `jk_key_at`. If they don't, drop the idea.
