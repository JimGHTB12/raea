// A song's map and the moment at a time.

export type SongEventKind = "build" | "drop" | "stop" | "hit" | "dip" | "break";

export type SongLevel = "intro" | "low" | "mid" | "high" | "outro";

/** A hit inside a stop: `time` on the beat grid (a beat or half beat), `at` where the onset is. */
export interface SongHit {
  /** seconds from the song's start, snapped to the beat grid (a beat or half beat) */
  time: number;
  /** seconds from the song's start where the onset itself is */
  at: number;
  /** how hard the hit is, 0..1 */
  strength: number;
}

export interface SongEvent {
  /** what the event is: build, drop, stop, hit, dip or break */
  kind: SongEventKind;
  /** seconds from the song's start: a beat (drop, a downbeat) or the grid's nearest beat or half beat (hit) */
  time: number;
  /** spans (build, stop, dip, break) end here */
  end?: number;
  /** the index of the beat it starts on, in `SongMap.beats` */
  beat: number;
  /** the index of the bar it starts in */
  bar: number;
  /** a span: how many beats it lasts */
  beats?: number;
  /** 0..1 */
  strength: number;
  /** a build or stop: the drop it leads into, seconds */
  into?: number | null;
  /** a stop: its sparse hits */
  hits?: SongHit[];
  /** a drop: the energy of the bar after minus the bar before (negative when the band cuts after one big hit) */
  rise?: number;
  /** a drop: the bar after stays up (false: one hit, then the band cuts out) */
  holds?: boolean;
  /** a drop: it comes straight out of a stop */
  afterStop?: boolean;
  /** a drop: it lands at the end of a build (within two bars of it) */
  afterBuild?: boolean;
  /** a drop: a breath of near-silence just before it */
  gapBefore?: boolean;
  /** a hit: the onset's own time; how far over the groove it stands */
  at?: number;
  /** a hit: how far over the groove it stands */
  over?: number;
}

export interface SongSection {
  /** the section's marker name when the song file has markers, else its level */
  name: string;
  /** how high the section sits in the song: intro, low, mid, high or outro */
  level: SongLevel;
  /** seconds from the song's start where the section begins */
  time: number;
  /** seconds from the song's start where it ends */
  end: number;
  /** the index of its first beat */
  beat: number;
  /** the index of its first bar */
  bar: number;
  /** its mean energy, 0..1 */
  energy: number;
  /** suggested cuts per bar: 0.25, 0.5, 1, 2 or 4 */
  cuts: number;
}

export interface SongBar {
  /** seconds from the song's start where the bar begins (its downbeat) */
  time: number;
  /** the index of its first beat */
  beat: number;
  /** how many beats it holds */
  beats: number;
  /** its mean energy, 0..1 */
  energy: number;
  /** suggested cuts in this bar: 0.25, 0.5, 1, 2 or 4 */
  cuts: number;
}

export interface SongFeatures {
  /** loudness (RMS) a beat, 0..1 over the song */
  loudness: number[];
  /** onset strength (spectral flux) a beat, 0..1: how much is struck */
  onsets: number[];
  /** brightness (spectral centroid) a beat, 0..1 */
  brightness: number[];
  /** the low end (kick and bass, under 150 Hz) a beat, 0..1 */
  low: number[];
  /** the high band (over 3 kHz) a beat, 0..1 */
  high: number[];
  /** the combined energy a beat, 0..1: loudness, low end, onsets and highs together */
  energy: number[];
}

export interface SongMap {
  /** the map format's version (1) */
  version: number;
  /** the song's length in seconds */
  duration: number;
  /** beats per minute, or null when no beat was found */
  tempo: number | null;
  /** beats in a bar (4 for 4/4) */
  beatsPerBar: number;
  /** every beat's time in seconds, ascending */
  beats: number[];
  /** every bar's first beat, in seconds */
  downbeats: number[];
  /** one value a beat, 0..1 over the song */
  features: SongFeatures;
  /** the bars in order, each with its energy and suggested cut rate */
  bars: SongBar[];
  /** the song's sections in order (intro, verse, chorus and so on) */
  sections: SongSection[];
  /** builds, drops, stops, hits, dips and breaks, in time order */
  events: SongEvent[];
  /** where the sections came from: the song file's markers, or novelty in the music itself */
  sectionsFrom?: "markers" | "novelty";
  /** the candidate section boundaries novelty found, each with its strength 0..1 */
  boundaries?: { time: number; beat: number; bar: number; strength: number }[];
  /** the song file's name */
  source?: string | null;
}

