# The JIS Moonshot logo and mode badges

**License: MIT.** Everything in this folder (the drawings, `make_logo.py`, `render.sh` and this
document), and the rendered icons in `mac/Resources/icons/`, are under the MIT license, the same as
the rest of the repository (`LICENSE`). Copyright 2026 Mirae Research & Min Jun Sung. Use them, change
them, and draw your own badges with them; keep the copyright notice.

| File | What |
|---|---|
| `moonshot.svg` | The logo: the JIS Moonshot disc alone (below). |
| `mode-a.svg` | Mode A, JIS Moonshot: the disc and the ISO/JIS Enter. |
| `mode-a-kana.svg` | JIS Kana (`A_KANA`): the disc with `KA` over its lower right. |
| `mode-a-qwerty.svg` | JIS English (`A_QWERTY`): the disc with `ABC` over its lower right. |
| `mode-a-dvorak.svg`, `mode-a-colemak.svg` | JIS Dvorak (`A_DVORAK`) and JIS Colemak (`A_COLEMAK`): DV and CMK. |
| `make_logo.py` | Draws all of the above. Edit this, never the SVGs. |

JIS Korean's taegeuk logo (with a dark variant) and its Mode B, Mode C, US / UK English and
reserved-mode badges are gone with those modes. There is no dark variant of `moonshot.svg`: it's
drawn in black, as macOS template icons are.
| `render.sh` | Renders the SVGs to the 16 px and 32 px PNGs the macOS input method ships. |
| `downscale.swift` | Scales `render.sh`'s 1024 px renders down to icon size with smooth edges. |

## Design rules

These keep every badge in one family. Follow them for a badge of your own.

1. **One color.** Every shape is the same color. macOS draws input source icons as templates and
   recolors them for light and dark menu bars, so a second color would be lost anyway.
2. **Shapes are separated only by transparent borders.** Nothing is outlined in another color: a
   gap where the background shows through is the only separator.
3. **Overlapping borders are transparent too.** Where a key's border crosses the disc, or
   another key's border, all of it is cut out. Two keys side by side share one gap.
4. **The disc is split by the S and the midpoint gap only.** They cut the disc, never a key.
5. **A badge shows the keys that make its keyboard different,** in real key proportions (key
   units), in the corner where they sit on that keyboard: Enter on the right (JIS Korean's ISO
   badge had its cluster on the left).
6. **Something singled out is hollow, not colored:** a ring as thick as the border, as JIS Korean's ISO
   badge drew the extra key.
7. **It must read at 16 px,** the menu bar's size: at least two clearly separate shapes, the disc
   and the keys. Check on a light and a dark bar.

## How the drawing is built

All badges share a 100 × 100 canvas (`viewBox="0 0 100 100"`).

### The disc

JIS Moonshot's disc is half taegeuk, half solid: the **left half** is the taegeuk with its S cut,
cut off at the vertical midpoint; the **right half** is a solid half disc. A transparent gap as wide
as the S (`MID_GAP`, 6 units) runs down the midpoint and separates them. In the drawing the S is
clipped to the left half (a clip path in `badge()`, an intersection in `lettered()`), and the gap is
a band cut out of the whole disc.

The taegeuk half follows the construction of the flag of Korea:

- A disc of radius *R*.
- The dividing diameter on the flag's **2:3 slope**: its ends are the centre ± *R* · (3, 2) / √13
  (down and to the right in SVG coordinates).
- The **S** is two half circles of radius *R* / 2 along that diameter: from one end to the centre
  (sweep 0), then from the centre to the other end (sweep 1).
- In color the upper part is red (`#cd2e3a`) and the lower blue (`#0047a0`). The badges are
  one color, so the S is drawn as a transparent gap (`S_GAP`, 6 units) instead.

| Drawing | Centre | *R* |
|---|---|---|
| Logo | (50, 50) | 48 |
| Badges (keys or letters on the lower right) | (41, 41) | 39 |

### The keys

- **Key unit *u*:** one standard key's width, chosen per badge so the cluster is about half the
  canvas: 29 (Mode A).
- **Placement:** the cluster is anchored to the canvas corner where it sits on the keyboard, 2
  units in from the edge (`EDGE` = 98), so its border stays inside the canvas.
- **Corners:** radius `CORNER` = 7. The border takes half its width off each edge, leaving about 4.
- **Gap between keys:** 4–5 units, plus the borders.

