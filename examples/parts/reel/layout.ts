// Where <Ad>'s parts sit in time, with no React and no imports: index.tsx renders from it, and the feed's host lays a
// composition's parts out with it (apps/app/host/parts.ts), so the parts lane on the timeline is the render's layout.
// The phrase finder is passed in (findPhrase from raea), which keeps this file importable from anywhere.

export type Kind = "take" | "band" | "card" | "captions" | "full" | "sound";
export interface PartMeta {
  kind: Kind;
  seconds: number;
  ask?: boolean;
  needs?: string[];
  /** its lines are the speaker's words (Phrases): given no cue, it starts where its first line's first words are said */
  says?: boolean;
}
/** every part the reel has, by name */
export const PARTS: Record<string, PartMeta> = {
  Take: { kind: "take", seconds: 0 },
  Hook: { kind: "band", seconds: 2.5, needs: ["text"] },
  StatCard: { kind: "card", seconds: 4, needs: ["big"] },
  Strike: { kind: "band", seconds: 5, needs: ["text", "not"] },
  Phrases: { kind: "band", seconds: 8, needs: ["text"], says: true },
  Callout: { kind: "card", seconds: 3, needs: ["title"] },
  Follow: { kind: "band", seconds: 2.5, ask: true, needs: ["text"] },
  EndCard: { kind: "full", seconds: 2.5, ask: true, needs: ["name"] },
  Captions: { kind: "captions", seconds: 0 },
  Bed: { kind: "sound", seconds: 0 },
};

/** how a part is placed in time (the timing half of parts.tsx's Cue) */
export interface Timing {
  say?: string;
  occurrence?: number;
  until?: string;
  seconds?: number;
  from?: number;
  beat?: number;
  untilBeat?: number;
  /** the part's lines, " / " between them: a part that says them (Phrases) and has no cue starts on the first */
  text?: string;
}
export interface Word {
  text: string;
  /** first frame */
  from: number;
  /** the frame after it ends */
  to: number;
}
/** findPhrase from @raea/video/phrase: the frame a phrase is said */
export type Find = (words: readonly Word[], phrase: string, options?: { occurrence?: number; edge?: "start" | "end" }) => number;
export interface Span {
  from: number;
  to: number;
}
export interface Placed {
  /** its place among <Ad>'s children */
  i: number;
  name: string;
  cue: Timing;
}
export interface Laid {
  spans: Map<number, Span>;
  /** the parts in the top band, the cards and the full-frame ones, in time order */
  band: (Span & { kind: Kind; i: number })[];
}

/** the shortest a part cued on the music stays up, seconds */
export const READ = 1.5;

/** The frame of the bed's beat n; the error names the part. */
export function onBeat(beats: readonly number[], part: string, n: number): number {
  if (!beats.length) throw new Error(`<${part}>: beat={${n}} needs the bed's beats, as <Ad beats={BEATS}>`);
  if (!Number.isInteger(n) || n < 0 || n >= beats.length) throw new Error(`<${part}>: there is no beat ${n} (the cut has beats 0 to ${beats.length - 1})`);
  return beats[n];
}

/** Where a phrase lands, said after `after` if it can be; the error names the part. */
export function when(find: Find, words: readonly Word[], part: string, phrase: string, opts: { edge?: "start" | "end"; occurrence?: number; after?: number } = {}): number {
  try {
    if (opts.after !== undefined && opts.occurrence === undefined) {
      for (let n = 1; ; n++) {
        const at = find(words, phrase, { occurrence: n, edge: opts.edge });
        if (at >= opts.after) return at;
      }
    }
    return find(words, phrase, { occurrence: opts.occurrence, edge: opts.edge });
  } catch (e) {
    if (opts.after !== undefined && opts.occurrence === undefined) {
      try { return find(words, phrase, { edge: opts.edge }); } catch { /* the first error says it */ }
    }
    throw new Error(`<${part}>: ${(e as Error).message.replace(/^word\(/, "the phrase (")}`);
  }
}