/** Where a moment of the song is. Seconds are the song's; infinities when there's no such event. */
export interface SongMoment {
  /** the moment, seconds from the song's start */
  time: number;
  /** the beat at or before `time` (-1 before the first) */
  beat: number;
  /** the bar at or before `time` (-1 before the first beat) */
  bar: number;
  /** 0-based position in the bar */
  beatInBar: number;
  /** 0..1 through the beat, and through the bar */
  phase: number;
  /** 0..1 through the bar */
  barPhase: number;
  /** the features at `time`, interpolated between beats */
  energy: number;
  /** loudness at `time`, 0..1 */
  loudness: number;
  /** onset strength at `time`, 0..1 */
  onsets: number;
  /** brightness at `time`, 0..1 */
  brightness: number;
  /** the low end at `time`, 0..1 */
  low: number;
  /** the section `time` is in (null before the first) */
  section: SongSection | null;
  /** that section's index in `SongMap.sections` (-1 for none) */
  sectionIndex: number;
  /** 0..1 through the section */
  sectionProgress: number;
  /** the bar's suggested cut rate (cuts per bar) */
  cutsPerBar: number;
  /** the last drop at or before `time`, the next after it */
  drop: SongEvent | null;
  /** the next drop after `time` */
  nextDrop: SongEvent | null;
  /** seconds since the last drop (Infinity when there's none) */
  sinceDrop: number;
  /** seconds until the next drop (Infinity when there's none) */
  untilDrop: number;
  /** the build `time` is in, and 0..1 through it */
  build: SongEvent | null;
  /** whether `time` is inside a build */
  inBuild: boolean;
  /** 0..1 through that build */
  buildProgress: number;
  /** the stop `time` is in */
  stop: SongEvent | null;
  /** whether `time` is inside a stop */
  inStop: boolean;
  /** the dip or break `time` is in */
  dip: SongEvent | null;
  /** the next event of any kind after `time` */
  next: SongEvent | null;
}

/** The last index i with v[i] <= t (-1 when t is before v[0]); v ascending. */
export function floorIndex(v: readonly number[], t: number): number {
  let lo = 0;
  let hi = v.length - 1;
  if (hi < 0 || t < v[0]) return -1;
  while (lo < hi) {
    const mid = (lo + hi + 1) >> 1;
    if (v[mid] <= t) lo = mid;
    else hi = mid - 1;
  }
  return lo;
}

export function check(map: SongMap): void {
  if (!map || !Array.isArray(map.beats) || !map.features || !Array.isArray(map.events)) {
    throw new TypeError("song: not a song map (make one with `raea song FILE -o song.json`)");
  }
}

function featureAt(v: readonly number[] | undefined, i: number, phase: number): number {
  if (!v || v.length === 0) return 0;
  if (i < 0) return v[0];
  const a = v[Math.min(i, v.length - 1)];
  const b = v[Math.min(i + 1, v.length - 1)];
  return a + (b - a) * phase;
}

const SPANS = new Set<SongEventKind>(["build", "stop", "dip", "break"]);

/** Where the song is at `t` seconds: a pure function of the map. */
export function songAt(map: SongMap, t: number): SongMoment {
  check(map);
  const beats = map.beats;
  const n = beats.length;
  const bpb = map.beatsPerBar || 4;
  const gap = map.tempo ? 60 / map.tempo : n > 1 ? (beats[n - 1] - beats[0]) / (n - 1) : 0.5;
  const beat = floorIndex(beats, t);
  const beatEnd = beat < 0 ? beats[0] ?? gap : beat + 1 < n ? beats[beat + 1] : beats[beat] + gap;
  const beatStart = beat < 0 ? beatEnd - gap : beats[beat];
  const phase = Math.min(1, Math.max(0, (t - beatStart) / Math.max(1e-9, beatEnd - beatStart)));
  const barStarts = map.bars.map((b) => b.time);
  const bar = floorIndex(barStarts, t);
  const barRow = bar >= 0 ? map.bars[bar] : null;
  const beatInBar = barRow && beat >= 0 ? beat - barRow.beat : beat < 0 ? 0 : beat % bpb;
  const barLen = barRow ? barRow.beats : bpb;
  const barPhase = Math.min(1, Math.max(0, (beatInBar + phase) / Math.max(1, barLen)));
  const f = map.features;
  const sectionIndex = floorIndex(map.sections.map((s) => s.time), t);
  const section = sectionIndex >= 0 ? map.sections[sectionIndex] : null;
  const sectionProgress = section ? Math.min(1, Math.max(0, (t - section.time) / Math.max(1e-9, section.end - section.time))) : 0;
  const drops = map.events.filter((e) => e.kind === "drop");
  const di = floorIndex(drops.map((d) => d.time), t);
  const drop = di >= 0 ? drops[di] : null;
  const nextDrop = drops[di + 1] ?? null;
  const within = (k: SongEventKind | SongEventKind[]) =>
    map.events.find((e) => (Array.isArray(k) ? k.includes(e.kind) : e.kind === k) && e.end !== undefined && t >= e.time && t < e.end) ?? null;
  const build = within("build");
  const stop = within("stop");
  const dip = within(["dip", "break"]);
  const next = map.events.find((e) => e.time > t) ?? null;
  return {
    time: t,
    beat,
    bar,
    beatInBar,
    phase,
    barPhase,
    energy: featureAt(f.energy, beat, phase),
    loudness: featureAt(f.loudness, beat, phase),
    onsets: featureAt(f.onsets, beat, phase),
    brightness: featureAt(f.brightness, beat, phase),
    low: featureAt(f.low, beat, phase),
    section,
    sectionIndex,
    sectionProgress,
    cutsPerBar: barRow ? barRow.cuts : section ? section.cuts : 0.25,
    drop,
    nextDrop,
    sinceDrop: drop ? t - drop.time : Infinity,
    untilDrop: nextDrop ? nextDrop.time - t : Infinity,
    build,
    inBuild: build !== null,
    buildProgress: build && build.end !== undefined ? (t - build.time) / Math.max(1e-9, build.end - build.time) : 0,
    stop,
    inStop: stop !== null,
    dip,
    next,
  };
}
