// The look, cues, the frame and the take.
import { Children, createContext, isValidElement, useContext, type CSSProperties, type ReactElement, type ReactNode } from "react";
import { AbsoluteFill, Audio, Sequence, Video, findPhrase, interpolate, sao, useCurrentFrame, useVideoConfig, useWords, Easing } from "raea";
import { PARTS, layout, spanOf, type Kind, type PartMeta, type Span, type Word } from "./layout";

export type Words = Word[];

// ---- look --------------------------------------------------------------------------------------------------------

export const ACCENT = "var(--brand-accent, #c8ff2e)";

export const STRIKE = "var(--brand-strike, #ff4d3d)";

export const INK = "var(--brand-ink, #ffffff)";

export const CARD = "var(--brand-card, rgba(12,12,12,0.88))";

export const ON_ACCENT = "var(--brand-on-accent, #0d0d0d)";

const HEAD = "var(--brand-font, 'Avenir Next Condensed')";

export const BODY = "var(--brand-font-body, 'Avenir Next')";

export const SHADOW = "0 3px 0 rgba(0,0,0,0.35), 0 0 3px rgba(0,0,0,0.5), 0 10px 36px rgba(0,0,0,0.55)";

/** type straight over the footage (hooks, a strike's lockup, captions), and its shadow: white on a dark grade by
 *  default, the ink of a light screen with a soft white halo when <Ad tone="light"> */
export const OVER = "var(--brand-over, #ffffff)";

const OVER_SHADOW = `var(--brand-over-shadow, ${SHADOW})`;

const LIGHT = { "--brand-over": "#17191d", "--brand-over-shadow": "0 0 4px rgba(255,255,255,0.9), 0 0 24px rgba(255,255,255,0.75)" };

/** the band's top, and its width */
export const TOP = 236;

/** where a band part sits: its top edge for each `place` (top is the reference's, over the head) */
const PLACES = { top: TOP, upper: 420, middle: 760 } as const;

export type Place = keyof typeof PLACES;

/** the top edge of the part being drawn */
export const TopContext = createContext<number>(TOP);

function topOf(place: Place | undefined): number {
  if (place !== undefined && !(place in PLACES)) throw new Error(`place must be "top", "upper" or "middle", got ${JSON.stringify(place)}`);
  return PLACES[place ?? "top"];
}

export const W = 960;

const OUT = Easing.out(Easing.cubic);

export const clamp = { extrapolateLeft: "clamp", extrapolateRight: "clamp" } as const;

/** a value along keys {frame: value}, eased out per segment, held outside them */
export function key(f: number, keys: Record<number, number>): number {
  const xs = Object.keys(keys).map(Number).sort((a, b) => a - b);
  return interpolate(f, xs, xs.map((x) => keys[x]), { easing: OUT, ...clamp });
}

export const POP = { 0: 0.6, 4: 1.08, 8: 1 };

export const rise = (local: number) => key(local, { 0: 30, 6: 0 });

export function type(size: number, color: string = INK, tracking = -2, shadow = true, line = 0.92): CSSProperties {
  return { fontFamily: HEAD, fontWeight: 800, fontSize: size, color, lineHeight: line, textTransform: "uppercase", letterSpacing: tracking, textShadow: shadow ? OVER_SHADOW : undefined, whiteSpace: "nowrap" };
}

/** advance widths in em of Avenir Next Condensed Heavy (what weight 800 resolves to), measured from the font; a
 *  brand's own face is near enough for fitting */
const ADVANCE: Record<string, number> = {
  A: 0.596, B: 0.58, C: 0.53, D: 0.621, E: 0.467, F: 0.454, G: 0.577, H: 0.632, I: 0.324, J: 0.485, K: 0.624, L: 0.465, M: 0.807,
  N: 0.63, O: 0.64, P: 0.562, Q: 0.65, R: 0.582, S: 0.478, T: 0.464, U: 0.6, V: 0.602, W: 0.844, X: 0.596, Y: 0.538, Z: 0.497,
  " ": 0.2, "%": 0.765, "'": 0.254, ".": 0.3, ",": 0.3, "!": 0.35, "?": 0.46, ":": 0.32, ";": 0.32, "-": 0.26, "+": 0.666, "/": 0.32, "&": 0.66,
};