/** The first words of a part's first line as a phrase to find (two, or one of three letters or more; *accents* plain),
 *  the words a line is said by (parts.tsx lands each line on them). */
export function firstWords(text: string): string[] {
  const ws = (text.split(" / ").map((l) => l.trim()).find(Boolean) ?? "").replace(/\*/g, "").split(/\s+/).filter(Boolean);
  return ws.length ? [ws.slice(0, 2).join(" "), ...(ws.length > 1 && ws[0].length >= 3 ? [ws[0]] : [])] : [];
}

/** Where a part that says its lines starts when it isn't cued: its first line's first words, as `say` would give them
 *  (undefined when they aren't said: it then starts at 0 and the part says which line isn't said). */
function saidFirst(find: Find, words: readonly Word[], text: string): number | undefined {
  for (const phrase of firstWords(text)) {
    try {
      return find(words, phrase);
    } catch {
      // not said: try the first word alone
    }
  }
  return undefined;
}

/** One part's span from its own cue, before <Ad> fits the band together. */
export function spanOf(find: Find, name: string, p: Timing, meta: PartMeta, words: readonly Word[], beats: readonly number[], fps: number, total: number): Span {
  const from = p.from ?? (p.beat !== undefined ? onBeat(beats, name, p.beat) : p.say ? when(find, words, name, p.say, { occurrence: p.occurrence }) : meta.says && p.text ? saidFirst(find, words, p.text) ?? 0 : 0);
  // untilBeat is the last beat it's up for (as `until` is the last word): it leaves where the next beat starts
  const to = p.untilBeat !== undefined ? (p.untilBeat + 1 < beats.length ? onBeat(beats, name, p.untilBeat + 1) : (onBeat(beats, name, p.untilBeat), total)) : p.until ? when(find, words, name, p.until, { edge: "end", after: from }) : from + Math.round((p.seconds ?? meta.seconds) * fps);
  return { from, to: Math.min(total, Math.max(from + 1, to)) };
}

/** <Ad>'s layout: each part's span, the band's parts ending where the next starts, the ask holding to the end, and a
 *  part cued on the music up at least READ seconds (else the error says how to fix it). */
export function layout(find: Find, items: Placed[], words: readonly Word[], beats: readonly number[], fps: number, total: number): Laid {
  const spans = new Map<number, Span>();
  const band: (Span & { kind: Kind; i: number })[] = [];
  const at = new Map(items.map((it) => [it.i, it]));
  for (const it of items) {
    const meta = PARTS[it.name];
    if (!meta || meta.kind === "take" || meta.kind === "captions") continue;
    const s = spanOf(find, it.name, it.cue, meta, words, beats, fps, total);
    spans.set(it.i, s);
    if (meta.kind === "band" || meta.kind === "card" || meta.kind === "full") band.push({ ...s, kind: meta.kind, i: it.i });
  }
  band.sort((x, y) => x.from - y.from);
  band.forEach((b, n) => {
    const next = band[n + 1];
    const it = at.get(b.i)!;
    const p = it.cue;
    // a part without its own `until` ends where the next one starts
    if (next && b.to > next.from) b.to = Math.max(b.from + 1, next.from);
    // the ask, last in the band and given no length, holds to the end: the piece ends on it, not on a caption
    if (!next && PARTS[it.name]?.ask && !p.until && p.untilBeat === undefined && p.seconds === undefined) b.to = total;
    // a part cued on the music is up long enough to read: a card for a flip is gone before it's seen (promo run 7 put
    // thirteen cards a beat or two each). The error says how to fix it, so the writer can.
    if (p.beat !== undefined && b.to - b.from < READ * fps && b.to < total)
      throw new Error(`<${it.name} beat={${p.beat}}> is on screen ${((b.to - b.from) / fps).toFixed(1)} s (${next && b.to === next.from ? `until the next part, at beat ${at.get(next.i)!.cue.beat}` : `through beat ${p.untilBeat}`}); a part stays up at least ${READ} s, a bar or more, so it can be read: end it later, or drop it`);
    spans.set(b.i, { from: b.from, to: b.to });
  });
  return { spans, band };
}
