// song(): cuts and frames on its beats, and useSong.
import { useCurrentFrame, useVideoConfig } from "@raea/app/clock";
import { SongEvent, SongEventKind, SongMap, SongMoment, check, floorIndex, songAt } from "./all";

export interface SongOptions {
  /** the composition's frame rate */
  fps: number;
  /** seconds into the song at frame 0 (where an <Audio> of it starts: its startFrom over fps) */
  offset?: number;
}

export interface FrameRange {
  /** first frame (inclusive), default 0 */
  from?: number;
  /** last frame (exclusive), default the song's end */
  to?: number;
}

export interface CutOptions extends FrameRange {
  /** "section" (default) takes each section's rate, steady across it; "bar" each bar's own */
  by?: "section" | "bar";
  /** multiplies every rate (2: twice as many cuts), kept on the ladder 0.25..4 */
  scale?: number;
  /** cut on every drop (default true) */
  drops?: boolean;
  /** cut on a stop's hits (default true) */
  stops?: boolean;
  /** double the rate through a build's second half (default true) */
  builds?: boolean;
  /** cuts closer than this many frames are merged, the earlier kept (default a quarter second) */
  minGap?: number;
}

export interface Song {
  map: SongMap;
  fps: number;
  offset: number;
  /** the frame a song time falls on */
  frame(seconds: number): number;
  /** the song's seconds at a frame */
  seconds(frame: number): number;
  /** where the song is at a frame */
  at(frame: number): SongMoment;
  /** frames of every `every`-th beat (1: each beat), counted from the song's first */
  onBeat(every?: number, range?: FrameRange): number[];
  /** frames of every `every`-th downbeat */
  onDownbeat(every?: number, range?: FrameRange): number[];
  /** frames of every event of a kind (a build's or stop's start), with the event */
  eventsOf(kind: SongEventKind, range?: FrameRange): { frame: number; event: SongEvent }[];
  /** frames to cut on, paced to the energy: each section's (or bar's) rate, on its beats, plus drops, a stop's hits,
   *  and a build's second half at double rate */
  cutsAt(options?: CutOptions): number[];
}

const LADDER = [0.25, 0.5, 1, 2, 4];

function onLadder(r: number): number {
  return LADDER.reduce((best, x) => (Math.abs(Math.log2(x) - Math.log2(r)) < Math.abs(Math.log2(best) - Math.log2(r)) ? x : best), 1);
}

/** A song map bound to a frame rate (and where frame 0 sits in the song): frames from song times, and the reverse. */
export function song(map: SongMap, options: SongOptions): Song {
  check(map);
  const { fps, offset = 0 } = options;
  if (!(fps > 0)) throw new RangeError(`song: fps must be positive, got ${fps}`);
  const frame = (s: number) => Math.round((s - offset) * fps);
  const seconds = (f: number) => f / fps + offset;
  const inRange = (r: FrameRange = {}) => {
    const from = r.from ?? 0;
    const to = r.to ?? frame(map.duration) + 1;
    return (f: number) => f >= from && f < to;
  };
  const every = (times: readonly number[], k: number, r?: FrameRange) => {
    if (!Number.isInteger(k) || k < 1) throw new RangeError(`song: every must be a whole number from 1, got ${k}`);
    const ok = inRange(r);
    return times.filter((_, i) => i % k === 0).map(frame).filter(ok);
  };
  const s: Song = {
    map,
    fps,
    offset,
    frame,
    seconds,
    at: (f) => songAt(map, seconds(f)),
    onBeat: (k = 1, r) => every(map.beats, k, r),
    onDownbeat: (k = 1, r) => every(map.downbeats, k, r),
    eventsOf: (kind, r) => {
      const ok = inRange(r);
      return map.events.filter((e) => e.kind === kind).map((event) => ({ frame: frame(event.time), event })).filter((x) => ok(x.frame));
    },
    cutsAt: (o = {}) => {
      const { by = "section", scale = 1, drops = true, stops = true, builds = true } = o;
      const minGap = o.minGap ?? Math.max(1, Math.round(fps / 4));
      const ok = inRange(o);
      const times: number[] = [];
      const bpb = map.beatsPerBar || 4;
      const builds_ = map.events.filter((e) => e.kind === "build");
      const stops_ = map.events.filter((e) => e.kind === "stop");
      const sectionOf = (t: number) => map.sections[Math.max(0, floorIndex(map.sections.map((x) => x.time), t))];
      map.bars.forEach((bar, j) => {
        const sec = sectionOf(bar.time);
        // a stop's bars cut on its hits instead
        if (stops && stops_.some((st) => st.end !== undefined && bar.time >= st.time && bar.time < st.end)) return;
        let rate = (by === "bar" ? bar.cuts : sec?.cuts ?? bar.cuts) * scale;
        const b = builds_.find((e) => e.end !== undefined && bar.time >= (e.time + e.end) / 2 && bar.time < e.end);
        if (builds && b) rate *= 2;
        rate = onLadder(Math.min(4, Math.max(0.25, rate)));
        if (rate < 1) {
          // one cut every 1/rate bars, counted from the section's first bar
          const k = Math.round(1 / rate);
          const first = sec ? map.bars.findIndex((x) => x.time >= sec.time - 1e-6) : 0;
          if ((j - Math.max(0, first)) % k === 0) times.push(bar.time);
          return;
        }
        // rate cuts spread over the bar's beats
        const len = bar.beats || bpb;
        const step = len / rate;
        for (let c = 0; c < rate; c++) {
          const at = bar.beat + Math.round(c * step);
          if (at < bar.beat + len && at < map.beats.length) times.push(map.beats[at]);
        }
      });
      if (drops) for (const e of map.events) if (e.kind === "drop") times.push(e.time);
      if (stops) for (const st of stops_) for (const h of st.hits ?? []) times.push(h.time);
      const frames = [...new Set(times.map(frame))].filter(ok).sort((a, b) => a - b);
      // drops win where cuts crowd: keep a drop's frame, then the earlier of the rest
      const dropFrames = new Set(drops ? map.events.filter((e) => e.kind === "drop").map((e) => frame(e.time)) : []);
      const out: number[] = [];
      for (const f of frames) {
        const last = out[out.length - 1];
        if (last === undefined || f - last >= minGap) out.push(f);
        else if (dropFrames.has(f) && !dropFrames.has(last)) out[out.length - 1] = f;
      }
      return out;
    },
  };
  return s;
}

export interface UseSongOptions {
  /** seconds into the song at frame 0 of the enclosing Sequence */
  offset?: number;
}

/** Where the song is at the current frame (frame 0 of the enclosing Sequence is `offset` seconds into the song), with
 *  the map bound to the composition's fps as `song`. */
export function useSong(map: SongMap, options: UseSongOptions = {}): SongMoment & { song: Song } {
  const f = useCurrentFrame();
  const { fps } = useVideoConfig();
  const s = song(map, { fps, offset: options.offset ?? 0 });
  return { ...s.at(f), song: s };
}