/** a line's width in em, set in capitals */
export const ems = (text: string) => [...text.replace(/\*/g, "").toUpperCase()].reduce((n, c) => n + (ADVANCE[c] ?? (/\d/.test(c) ? 0.56 : 0.6)), 0);

/** the largest size up to `max` at which `text` fits `width`, with the type's -2 px tracking */
export const fit = (text: string, width: number, max: number) => Math.min(max, Math.floor((width + 2 * plain(text).length) / Math.max(1, ems(text))));

/** a line with its *accent* words */
export const accent = (line: string, color: string = ACCENT) =>
  line.split(/(\*[^*]+\*)/).filter(Boolean).map((p, k) => (p.startsWith("*") ? <span key={k} style={{ color }}>{p.slice(1, -1)}</span> : <span key={k}>{p}</span>));

export const lines = (text: string) => text.split(" / ").map((l) => l.trim()).filter(Boolean);

/** A lockup's lines: as given with " / ", or a long line with none broken into two or three lines of even width (the
 *  widest as narrow as it can be), so a one-line hook still reads big. */
export function lockup(text: string): string[] {
  const given = lines(text);
  const ws = text.trim().split(/\s+/);
  if (given.length > 1 || ws.length < 4) return given;
  const n = ws.length > 6 ? 3 : 2;
  let best: string[] = given;
  let widest = Infinity;
  const tryCuts = (from: number, left: number, acc: string[]) => {
    if (left === 1) {
      const ls = [...acc, ws.slice(from).join(" ")];
      const w = Math.max(...ls.map(ems));
      if (w < widest) [best, widest] = [ls, w];
      return;
    }
    for (let k = from + 1; k <= ws.length - left + 1; k++) tryCuts(k, left - 1, [...acc, ws.slice(from, k).join(" ")]);
  };
  tryCuts(0, n, []);
  return best;
}

export const plain = (line: string) => line.replace(/\*/g, "");

// ---- cues --------------------------------------------------------------------------------------------------------

export interface Cue {
  /** the phrase the part lands on, word for word from the transcript */
  say?: string;
  /** which time `say` is said, from 1 */
  occurrence?: number;
  /** the phrase it leaves on (the end of its last word) */
  until?: string;
  /** its length when there's no `until` */
  seconds?: number;
  /** a start frame, when it isn't cued on words */
  from?: number;
  /** the beat of the bed it lands on, counted from 0 (a take with no voice is cut on the music: <Ad beats>) */
  beat?: number;
  /** the last beat it's up for (it leaves as the next beat starts) */
  untilBeat?: number;
  /** foley on its moment: an SAO prompt, or false for none */
  sound?: string | false;
  /** where it sits: "top" (over the head, the default), "upper" or "middle" */
  place?: Place;
}

/** what <Ad> worked out: each part's span (by its place among the children) and the band's spans with their kinds */
interface AdLayout { spans: Map<number, Span>; band: (Span & { kind: Kind; top: number })[] }

export const AdContext = createContext<AdLayout | null>(null);

/** the bed's beats in the cut's frames, from <Ad beats> */
const BeatsContext = createContext<number[]>([]);

const SpanContext = createContext<Span | null>(null);

export const META = new WeakMap<object, PartMeta>();

export function part<P>(name: string, meta: PartMeta, Body: (p: P & { span: Span }) => ReactNode) {
  const Part = (p: P & Cue) => {
    // a prop the part can't go without is named, with the props it was given (a wrong name reads as a missing one)
    const missing = (meta.needs ?? []).filter((k) => typeof (p as Record<string, unknown>)[k] !== "string");
    if (missing.length) throw new Error(`<${name}> needs ${missing.map((k) => `${k}="..."`).join(" and ")} (it was given ${Object.keys(p).join(", ") || "nothing"})`);
    const given = useContext(SpanContext);
    const words = useWords();
    const beats = useContext(BeatsContext);
    const { fps, durationInFrames } = useVideoConfig();
    const span = given ?? spanOf(findPhrase, name, p, meta, words, beats, fps, durationInFrames);
    return (
      <Sequence from={span.from} durationInFrames={span.to - span.from} name={name.toLowerCase()} layout="none">
        <TopContext.Provider value={topOf(p.place)}>
          <Body {...p} span={span} />
        </TopContext.Provider>
      </Sequence>
    );
  };
  Part.displayName = name;
  META.set(Part, meta);
  return Part;
}

