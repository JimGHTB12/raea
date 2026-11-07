// The band's parts: words landed, Hook and StatCard.
import { useContext } from "react";
import { findPhrase, useCurrentFrame, useWords } from "raea";
import { PARTS } from "./layout";
import { ACCENT, BLIP, BODY, CARD, Cue, Hit, INK, OVER, POP, THUD, TOP, TopContext, W, Words, accent, ems, fit, key, lines, lockup, part, plain, rise, type } from "./all";

// ---- the band's parts --------------------------------------------------------------------------------------------

/** One line of a lockup: the accent lines pop, the rest rise; each lands when it's said (or with the part). */
export function LockLine({ line, at, size, f }: { line: string; at: number; size: number; f: number }) {
  const hot = line.startsWith("*") && line.endsWith("*") && !line.slice(1, -1).includes("*");
  const local = f - at;
  return (
    <div style={{ height: Math.round(size * 0.92), marginBottom: 14, opacity: local >= 0 ? 1 : 0 }}>
      <div style={{ ...type(size, hot ? ACCENT : OVER), textAlign: "center", transform: hot ? `scale(${key(local, POP)})` : `translateY(${rise(local)}px)` }}>{accent(line)}</div>
    </div>
  );
}

/** each line's landing frame (local): a plain line lands with the part, two frames after the line before it; a line
 *  with an *accent* (or every line, `wait`) waits for its first two words, or its first word alone, said inside the
 *  part, else lands with the line before it (or, `strict`, is refused by name, saying where it is said: `from` is the
 *  part's first frame in the composition, which the words are local to); lines land in order */
export function landings(words: Words, ls: string[], len: number, first = 0, wait = false, strict = "", from = 0): number[] {
  const out: number[] = []
  const said = (phrase: string, after: number) => {
    for (let n = 1; n < 20; n++) {
      try {
        const f = findPhrase(words, phrase, { occurrence: n })
        if (f >= len - 6) return undefined
        if (f >= after) return f
      } catch {
        return undefined
      }
    }
    return undefined
  }
  ls.forEach((l, i) => {
    const prev = i ? out[i - 1] : first
    if (!wait && !l.includes("*")) return void out.push(i ? prev + 2 : first)
    const ws = plain(l).split(/\s+/)
    const at = said(ws.slice(0, 2).join(" "), prev) ?? (ws[0].length >= 3 && !strict ? said(ws[0], prev) : undefined)
    if (at === undefined && strict) throw new Error(`<${strict}> line "${plain(l)}" ${unsaid(words, ws.slice(0, 2).join(" "), len, from, i ? undefined : strict)}`)
    out.push(at ?? prev)
  })
  return out
}

/** Why a line isn't said where the part needs it, and what to change: where it is said instead (in the composition's
 *  frames), or that it isn't said at all. `first` names the part when this is its first line: starting the part there
 *  fixes it. */
function unsaid(words: Words, phrase: string, len: number, from: number, first?: string): string {
  let at: number | undefined;
  try {
    at = findPhrase(words, phrase);
  } catch {
    // said nowhere
  }
  if (at === undefined) return "isn't said in the take: each line is the speaker's words, in the order said";
  const n = from + at, win = `frames ${from}–${from + len}`;
  if (at >= 0 && at < len - 6) return `is said at frame ${n}, before the line before it: the lines go in the order said`;
  if (first) return `is said at frame ${n}, outside the part's window (${win}): start the ${first} at frame ${n}, as the next part in the band (say="${phrase}")`;
  if (at < 0) return `is said at frame ${n}, before the part's window (${win}): start the part at its first line, or take this line out`;
  return `is said at frame ${n}, after the part's window (${win}): give \`until\` the last line's words so the part is up until then`;
}

/** The first frame at or after 0 a line's first words are said (two, or one of three letters or more), or Infinity. */
export function saidFrom(words: Words, line: string): number {
  const ws = plain(line).split(/\s+/)
  for (const phrase of [ws.slice(0, 2).join(" "), ...(ws[0].length >= 3 ? [ws[0]] : [])]) {
    for (let n = 1; n < 20; n++) {
      try {
        const f = findPhrase(words, phrase, { occurrence: n })
        if (f >= 0) return f
      } catch {
        break
      }
    }
  }
  return Infinity
}

export interface HookProps extends Cue {
  /** the lines, " / " between them; a line wrapped whole in *stars* is the accent line and pops */
  text: string;
}

/** The hook: a few lines stacked big in the top band, each landing as it's said. */
export const Hook = part<HookProps>("Hook", PARTS.Hook, ({ text, sound, span }) => {
  const f = useCurrentFrame();
  const TOP = useContext(TopContext);
  const ls = lockup(text);
  const at = landings(useWords(), ls, span.to - span.from);
  return (
    <>
      <div style={{ position: "absolute", left: 90, top: TOP, width: 900, display: "flex", flexDirection: "column" }}>
        {ls.map((l, i) => <LockLine key={i} line={l} at={at[i]} f={f} size={fit(l, 900, /^\*.*\*$/.test(l) ? 220 : 130)} />)}
      </div>
      <Hit sound={sound} fallback="deep punchy cinematic impact hit, tight, no reverb tail" gain={-16} seconds={1.2} screen={THUD} />
    </>
  );
});

