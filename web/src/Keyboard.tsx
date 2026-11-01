import type { CSSProperties } from "react";
import type { Engine } from "./engine";

type Row = { offset: number; codes: string[] };

// The JIS kana-typing area (spec §2), with F6–F10 over 6–0. Offsets are in key units.
const ROWS: Row[] = [
  { offset: 5, codes: ["F6", "F7", "F8", "F9", "F10"] },
  {
    offset: 0,
    codes: ["Digit1", "Digit2", "Digit3", "Digit4", "Digit5", "Digit6", "Digit7", "Digit8", "Digit9", "Digit0", "Minus", "Equal", "IntlYen"],
  },
  { offset: 0.5, codes: [..."QWERTYUIOP"].map((l) => "Key" + l).concat("BracketLeft", "BracketRight") },
  { offset: 0.75, codes: [..."ASDFGHJKL"].map((l) => "Key" + l).concat("Semicolon", "Quote", "Backslash") },
  { offset: 1.25, codes: [..."ZXCVBNM"].map((l) => "Key" + l).concat("Comma", "Period", "Slash", "IntlRo") },
];

// F6 / F7 show the staged keys as hiragana / katakana, F8 (and Shift+F8) as kanji.
const FUNCTION_KEYS: Record<string, { label: string; planned?: boolean }> = {
  F6: { label: "ひら" },
  F7: { label: "カナ" },
  F8: { label: "漢字" },
  F9: { label: "전각" },
  F10: { label: "반각" },
};

// The editing keys, under the board, so a touch screen (no physical keyboard) can type spaces,
// delete and start new lines. Shift is the toggle beside the typing area.
const EDIT_KEYS: { code: string; label: string; name: string }[] = [
  { code: "Backspace", label: "⌫", name: "Backspace" },
  { code: "Space", label: "", name: "Space" },
  { code: "Enter", label: "⏎", name: "Enter" },
];

interface Props {
  engine: Engine;
  /** The mode: 0 = Mode A (JIS Moonshot), 11 = JIS Kana. Both use the JIS board. */
  mode: number;
  shift: boolean;
  pressed: Set<string>;
  /** Float over the page (fixed to the bottom of the window) while the text is long. */
  floating: boolean;
  onPress(code: string): void;
}

export function Keyboard({ engine, mode, shift, pressed, floating, onPress }: Props) {
  return (
    <div className={"board" + (floating ? " floating" : "")}>
      <div className="plate" role="group" aria-label={mode === 11 ? "JIS Kana keyboard" : "JIS Moonshot keyboard"}>
        {ROWS.map((row, i) => (
          <div key={i} className={"row" + (i === 0 ? " fn-row" : "")} style={{ "--offset": row.offset } as CSSProperties}>
            {row.codes.map((code) => {
              const fn = FUNCTION_KEYS[code];
              const base = fn ? code : engine.legend(code, false);
              const shifted = fn ? fn.label : engine.legend(code, true);
              const letter = code.startsWith("Key") ? code.slice(3) : "";
              const className = [
                "key",
                fn && "fn",
                fn?.planned && "planned",
                !letter && !fn && "sym",
                pressed.has(code) && "down",
                (code === "KeyF" || code === "KeyJ") && "home",
              ]
                .filter(Boolean)
                .join(" ");
              return (
                <button
                  key={code}
                  type="button"
                  className={className}
                  title={fn?.planned ? "Planned" : undefined}
                  aria-label={`${code}: ${base}${shifted && shifted !== base ? `, Shift: ${shifted}` : ""}`}
                  // Keep focus in the typing area so the composition isn't committed.
                  onMouseDown={(e) => e.preventDefault()}
                  onClick={() => onPress(code)}
                >
                  {shifted !== base && <span className={"shifted" + (shift && !fn ? " live" : "")}>{shifted}</span>}
                  <span className="base">{base}</span>
                  {letter && <span className="latin">{letter}</span>}
                </button>
              );
            })}
          </div>
        ))}
        <div className="row edit-row">
          {EDIT_KEYS.map(({ code, label, name }) => (
            <button
              key={code}
              type="button"
              className={"key edit" + (code === "Space" ? " space" : "") + (pressed.has(code) ? " down" : "")}
              aria-label={name}
              onMouseDown={(e) => e.preventDefault()}
              onClick={() => onPress(code)}
            >
              <span className="base">{label}</span>
            </button>
          ))}
        </div>
      </div>
    </div>
  );
}