/** The SAO hit on a part's moment (frames local to the part). */
/** the foley a part makes in each kit: the talking reel's under a voice, and a screen piece's (golden 30's: a round
 *  thud for a landing, a cartoon pop for a card, louder, as nothing is said over them) */
type Foley = [prompt: string, gain: number, seconds: number];

export const THUD: Foley = ["deep round thud, a heavy object dropping onto something soft, short, no reverb", -5, 1];

export const BLIP: Foley = ["bubbly cartoon UI pop, satisfying and clean, short", -5, 0.6];

const KitContext = createContext<"voice" | "screen">("voice");

/** The SAO hit on a part's moment (frames local to the part): `sound` when given, else the kit's. */
export function Hit({ at = 0, sound, fallback, gain, seconds = 1, screen }: { at?: number; sound: string | false | undefined; fallback: string; gain: number; seconds?: number; screen?: Foley }) {
  const kit = useContext(KitContext);
  if (sound === false) return null;
  const [prompt, db, len] = sound === undefined && kit === "screen" && screen ? screen : [sound ?? fallback, gain, seconds];
  return (
    <Sequence from={at} name="foley" layout="none">
      <Audio src={sao(prompt, { seconds: len })} gainDb={db} />
    </Sequence>
  );
}

// ---- the frame ---------------------------------------------------------------------------------------------------

export interface AdProps {
  children: ReactNode;
  /** the brand's accent (lime by default), its strike colour, and its typeface for the big type */
  accent?: string;
  strike?: string;
  font?: string;
  /** the dark grade under the top band, so type reads on any footage (on by default) */
  shade?: boolean;
  /** the footage's tone: "dark" (the default; white type on a dark grade) or "light" (a bright screen: dark type,
   *  no grade); the host gives TONE */
  tone?: "dark" | "light";
  /** the parts' foley: "voice" (the talking reel's, quiet under a voice) or "screen" (thuds and pops at the level of
   *  a piece with no voice, and a thud on each quick cut); "screen" when the piece is cued on beats */
  kit?: "voice" | "screen";
  /** the bed's beats in the cut's frames, for parts cued by `beat` (a take with no voice; the host gives BEATS) */
  beats?: number[];
}

/** The frame a take's parts sit in: works out every part's span from the words, gives the top band to one part at a
 *  time (each ends where the next starts), and tells Captions where the band is busy. */
export function Ad({ children, accent: a, strike, font, tone = "dark", shade = tone === "dark", beats = [], kit = beats.length ? "screen" : "voice" }: AdProps) {
  const words = useWords();
  const { fps, durationInFrames } = useVideoConfig();
  const items = Children.toArray(children).filter(isValidElement) as ReactElement<Cue>[];
  // the parts' spans are layout.ts's, the same the feed's host lays the parts lane out with
  const nameOf = (el: ReactElement) => Object.keys(PARTS).find((k) => PARTS[k] === META.get(el.type as object));
  const placed = items.flatMap((el, i) => { const name = nameOf(el); return name ? [{ i, name, cue: el.props }] : []; });
  const laid = layout(findPhrase, placed, words, beats, fps, durationInFrames);
  const spans = laid.spans;
  const band: (Span & { kind: Kind; top: number; i: number })[] = laid.band.map((b) => ({ ...b, top: topOf(items[b.i].props.place) }));
  const vars = { ...(tone === "light" ? LIGHT : {}), "--brand-accent": a, "--brand-strike": strike, "--brand-font": font } as Record<string, string | undefined>;
  const style = Object.fromEntries(Object.entries(vars).filter(([, v]) => v)) as CSSProperties;
  return (
    <KitContext.Provider value={kit}>
    <BeatsContext.Provider value={beats}>
    <AdContext.Provider value={{ spans, band }}>
      <AbsoluteFill style={{ backgroundColor: "#000", ...style }}>
        {items.map((el, i) => {
          const meta = META.get(el.type as object);
          const node = meta?.kind === "take" && shade ? [el, <Shade key="shade" />] : el;
          return spans.has(i) ? <SpanContext.Provider key={i} value={spans.get(i)!}>{node}</SpanContext.Provider> : <AdItem key={i}>{node}</AdItem>;
        })}
      </AbsoluteFill>
    </AdContext.Provider>
    </BeatsContext.Provider>
    </KitContext.Provider>
  );
}

