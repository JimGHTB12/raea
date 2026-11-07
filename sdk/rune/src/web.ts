// What Rune's props are on the web target (docs/targets.feature T4b). Blitz, like a browser, ignores `data-rune`, so
// `rune()` also writes a `data-rune-*` attribute for each prop that has a CSS fallback, and web.css (gen-css.ts writes
// it) draws them: a motion preset as a CSS transition of its duration, an enter transition as a keyframe animation,
// blur and backdropBlur (whole points, 0 to 64) and glass as filters, instant as no transitions or animations inside.
// Blitz draws all of these (filter, backdrop-filter, transitions and @keyframes; checked 11-06; it doesn't do
// @starting-style). What has no CSS form is named in `data-rune-web-ignored`, which `raea app check` lists on a web app:
// an exit (React removes the element at once), lead and trail, a spring of your own, an effect with numbers of its own,
// a blur that isn't a whole number of points from 0 to 64.

import type { Effect, Motion, RuneProps, SpringPreset } from "./index.ts";

/** Rune's motion tokens (rune-motion tokens.rs): each preset's settle time in seconds and its bounce. */
export const PRESETS: Record<SpringPreset, { duration: number; bounce: number }> = {
  instant: { duration: 0.1, bounce: 0 },
  snap: { duration: 0.25, bounce: 0 },
  settle: { duration: 0.3, bounce: 0 },
  reflow: { duration: 0.4, bounce: 0 },
  glide: { duration: 0.32, bounce: 0 },
  morph: { duration: 0.3, bounce: 0.16 },
  pop: { duration: 0.25, bounce: 0.15 },
  drift: { duration: 0.85, bounce: 0 },
};

/** The entrances web.css has keyframes for, by the attribute value. */
export const ENTERS = ["none", "opacity", "slide", "blurReplace", "move-top", "move-bottom", "move-leading", "move-trailing", "push-top", "push-bottom", "push-leading", "push-trailing"] as const;

/** The largest blur sigma web.css has a rule for. */
export const MAX_BLUR = 64;

const preset = (m: Motion | undefined): SpringPreset | null => (typeof m === "string" ? m : null);

/** An effect's attribute value, or null when it has numbers of its own. */
function enter(e: Effect): string | null {
  if (typeof e === "string") return e;
  if ("move" in e) return `move-${e.move}`;
  if ("push" in e) return `push-${e.push}`;
  return null;
}

/** The web's attributes for `p`, and the names of the props it has no CSS for. */
export function web(p: RuneProps): { attrs: Record<string, string>; ignored: string[] } {
  const attrs: Record<string, string> = {};
  const ignored: string[] = [];
  if (p.motion !== undefined) {
    const m = preset(p.motion);
    if (m) attrs["data-rune-motion"] = m;
    else ignored.push("motion (a spring of its own)");
  }
  if (p.transition !== undefined) {
    const t = typeof p.transition === "object" && "effect" in p.transition ? p.transition : { effect: p.transition as Effect };
    const e = enter(t.effect);
    if (e) attrs["data-rune-enter"] = e;
    else ignored.push("transition (an effect with numbers of its own)");
    if (t.enter !== undefined) {
      const m = preset(t.enter);
      if (m) attrs["data-rune-enter-motion"] = m;
      else ignored.push("transition's enter (a spring of its own)");
    }
    if (e !== "none") ignored.push("transition's exit (React removes the element at once)");
  }
  if (p.lead !== undefined) ignored.push("lead");
  if (p.instant) attrs["data-rune-instant"] = "";
  for (const [k, attr] of [["blur", "data-rune-blur"], ["backdropBlur", "data-rune-backdrop-blur"]] as const) {
    const v = p[k];
    if (v === undefined) continue;
    if (Number.isInteger(v) && v >= 0 && v <= MAX_BLUR) attrs[attr] = String(v);
    else ignored.push(`${k} (${v}: whole points from 0 to ${MAX_BLUR} only)`);
  }
  if (p.glass) attrs["data-rune-glass"] = "";
  if (ignored.length) attrs["data-rune-web-ignored"] = ignored.join("; ");
  return { attrs, ignored };
}
