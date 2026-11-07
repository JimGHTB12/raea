// Captions and the bed.
import { useContext } from "react";
import { Audio, Sequence, interpolate, useCurrentFrame, useVideoConfig, useWords } from "raea";
import { PARTS } from "./layout";
import { ACCENT, AdContext, META, OVER, TOP, Words, clamp, fit, key, type } from "./all";

// ---- captions ----------------------------------------------------------------------------------------------------

export interface CaptionsProps {
  /** words that stay lit once said, "," between them: "not, more, bets" */
  keys?: string;
  /** words as they should read, "heard=shown" pairs: "two=2, followjet=Followjet" */
  fix?: string;
}

/** Karaoke captions from the take's words: groups of up to three, the word being said lit. Inside <Ad> they sit under
 *  a card, hide under a hook, a strike, the ask or the end card, and go big at the top when the band is empty. */
export function Captions({ keys = "", fix = "" }: CaptionsProps) {
  const f = useCurrentFrame();
  const ad = useContext(AdContext);
  const fixes: Record<string, string> = Object.fromEntries(fix.split(",").map((s) => s.trim().split("=").map((x) => x.trim())).filter((p) => p.length === 2));
  const hot = new Set(keys.split(",").map((s) => s.trim().toLowerCase()).filter(Boolean));
  const ws = useWords().map((w) => ({ ...w, text: fixes[w.text] ?? fixes[w.text.replace(/[.,?!]$/, "")] ?? w.text }));
  const busy = ad?.band.find((b) => f >= b.from && f < b.to);
  if (busy && busy.kind !== "card") return null;
  const groups: Words[] = [];
  for (const w of ws) {
    const g = groups[groups.length - 1];
    const prev = g?.[g.length - 1];
    if (!g || g.length >= 3 || /[.,?!]$/.test(prev!.text) || w.from - prev!.to > 8) groups.push([w]);
    else g.push(w);
  }
  const i = groups.findIndex((g, n) => f >= g[0].from && f < Math.min(groups[n + 1]?.[0].from ?? Infinity, g[g.length - 1].to + 8));
  if (i < 0) return null;
  const g = groups[i];
  const gl = f - g[0].from;
  const text = g.map((w) => w.text).join(" ");
  const size = busy ? 78 : fit(text, 940, 136);
  // under a card in the top band; over a card placed lower (its top less the caption's height)
  const y = !busy ? 260 : busy.top === TOP ? TOP + 476 : busy.top - 120;
  return (
    <div data-caption={text} style={{ position: "absolute", left: 70, top: y, width: 940, textAlign: "center", ...type(size, OVER, -2, true, 0.95), transform: `translateY(${key(gl, { 0: 26, 5: 0 })}px) scale(${key(gl, { 0: 0.8, 3: 1.06, 6: 1 })})` }}>
      {g.map((w, j) => {
        const isKey = hot.has(w.text.toLowerCase().replace(/[^a-z0-9%']/g, ""));
        const lit = isKey ? f >= w.from : f >= w.from && f < Math.max(w.to, w.from + 4);
        return <span key={j} style={{ color: lit ? ACCENT : OVER }}>{w.text + (j < g.length - 1 ? " " : "")}</span>;
      })}
    </div>
  );
}

META.set(Captions, PARTS.Captions);

// ---- the bed -----------------------------------------------------------------------------------------------------

export interface BedProps {
  src: string;
  /** its level under the voice, dB */
  gain?: number;
  /** its level once the voice is done (the last second), dB */
  outro?: number;
}

/** Music under the voice, coming up as the voice ends and fading out over the last 1.4 s. */
export function Bed({ src, gain = -21, outro = -12 }: BedProps) {
  const { fps, durationInFrames: n } = useVideoConfig();
  const words = useWords();
  const done = Math.min(n - 1, (words[words.length - 1]?.to ?? n) + 2);
  const fade = Math.round(1.4 * fps);
  const db = (m: number) => key(m, { 0: gain, [done]: gain, [done + 10]: outro }) + 20 * Math.log10(Math.max(1e-4, interpolate(m, [n - fade, n], [1, 0], clamp)));
  return (
    <Sequence from={0} durationInFrames={n} name="bed" layout="none">
      <Audio src={src} gainDb={db} />
    </Sequence>
  );
}

META.set(Bed, PARTS.Bed);
