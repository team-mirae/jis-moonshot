# Interactive installation prompts

Prompts for an AI assistant that walk someone through building, installing and setting up **JIS
Moonshot**, and that end with an **interactive training-file setup** (strongly recommended). The
assistant checks the clone, asks which operating system and keyboards, explains predictive typing
(which must be accepted before JIS Moonshot types anything), installs, and then helps the user write
their training file.

JIS Moonshot is an experiment (docs/moonshot.md): there are no official builds and no published
repository yet, so these prompts build from a clone.

## Start

In a coding assistant that can run commands (Claude Code, for example), started in a clone of this
repository, say:

```
Read docs/interactive-prompts/start.md and follow it.
```

In a chat assistant without access to your computer, paste [start.md](start.md), then
[rules.md](rules.md). It can't scan anything, so it shows you how to check things yourself.

## Files

| File | What |
|---|---|
| [start.md](start.md) | The clone check, the questions, the session notes. Read first. |
| [rules.md](rules.md) | Rules for every prompt: ask before scanning, ask before installing, one tool at a time. |
| [macos.md](macos.md) | macOS (supported first): build, install, accept predictive typing, the switch keys |
| [windows.md](windows.md) | Windows: build, install, accept predictive typing, 無変換 / 変換 |
| [training.md](training.md) | The interactive training-file setup every platform prompt ends with |
| [other.md](other.md) | The web playground, and Linux and other systems |

## Writing a prompt

- Write it to the assistant ("Ask…", "Don't…"), with the questions for the user as quotes.
- Link to docs/moonshot.md, the spec and the platform READMEs for facts. Don't copy them in: they
  change, and the copy wouldn't.
- Every prompt follows [rules.md](rules.md). Don't add a step that scans or installs without its
  question.
- Every platform prompt ends with [training.md](training.md).
