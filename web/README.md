# web

Browser playground for JIS Moonshot, and its docs pages. It runs the real engine (`/engine`,
compiled to wasm32), so it behaves like the desktop input methods: Mode A (Korean, English and kana
readings) and JIS Kana, with predictions always on.

It's deployed to https://jis-moonshot.sjun.me by `.github/workflows/deploy-web.yml` on every push
to `main`. Locally:

```sh
npm install
npm run dev     # builds the engine to wasm, then starts Vite
npm run build   # production build in dist/
```

Needs Node 24 (24.22.1 or later) or 26.12.1+ (the patched lines; `.nvmrc` pins 24.22.1, `package.json`'s
`engines` asks for it) and the Rust `wasm32-unknown-unknown` target
(`rustup target add wasm32-unknown-unknown`).

## Key-press counts

The live comparison under the typing area runs the engine's planner (`engine/src/plan.rs`, exported
as `jk_plan`) on whatever is in the box, or on `public/sample.txt` when it's empty. Shift counts as
a press, and a character a layout has no key for counts as 3 on that layout.

The table of well-known texts is precomputed into `src/stats.json` by:

```sh
python3 -I web/tools/build_stats.py   # from the repo root; fetches from Korean Wikisource, needs cargo
```

It stores only the counts and the exact source revisions, not the texts.