| Badge | Keys, in key units |
|---|---|
| Mode A, JIS | Enter: 1.5 wide on the upper row, 1.25 on the lower (the step is on the left), 2 rows tall |

### Lettered badges

JIS Kana and the English-layout modes show letters instead of keys (`lettered()` in
`make_logo.py`): the badge's text over the lower right of the disc, (41, 41, 39). The letters are our own drawing in
the style of a bold neo-grotesque (Helvetica): strokes `LETTER` (6) wide with square ends, mitred
corners with sharp points flattened (miter limit 2, so A, V and M have flat tips), a low crossbar on
A, M's middle point on the baseline, and C, G and S ending in horizontal cuts (rectangles masked out
of the stroke). They're drawn, not taken from a font, so no font license comes into the logo.

Lettered badges are computed as geometry rather than drawn with masks (`lettered()`, with
[shapely](https://pypi.org/project/shapely/)): each letter's stroke becomes a filled outline (minus
its cuts), its transparent border is that outline grown by half of `BORDER` (a key's gap), and the
disc is what's left once the S, the midpoint gap and the borders are taken out. Any leftover piece of the disc smaller
than `MIN_PIECE_AREA` or thinner than `MIN_PIECE_THICKNESS` is dropped: the disc shows inside a
letter only where it comes in from outside (U's opening), never as a sliver or as a piece sealed
inside a letter's bowl (G, B, D), which read as a stray shape. The SVG is two plain
filled paths, the disc and the letters. Every letter is trimmed to its box, so all share one cap line and one baseline and have the same
height: diagonals are aimed so their outer edge lands on the box edge and cut flat there, as in
Helvetica (A's apex, V's and M's points, K's and Y's arms). Every letter is `LETTER_WIDTH` ×
`LETTER_HEIGHT` (23 × 32), `LETTER_GAP` (4) apart (two-letter codes are scaled up by
`TWO_LETTER_SCALE`, 1.4, stroke included: 32.2 × 44.8), aligned to the lower right corner `EDGE` in
(`letter_box()`). `letters()` draws A B C D G H K M Q S U V Y Z so far. At 16 px the text is at the limit of rule 7: the disc and a row of letters, not each letter, are what read there.

### The cut-outs (masks)

`make_logo.py` draws each badge as solid shapes with two masks:

- **`s`**, on the disc only: the S, stroked `S_GAP` wide and clipped to the left half, and the
  midpoint gap, a band `MID_GAP` wide.
- **`borders`**, on everything: each key's outline stroked `BORDER` (6) wide, half outside its
  edge and half inside, and for a hollow key its inside inset by `BORDER`.

Because the borders mask covers the whole group, a border cuts the disc and neighbouring keys
alike, which is what makes overlapping borders transparent (rule 3).

## Drawing a badge for a new mode

1. In `make_logo.py`, add a block to `main()` like Mode A's: pick *u*, build the key outlines with
   `rect(x, y, w, h)` (or `rounded([...points])` for a shape like the ISO Enter), put the disc on
   the side away from the keys, and write it with `badge(title, (cx, cy, r), keys, hollow=[...])`.
   For a lettered badge, add it to the `lettered()` loop with its text.
2. Run `python3 make_logo.py` (lettered badges need `pip install shapely`), then look at it at 128, 32 and 16 px on a light and a dark
   background. Check rule 7 above all.
3. Run `sh render.sh` (needs Google Chrome) to write `mac/Resources/icons/<name>.png` and
   `<name>@2x.png`. Add your name to its loop.
4. For the macOS input method, add `$(BUILD)/<name>.tiff` to `ICONS` in `mac/Makefile`, and name
   `<name>.png` as the mode's `tsInputModeMenuIconFileKey` and `tsInputModePaletteIconFileKey` in
   `mac/Info.plist` (the build ships it as a `.tiff` holding both sizes, as Apple's input methods
   do). `docs/modes.md` covers the rest of adding a mode.

Why `render.sh` uses Chrome: macOS's own SVG support (`NSImage`) doesn't draw these masks fully
(it leaves out the key borders), so the PNGs are rendered with a browser engine and committed, and
the build only packs them.

Why `downscale.swift`: the icons are rendered at 1024 px and scaled down in halving steps with
high-quality filtering, so curves and corners stay anti-aliased at 16 and 32 px. A single resample
(`sips`) left them stair-stepped. The result is pure black, with only the edge pixels partly
transparent, as a macOS template icon must be.
