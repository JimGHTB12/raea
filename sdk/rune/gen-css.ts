// Writes src/web.css: the CSS fallbacks for Rune's props on the web target (src/web.ts, docs/targets.feature T4b).
//   bun sdk/rune/gen-css.ts        (src/web.css is committed; a test checks it's what this writes)
import { ENTERS, MAX_BLUR, PRESETS } from "./src/web.ts";

/** A spring as a cubic-bezier: a critically damped one eases out; a bouncy one overshoots by its bounce. */
const ease = (bounce: number) => (bounce > 0 ? `cubic-bezier(0.34, ${(1 + bounce * 3).toFixed(2)}, 0.64, 1)` : "cubic-bezier(0.2, 0, 0, 1)");

const ms = (s: number) => `${Math.round(s * 1000)}ms`;

/** Each entrance's first frame, as Rune's preset transitions start (rune-motion transition.rs). */
const FROM: Record<(typeof ENTERS)[number], string> = {
  none: "",
  opacity: "opacity: 0;",
  slide: "transform: translateX(-100%);",
  blurReplace: "opacity: 0; transform: scale(0.96); filter: blur(1.5px);",
  "move-top": "transform: translateY(-100%);",
  "move-bottom": "transform: translateY(100%);",
  "move-leading": "transform: translateX(-100%);",
  "move-trailing": "transform: translateX(100%);",
  "push-top": "opacity: 0; transform: translateY(-100%);",
  "push-bottom": "opacity: 0; transform: translateY(100%);",
  "push-leading": "opacity: 0; transform: translateX(-100%);",
  "push-trailing": "opacity: 0; transform: translateX(100%);",
};

export function css(): string {
  const out = ["/* @raea/rune on the web target (docs/targets.feature T4b): written by gen-css.ts, don't edit. */"];
  const props = "opacity, transform, translate, scale, rotate, width, height, background-color, color, border-color, box-shadow, filter";
  for (const [name, p] of Object.entries(PRESETS)) {
    out.push(`[data-rune-motion="${name}"] { transition-property: ${props}; transition-duration: ${ms(p.duration)}; transition-timing-function: ${ease(p.bounce)}; }`);
  }
  for (const e of ENTERS) {
    if (e === "none") {
      out.push(`[data-rune-enter="none"] { animation: none; }`);
      continue;
    }
    out.push(`@keyframes rune-enter-${e} { from { ${FROM[e]} } }`);
    out.push(`[data-rune-enter="${e}"] { animation-name: rune-enter-${e}; animation-duration: ${ms(PRESETS.settle.duration)}; animation-timing-function: ${ease(0)}; animation-fill-mode: backwards; }`);
  }
  // an enter's own spring, after the effects so it wins
  for (const [name, p] of Object.entries(PRESETS)) {
    out.push(`[data-rune-enter-motion="${name}"] { animation-duration: ${ms(p.duration)}; animation-timing-function: ${ease(p.bounce)}; }`);
  }
  out.push(`[data-rune-instant], [data-rune-instant] * { transition: none !important; animation: none !important; }`);
  for (let s = 0; s <= MAX_BLUR; s++) out.push(`[data-rune-blur="${s}"] { filter: blur(${s}px); }`);
  for (let s = 0; s <= MAX_BLUR; s++) out.push(`[data-rune-backdrop-blur="${s}"] { backdrop-filter: blur(${s}px); }`);
  // the system's glass is a blur where there's no system material (the SDK's own words for it)
  out.push(`[data-rune-glass] { backdrop-filter: blur(24px); }`);
  return out.join("\n") + "\n";
}

if (import.meta.main) await Bun.write(new URL("./src/web.css", import.meta.url), css());
