// measureText: each character's advance from the host's faces (canvas.advances), else the bundled metrics.
import * as wire from "../wire";
import { S, metrics, parseFont } from "./all";

/** measureText's answer for `text` in a CSS `font`. */
export function measure(font: string, text: string) {
  const f = parseFont(font)!;
  const fam = f.family.replace(/["']/g, "").split(",")[0].trim().toLowerCase();
  const m = metrics.get(fam) ?? metrics.get(S.defaultFamily.toLowerCase());
  const host = hostAdvances(f, text);
  // without the host's advances: the default face's (Fira Mono, monospace: 600 units of 1000 a character)
  let w = 0;
  for (const ch of text) w += host?.get(ch) ?? (m ? (m.adv[ch] ?? m.dflt) / m.em : 0.6);
  w *= f.size;
  return { width: w, actualBoundingBoxLeft: 0, actualBoundingBoxRight: w, fontBoundingBoxAscent: f.size * 0.8, fontBoundingBoxDescent: f.size * 0.2, actualBoundingBoxAscent: f.size * 0.7, actualBoundingBoxDescent: 0 };
}

/** Each character's advance (a fraction of the size) in a font, as the host draws it (canvas.advances, systems/canvas
 *  measure.rs): asked once per character the font's own family has; one another family stood in for is asked again, so a
 *  face loaded later takes over. Undefined where there's no host to ask. */
const ADVANCES = new Map<string, Map<string, number>>();
function hostAdvances(f: { family: string; weight: string; style: string }, text: string) {
  const key = `${f.family}|${f.weight}|${f.style}`;
  let known = ADVANCES.get(key);
  if (!known) ADVANCES.set(key, (known = new Map()));
  const ask = [...new Set(text)].filter((c) => !known!.has(c));
  if (!ask.length) return known;
  const r = wire.callSync<{ adv: (number | null)[]; own: boolean[] }>("canvas.advances", { font: f, chars: ask.join("") });
  if (!r) return known.size ? known : undefined;
  const now = new Map(known);
  ask.forEach((c, i) => {
    const a = r.adv[i];
    if (a == null) return;
    now.set(c, a);
    if (r.own[i]) known!.set(c, a);
  });
  return now;
}