export interface Bar {
  label: string;
  /** what it reads at its end: "2%", "~50%" */
  value: string;
  /** how full, 0 to 1 */
  fill: number;
  /** the phrase it fills on (with the card when not given) */
  say?: string;
}

export interface StatCardProps extends Cue {
  /** the number, big: "2%" */
  big: string;
  /** up to three lines beside it, " / " between them, *accent* words lit */
  text?: string;
  /** up to two bars that fill as they're said */
  bars?: Bar[];
}

/** A dark card in the top band: the number big, lines beside it, bars filling on their words. */
export const StatCard = part<StatCardProps>("StatCard", PARTS.StatCard, ({ big, text = "", bars = [], sound, span }) => {
  const f = useCurrentFrame();
  const TOP = useContext(TopContext);
  const words = useWords();
  const PAD = 50, TRACK = W - 2 * PAD;
  const ls = lines(text).slice(0, 3);
  const bigSize = Math.min(250, Math.floor(420 / Math.max(1, ems(big))));
  // each line as big as its own width allows (a short line isn't held to the longest's size), the lines together
  // within the number's height
  const room = W - 2 * PAD - ems(big) * bigSize - 30;
  const fitted = ls.map((l) => fit(l, room, ls.length > 2 ? 74 : 92));
  const tall = fitted.reduce((n, z) => n + Math.round(z * 0.97), 0);
  const sizes = fitted.map((z) => Math.floor(z * Math.min(1, 220 / Math.max(1, tall))));
  const at = landings(words, ls, span.to - span.from, 2);
  return (
    <>
      <div style={{ position: "absolute", left: 60, top: TOP, width: W, height: bars.length ? 440 : 300, boxSizing: "border-box", padding: `20px ${PAD}px 0`, borderRadius: 44, backgroundColor: CARD, boxShadow: "0 20px 60px rgba(0,0,0,0.35)", display: "flex", flexDirection: "column", transform: `translateY(${key(f, { 0: -30, 7: 0 })}px)`, opacity: key(f, { 0: 0, 4: 1 }) }}>
        <div style={{ height: 246, display: "flex", flexDirection: "row", alignItems: "flex-start", gap: 18 }}>
          <div style={{ ...type(bigSize, ACCENT, -6, false), flexShrink: 0, transform: `scale(${key(f, POP)})` }}>{big}</div>
          <div style={{ marginTop: 24, flexGrow: 1, display: "flex", flexDirection: "column" }}>
            {ls.map((l, i) => (
              <div key={i} style={{ height: Math.round(sizes[i] * 0.97), opacity: f >= at[i] ? 1 : 0 }}>
                <div style={{ ...type(sizes[i], i === 1 && !l.includes("*") ? "#b8b8b8" : INK, -1, false), transform: `translateY(${rise(f - at[i])}px)` }}>{accent(l)}</div>
              </div>
            ))}
          </div>
        </div>
        {bars.slice(0, 2).map((b, i) => {
          let from = 8 + 6 * i;
          if (b.say) try { from = Math.max(0, findPhrase(words, b.say)); } catch (e) { throw new Error(`<StatCard> bar "${b.label}": ${(e as Error).message}`); }
          const fill = Math.max(18, Math.min(1, Math.max(0, b.fill)) * TRACK);
          return (
            <div key={i} style={{ marginTop: i === 0 ? 0 : 2 }}>
              <div style={{ height: 38, marginBottom: 8, display: "flex", justifyContent: "space-between", alignItems: "flex-end" }}>
                <div style={{ fontFamily: BODY, fontWeight: 800, fontSize: 38, lineHeight: 1, letterSpacing: 2, color: "#e0e0e0", textTransform: "uppercase", whiteSpace: "nowrap" }}>{b.label}</div>
                <div style={{ ...type(42, ACCENT, 0, false, 1), opacity: f >= from ? 1 : 0 }}>{b.value}</div>
              </div>
              <div style={{ position: "relative", width: TRACK, height: 40, borderRadius: 20, backgroundColor: "rgba(255,255,255,0.16)" }}>
                <div style={{ position: "absolute", left: 0, top: 0, height: 40, borderRadius: 20, backgroundColor: ACCENT, width: key(f, { [from]: 18, [from + 24]: fill }) }} />
              </div>
            </div>
          );
        })}
      </div>
      <Hit sound={sound} fallback="quick digital swoosh, bright, short" gain={-21} screen={BLIP} />
    </>
  );
});