const AdItem = ({ children }: { children: ReactNode }) => <>{children}</>;

/** The dark grade behind the top band. */
export function Shade() {
  return <AbsoluteFill style={{ backgroundImage: "linear-gradient(180deg, rgba(20,16,12,0.55) 0%, rgba(20,16,12,0.35) 24%, rgba(20,16,12,0) 52%)" }} />;
}

// ---- the take ----------------------------------------------------------------------------------------------------

export interface TakeProps {
  src: string;
  /** the take's kept stretches in its own seconds, [from, to], played one after another (a tight cut) */
  segments?: [number, number][];
  /** where the face is in the take, as shares of its picture (x across, y down); the zoom holds it in place */
  face?: { x: number; y: number };
  /** the zoom on alternate cuts: [the even cuts', the odd cuts'] (a punch-in on every other cut) */
  punch?: [number, number];
  /** the voice's gain in dB */
  gain?: number;
  /** per segment, where to look and how close, in place of `face` and `punch`: {x, y} shares of the picture held
   *  still, `zoom` 1 for the whole picture (a screen recording's shots: the host gives them) */
  shots?: { x?: number; y?: number; zoom?: number }[];
}

/** A talking take, cut to its segments, each pushing in slowly, the zoom punching in on every other cut. */
function TakeBody({ src, segments, face = { x: 0.5, y: 0.35 }, punch = [1.12, 1.26], gain = 0, shots }: TakeProps) {
  const { fps, durationInFrames } = useVideoConfig();
  const kit = useContext(KitContext);
  const segs = segments ?? [[0, durationInFrames / fps]];
  let at = 0;
  return (
    <>
      {segs.map(([a, b], n) => {
        const from = at;
        const len = Math.max(1, Math.round((b - a) * fps));
        at += len;
        return (
          <Sequence key={n} from={from} durationInFrames={len} name={`take ${n + 1}`}>
            {/* a screen piece's quick flips (under a second) each land with a thud, softer than a part's */}
            {kit === "screen" && n > 0 && len < fps ? <Hit sound={undefined} fallback={THUD[0]} gain={-12} seconds={1} /> : null}
            <Shot src={src} start={Math.round(a * fps)} len={len} face={shots?.[n] ? { x: shots[n].x ?? 0.5, y: shots[n].y ?? 0.5 } : face} zoom={shots?.[n] ? shots[n].zoom ?? 1 : punch[n % 2]} gain={gain} />
          </Sequence>
        );
      })}
    </>
  );
}

function Shot({ src, start, len, face, zoom, gain }: { src: string; start: number; len: number; face: { x: number; y: number }; zoom: number; gain: number }) {
  const f = useCurrentFrame();
  const { width, height } = useVideoConfig();
  const z = interpolate(f, [0, len], [zoom, zoom + 0.035], clamp);
  // cover with the face's point at its own share of the box; the zoom is about the centre, so it's shifted back to
  // keep that point still
  const tx = (1 - z) * (face.x - 0.5) * width;
  const ty = (1 - z) * (face.y - 0.5) * height;
  return (
    <AbsoluteFill style={{ overflow: "hidden", backgroundColor: "#000" }}>
      <Video src={src} startFrom={start} endAt={start + len} gainDb={gain}
        style={{ position: "absolute", width: "100%", height: "100%", objectFit: "cover", objectPosition: `${face.x * 100}% ${face.y * 100}%`, transform: `translate(${tx}px, ${ty}px) scale(${z})` }} />
    </AbsoluteFill>
  );
}

export const Take = (p: TakeProps) => <TakeBody {...p} />;

META.set(Take, PARTS.Take);
