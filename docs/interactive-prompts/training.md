# Interactive training-file setup (strongly recommended)

You come here at the end of [macos.md](macos.md) or [windows.md](windows.md), with JIS Moonshot
installed and predictive typing accepted. [rules.md](rules.md) still applies, and here it matters
most: you may be reading the user's own writing.

The facts are in [docs/training-contract.md](../training-contract.md): read it first, its **For AI
agents** section in full. Follow it where this prompt is less precise. The file you'll write is
validated by [training-contract.schema.json](../training-contract.schema.json).

## Why recommend it

Say this, in your own words, in two or three sentences:

> JIS Moonshot reads every key three ways: Korean, English and Japanese kana. Its built-in word
> lists are generic, and the Japanese one is small. A training file tells it the words *you* write
> in each language and the readings you never mean, so the right one comes out on top more often.
> It stays on this computer. It takes about ten minutes.

Then ask:

> Shall we set up your training file now?
>
> 1. Yes (recommended)
> 2. Later: tell me where it goes and how to start
> 3. No

- **2:** say where the training folder is (the platform prompt's last step), that
  [training-contract.md](../training-contract.md) explains the file, and that saying "Read
  docs/interactive-prompts/training.md and follow it" in the clone starts this again. Then finish.
- **3:** finish. Don't bring it up again.

## 1. What they type

One question at a time. Write the answers into the session notes.

1. > Which languages do you type, and roughly how much of each? (Korean, English, Japanese, mixed
   > in one sentence?)
2. > Is there writing of yours I may read to find your words: notes, documents, chat exports, a
   > word list? You choose the files; I only read what you name, and only words go into the file.
   >
   > 1. Yes, I'll name files or a folder
   > 2. No, I'll tell you the words myself
   > 3. No words: just the questions

For 1, every file or folder is a scan: ask before reading each (rules.md section 1). Read for words,
not content: don't quote the user's text back, and never put sentences, names of people, addresses,
account numbers or anything private into the file (training-contract.md, "Privacy"). If a file
looks private (mail, chats with others, finances), ask before using anything from it even though
they named it.

## 2. The words

Build the `vocabulary` from what step 1 gave you:

- **korean:** Hangul words they write that are likely missing from the built-in lists: jargon,
  joined compounds (데이터센터), product and team words.
- **english:** Latin words they mix in (kubernetes, paradox).
- **japanese** (JIS Moonshot): words in **kana**, as JIS Moonshot types them, without kanji:
  hiragana for native words (がっこう, ありがとう), katakana for loanwords (サーバー, コーヒー). If they
  write Japanese with kanji, ask how they want the kana reading of their common words, or write the
  readings with them. Only words they use.

Use real counts when you counted them, else `confidence` (`tentative` / `likely` / `confirmed`).
Show the list grouped by language, at most about 30 words per group on screen, and ask them to strike
anything wrong before it goes in.

## 3. What they never mean

Ask, one at a time, and only what fits their languages:

1. > Are there readings that keep coming out on top that you never mean? For example an English
   > word where you meant Korean (god for 행), or kana where you meant Korean.
2. (Japanese typists) > Are there kana words you never type, which you'd rather never see on top?

Add each as an `avoid` entry with its `language` (`korean`, `english` or `japanese`), `soft` unless
they ask for `strong`.

## 4. Taste: the weights

Ask only the questions of training-contract.md's table that matter for this person, then these
for JIS Moonshot, each its own question, each recorded in `preferences`:

| Ask | Answer → weights |
|---|---|
| "How often will you type Japanese kana in JIS Moonshot, compared with Korean?" | often → `kana: 1.5`; sometimes → leave out; rarely → `kana: 0.5`; never → `kana: 0` (the kana reading turns off; F6 / F7 still work) |
| "When you type Japanese, is it mostly loanwords in katakana?" | yes → add the katakana words to `vocabulary.japanese`; no weight |

Leave a weight out when the answer doesn't settle it.

## 5. Examples (optional)

If the data shows words a plain reading gets wrong (기준입니다, 산이, GPU랙), add a few as `examples`,
one word each, with `keys` only when you know how they typed it. Kana words can be examples too
(they need `keys` on the JIS kana layout: きって is `KeyG S-KeyZ KeyW`). Up to 2,000; a few dozen is plenty.

## 6. Write it, check it, place it

1. Assemble the JSON with `"format": "jis-moonshot-training"` (JIS Korean's
   `"jis-korean-training"` is read too) and `"version": 1`, and a `source` naming yourself and the
   date. Show it to the user in full before writing it.
2. Validate it against [training-contract.schema.json](../training-contract.schema.json). If you
   can run commands, a few lines of Python with no third-party packages that check the required
   fields, the types and the limits will do; installing a validator is a question of its own.
3. Writing it is a change (rules.md section 3): ask, then save it as `training.json` in the training
   folder (macOS `~/Library/Application Support/JIS Moonshot/`, Windows `%APPDATA%\JIS Moonshot\`),
   creating the folder if needed. If a `training.json` is already there, show what differs and ask
   whether to replace it, merge, or keep it.
4. It applies the next time JIS Moonshot is selected in a text field (switch away and back). The
   examples are learned in the background and turn on "Learn from My Choices"; say so, and that they
   can turn learning off again in the input menu. The file's words and weights apply either way.

## 7. Try it

Ask them to type a few of their own words in each language they use, and say what came out on top.
For a reading still wrong on top: add the word to `vocabulary`, or the wrong reading to `avoid`, and
write the file again (it re-applies when its content changes).

## 8. Wrap up

Say what's in the file (counts per section, the weights and why), where it is, and how to undo it:
delete `training.json` (its words go the next time JIS Moonshot activates), and "Reset What It
Learned" in the input menu for what the examples taught.
