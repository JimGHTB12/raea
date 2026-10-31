// raea/views: type-free pictures of footage being worked on, for showing an edit happen: a timeline (an NLE's
// tracks, clips, ruler and playhead), a graph of embeddings in 3D, and perception overlays (face brackets, hand
// skeletons, a film strip with its cut lines, a word track, a waveform). Each is a pure function of its props, drawn
// in the page's own DOM and SVG, so it renders the same frame every time and costs no 3D pass. None draws text; what
// the shapes mean is left to what they show. docs/views.md has the guide.
import type { CSSProperties, ReactNode } from "react";
import { Img } from "@raea/video/media";

export type Vec3 = [number, number, number];

// ---- 3D projection -------------------------------------------------------------------------------------------------

/** An orbit camera: it looks at `target` from `distance` away, turned `yaw` about the vertical and tilted `pitch` up
 *  (radians), with a vertical field of view `fov` (degrees). */
export interface OrbitCamera {
  target: Vec3;
  yaw: number;
  pitch: number;
  distance: number;
  fov: number;
}

/** A point on screen: `x`, `y` in pixels, `depth` along the view (world units, > 0 in front), `scale` pixels per world
 *  unit at that depth. */
export interface Projected {
  x: number;
  y: number;
  depth: number;
  scale: number;
}

const sub = (a: Vec3, b: Vec3): Vec3 => [a[0] - b[0], a[1] - b[1], a[2] - b[2]];
const dot = (a: Vec3, b: Vec3) => a[0] * b[0] + a[1] * b[1] + a[2] * b[2];
const cross = (a: Vec3, b: Vec3): Vec3 => [a[1] * b[2] - a[2] * b[1], a[2] * b[0] - a[0] * b[2], a[0] * b[1] - a[1] * b[0]];
const norm = (a: Vec3): Vec3 => {
  const l = Math.hypot(a[0], a[1], a[2]) || 1;
  return [a[0] / l, a[1] / l, a[2] / l];
};

/** Where the orbit camera's eye is. */
export function cameraEye(cam: OrbitCamera): Vec3 {
  const c = Math.cos(cam.pitch);
  return [cam.target[0] + cam.distance * c * Math.sin(cam.yaw), cam.target[1] + cam.distance * Math.sin(cam.pitch), cam.target[2] + cam.distance * c * Math.cos(cam.yaw)];
}

/** A projector for one frame: world point to screen, or null when it is behind the camera (closer than `near`). */
export function projector(cam: OrbitCamera, width: number, height: number, near = 0.05): (p: Vec3) => Projected | null {
  const eye = cameraEye(cam);
  const fwd = norm(sub(cam.target, eye));
  const right = norm(cross(fwd, [0, 1, 0]));
  const up = cross(right, fwd);
  const focal = height / 2 / Math.tan((cam.fov * Math.PI) / 360);
  return (p) => {
    const d = sub(p, eye);
    const z = dot(d, fwd);
    if (z < near) return null;
    const s = focal / z;
    return { x: width / 2 + dot(d, right) * s, y: height / 2 - dot(d, up) * s, depth: z, scale: s };
  };
}

// ---- the graph of embeddings ---------------------------------------------------------------------------------------

export interface GraphNode {
  /** where it sits in the embedding's 3D layout (UMAP, PCA...) */
  pos: Vec3;
  /** a still of what it stands for, drawn on the node */
  src?: string;
  /** its colour: the ring round a still, or the dot's fill */
  color: string;
  /** its size in world units (a still keeps its aspect `aspect`, width over height) */
  size?: number;
  aspect?: number;
  /** 0..1: lit nodes grow, brighten and glow */
  lit?: number;
  /** 0..1, multiplies everything (0 hides it) */
  opacity?: number;
}

export interface GraphEdge {
  a: number;
  b: number;
  /** 0..1 of the edge drawn from a toward b (to draw it in) */
  drawn?: number;
  /** 0..1: a lit edge is thicker and takes `litColor` */
  lit?: number;
  opacity?: number;
}

export interface EmbeddingGraphProps {
  nodes: GraphNode[];
  edges?: GraphEdge[];
  camera: OrbitCamera;
  width: number;
  height: number;
  edgeColor?: string;
  litColor?: string;
  /** depth (world units) over which nodes fade to nothing behind the target; 0 for no fog */
  fog?: number;
  /** world size of a node when it gives none */
  nodeSize?: number;
  style?: CSSProperties;
}

/** Nodes (stills or dots) at their embedding's 3D places, seen through an orbit camera, with edges between them
 *  (nearest neighbours, a query's path). Drawn far to near; edges under the nodes. */
export function EmbeddingGraph({ nodes, edges = [], camera, width, height, edgeColor = "#ffffff", litColor = "#ffd400", fog = 0, nodeSize = 0.08, style }: EmbeddingGraphProps) {
  const proj = projector(camera, width, height);
  const pts = nodes.map((n) => proj(n.pos));
  const far = camera.distance;
  const fade = (depth: number) => (fog > 0 ? Math.max(0, Math.min(1, 1 - (depth - far) / fog)) : 1);
  const lines: ReactNode[] = [];
  edges.forEach((e, i) => {
    const pa = pts[e.a], pb = pts[e.b];
    if (!pa || !pb) return;
    const drawn = e.drawn ?? 1;
    if (drawn <= 0) return;
    const lit = e.lit ?? 0;
    const o = (e.opacity ?? 1) * Math.min(fade(pa.depth), fade(pb.depth)) * Math.min(nodes[e.a].opacity ?? 1, nodes[e.b].opacity ?? 1);
    if (o <= 0.01) return;
    const x2 = pa.x + (pb.x - pa.x) * drawn, y2 = pa.y + (pb.y - pa.y) * drawn;
    const w = Math.max(1, Math.min(pa.scale, pb.scale) * 0.008) * (1 + 2.5 * lit);
    lines.push(<line key={i} x1={pa.x} y1={pa.y} x2={x2} y2={y2} stroke={lit > 0.5 ? litColor : edgeColor} strokeWidth={w} strokeOpacity={o * (0.35 + 0.65 * lit)} strokeLinecap="round" />);
  });
  const order = nodes.map((_, i) => i).filter((i) => pts[i]).sort((a, b) => pts[b]!.depth - pts[a]!.depth);
  return (
    <div style={{ position: "absolute", left: 0, top: 0, width, height, overflow: "hidden", ...style }}>
      <svg width={width} height={height} style={{ position: "absolute", left: 0, top: 0 }}>{lines}</svg>
      {order.map((i) => {
        const n = nodes[i], p = pts[i]!;
        const lit = n.lit ?? 0;
        const o = (n.opacity ?? 1) * fade(p.depth);
        if (o <= 0.01) return null;
        const size = (n.size ?? nodeSize) * p.scale * (1 + 0.8 * lit);
        const a = n.aspect ?? 1;
        const w = a >= 1 ? size : size * a, h = a >= 1 ? size / a : size;
        if (w < 0.5) return null;
        const ring = Math.max(1, size * 0.06);
        const box: CSSProperties = {
          position: "absolute", left: p.x - w / 2, top: p.y - h / 2, width: w, height: h, opacity: o,
          boxShadow: lit > 0 ? `0 0 ${size * 0.5 * lit}px ${litColor}` : undefined,
        };
        return n.src ? (
          <div key={i} style={{ ...box, border: `${ring}px solid ${lit > 0.5 ? litColor : n.color}`, overflow: "hidden", background: n.color }}>
            <Img src={n.src} style={{ width: "100%", height: "100%", objectFit: "cover" }} />
          </div>
        ) : (
          <div key={i} style={{ ...box, borderRadius: w, background: lit > 0.5 ? litColor : n.color }} />
        );
      })}
    </div>
  );
}

// ---- the timeline --------------------------------------------------------------------------------------------------

export interface TimelineClip {
  /** where it sits on the timeline, seconds */
  from: number;
  to: number;
  color: string;
  /** stills drawn along it, evenly (a film strip) */
  thumbs?: string[];
  /** a waveform drawn along it: peaks 0..1, evenly over its length */
  wave?: number[];
  /** upside down (a flip) */
  flip?: boolean;
  /** pixels it is lifted off its track (dragged), and sideways */
  lift?: number;
  shift?: number;
  /** 0..1: an outline and a glow */
  selected?: number;
  /** times on the timeline drawn as small notches in the clip (word boundaries, beats inside it) */
  marks?: number[];
  opacity?: number;
}

export interface TimelineTrack {
  /** a track's height in pixels */
  height: number;
  clips: TimelineClip[];
}

export interface TimelineProps {
  width: number;
  height: number;
  /** the seconds shown, left edge to right edge (zoom and scroll) */
  view: [number, number];
  tracks: TimelineTrack[];
  /** the playhead, seconds (off the view: hidden) */
  playhead?: number;
  /** ruler ticks: beats (short) and downbeats (tall), seconds on the timeline; `flash` 0..1 per beat lights it */
  beats?: number[];
  downbeats?: number[];
  flash?: (beatIndex: number) => number;
  /** the ruler's height in pixels */
  ruler?: number;
  /** a blade: a vertical line at this time, cutting through every track */
  blade?: number;
  theme?: Partial<TimelineTheme>;
  style?: CSSProperties;
}

export interface TimelineTheme {
  background: string;
  track: string;
  ruler: string;
  tick: string;
  playhead: string;
  blade: string;
  gap: number;
  radius: number;
}

const TIMELINE_THEME: TimelineTheme = { background: "#0b0b0d", track: "#17171b", ruler: "#0b0b0d", tick: "#f3ead8", playhead: "#ffd400", blade: "#ff3b1f", gap: 6, radius: 6 };

/** An editor's timeline with no words on it: a ruler of beats, tracks of clips (film strips or waveforms), notches,
 *  a playhead and a blade, laid out over `view` seconds. */
export function Timeline({ width, height, view, tracks, playhead, beats = [], downbeats = [], flash, ruler = 28, blade, theme, style }: TimelineProps) {
  const t = { ...TIMELINE_THEME, ...theme };
  const [v0, v1] = view;
  const px = (s: number) => ((s - v0) / (v1 - v0)) * width;
  const inView = (s: number) => s >= v0 - 1 && s <= v1 + 1;
  const ticks: ReactNode[] = [];
  const down = new Set(downbeats.map((d) => Math.round(d * 1000)));
  beats.forEach((b, i) => {
    if (!inView(b)) return;
    const big = down.has(Math.round(b * 1000));
    const f = flash ? flash(i) : 0;
    const h = (big ? 0.8 : 0.4) * ruler * (1 + 0.25 * f);
    ticks.push(<div key={`b${i}`} style={{ position: "absolute", left: px(b) - (big ? 1.5 : 1), top: ruler - h, width: big ? 3 : 2, height: h, background: t.tick, opacity: 0.5 + 0.5 * Math.max(f, big ? 0.6 : 0) }} />);
    if (f > 0) ticks.push(<div key={`g${i}`} style={{ position: "absolute", left: px(b) - 1, top: ruler, width: 2, height: height - ruler, background: t.tick, opacity: 0.25 * f }} />);
  });
  let y = ruler + t.gap;
  const rows: ReactNode[] = [];
  tracks.forEach((tr, ti) => {
    rows.push(<div key={`t${ti}`} style={{ position: "absolute", left: 0, top: y, width, height: tr.height, background: t.track }} />);
    tr.clips.forEach((c, ci) => {
      const x0 = px(c.from) + (c.shift ?? 0), x1 = px(c.to) + (c.shift ?? 0);
      if (x1 < -50 || x0 > width + 50) return;
      const w = Math.max(2, x1 - x0 - 2);
      const sel = c.selected ?? 0;
      const top = y - (c.lift ?? 0);
      const inner: ReactNode[] = [];
      if (c.thumbs && c.thumbs.length) {
        const tw = tr.height * 0.6;
        const n = Math.max(1, Math.ceil(w / tw));
        for (let k = 0; k < n; k++) {
          const src = c.thumbs[Math.min(c.thumbs.length - 1, Math.floor((k / n) * c.thumbs.length))];
          inner.push(<Img key={k} src={src} style={{ position: "absolute", left: k * tw, top: 0, width: tw - 1, height: tr.height, objectFit: "cover" }} />);
        }
      }
      if (c.wave && c.wave.length) {
        const n = Math.max(1, Math.floor(w / 4));
        for (let k = 0; k < n; k++) {
          const v = c.wave[Math.min(c.wave.length - 1, Math.floor((k / n) * c.wave.length))];
          const h = Math.max(1, v * tr.height * 0.86);
          inner.push(<div key={`w${k}`} style={{ position: "absolute", left: k * 4 + 1, top: (tr.height - h) / 2, width: 2.5, height: h, background: "rgba(11,11,13,0.75)" }} />);
        }
      }
      (c.marks ?? []).forEach((m, mi) => {
        const mx = px(m) + (c.shift ?? 0) - x0;
        if (mx < 0 || mx > w) return;
        inner.push(<div key={`m${mi}`} style={{ position: "absolute", left: mx - 1, top: tr.height - tr.height * 0.28, width: 2, height: tr.height * 0.28, background: t.tick }} />);
      });
      rows.push(
        <div key={`c${ti}-${ci}`} style={{
          position: "absolute", left: x0 + 1, top, width: w, height: tr.height, overflow: "hidden", borderRadius: t.radius, background: c.color,
          opacity: c.opacity ?? 1, transform: c.flip ? "scaleY(-1)" : undefined,
          boxShadow: sel > 0 ? `0 0 0 ${3 * sel}px ${t.playhead}, 0 ${10 * sel}px ${30 * sel}px rgba(0,0,0,0.6)` : undefined,
        }}>
          {inner}
          <div style={{ position: "absolute", left: 0, top: 0, width: 5, height: tr.height, background: c.color }} />
          <div style={{ position: "absolute", left: w - 5, top: 0, width: 5, height: tr.height, background: c.color }} />
        </div>,
      );
    });
    y += tr.height + t.gap;
  });
  return (
    <div style={{ position: "absolute", left: 0, top: 0, width, height, overflow: "hidden", background: t.background, ...style }}>
      <div style={{ position: "absolute", left: 0, top: 0, width, height: ruler, background: t.ruler }} />
      {rows}
      {ticks}
      {blade !== undefined && inView(blade) ? <div style={{ position: "absolute", left: px(blade) - 2, top: 0, width: 4, height, background: t.blade, boxShadow: `0 0 18px ${t.blade}` }} /> : null}
      {playhead !== undefined && inView(playhead) ? (
        <>
          <div style={{ position: "absolute", left: px(playhead) - 1.5, top: 0, width: 3, height, background: t.playhead }} />
          <div style={{ position: "absolute", left: px(playhead) - 9, top: 0, width: 18, height: ruler * 0.6, background: t.playhead, borderRadius: "0 0 9px 9px" }} />
        </>
      ) : null}
    </div>
  );
}

// ---- perception overlays -------------------------------------------------------------------------------------------

export interface FaceBoxProps {
  /** the box in pixels: x, y, w, h */
  box: [number, number, number, number];
  /** 0..1 the lock: brackets fly in from 1.6x and snap on with a small overshoot */
  lock?: number;
  color?: string;
  thick?: number;
  /** each bracket arm as a share of the box's shorter side */
  arm?: number;
}

/** Corner brackets that lock onto a box (a face, a person, a product). */
export function FaceBox({ box, lock = 1, color = "#ffd400", thick = 4, arm = 0.28 }: FaceBoxProps) {
  if (lock <= 0) return null;
  const k = Math.min(1, lock);
  const over = 1 + 0.6 * Math.pow(1 - k, 2) - 0.06 * Math.sin(k * Math.PI);
  const [bx, by, bw, bh] = box;
  const cx = bx + bw / 2, cy = by + bh / 2, w = bw * over, h = bh * over;
  const a = Math.min(w, h) * arm;
  const x0 = cx - w / 2, y0 = cy - h / 2;
  const s = (l: number, t: number, ww: number, hh: number, key: string) => <div key={key} style={{ position: "absolute", left: l, top: t, width: ww, height: hh, background: color }} />;
  return (
    <div style={{ position: "absolute", left: 0, top: 0, opacity: Math.min(1, k * 3) }}>
      {s(x0, y0, a, thick, "a")}{s(x0, y0, thick, a, "b")}
      {s(x0 + w - a, y0, a, thick, "c")}{s(x0 + w - thick, y0, thick, a, "d")}
      {s(x0, y0 + h - thick, a, thick, "e")}{s(x0, y0 + h - a, thick, a, "f")}
      {s(x0 + w - a, y0 + h - thick, a, thick, "g")}{s(x0 + w - thick, y0 + h - a, thick, a, "h")}
    </div>
  );
}

/** MediaPipe's 21-point hand: the bones as pairs of point indices. */
export const HAND_BONES: [number, number][] = [
  [0, 1], [1, 2], [2, 3], [3, 4], [0, 5], [5, 6], [6, 7], [7, 8], [5, 9], [9, 10], [10, 11], [11, 12],
  [9, 13], [13, 14], [14, 15], [15, 16], [13, 17], [0, 17], [17, 18], [18, 19], [19, 20],
];

export interface HandSkeletonProps {
  /** the 21 points in pixels */
  points: [number, number][];
  color?: string;
  /** 0..1 of the bones drawn (they grow from the wrist out) */
  drawn?: number;
  width: number;
  height: number;
  thick?: number;
}

/** A hand's skeleton: its bones and joints. */
export function HandSkeleton({ points, color = "#ffd400", drawn = 1, width, height, thick = 3 }: HandSkeletonProps) {
  if (points.length < 21 || drawn <= 0) return null;
  return (
    <svg width={width} height={height} style={{ position: "absolute", left: 0, top: 0 }}>
      {HAND_BONES.map(([a, b], i) => {
        const k = Math.max(0, Math.min(1, drawn * 1.4 - (i / HAND_BONES.length) * 0.4));
        if (k <= 0) return null;
        const [x1, y1] = points[a], [x2, y2] = points[b];
        return <line key={i} x1={x1} y1={y1} x2={x1 + (x2 - x1) * k} y2={y1 + (y2 - y1) * k} stroke={color} strokeWidth={thick} strokeLinecap="round" />;
      })}
      {points.map(([x, y], i) => (drawn >= 0.5 ? <circle key={`p${i}`} cx={x} cy={y} r={thick * (i % 4 === 0 ? 1.6 : 1.1)} fill={color} /> : null))}
    </svg>
  );
}

export interface CutStripProps {
  /** the strip's stills, one per strip frame */
  frames: string[];
  /** the strip frame at the left edge (fractional scrolls) */
  scroll: number;
  /** a strip frame's width and the strip's height, pixels */
  frameWidth: number;
  height: number;
  width: number;
  /** the cuts, as strip positions (a cut between frames k and k+1 is k + 1), each with its snap 0..1 */
  cuts: { at: number; snap: number }[];
  /** a per-frame probability drawn as bars under the strip (TransNet's shot-end score); `probHeight` tall */
  prob?: number[];
  probHeight?: number;
  color?: string;
  barColor?: string;
}

/** A film strip scrolling past, with each cut's line snapping across it (and a spike under it when `prob` is given). */
export function CutStrip({ frames, scroll, frameWidth, height, width, cuts, prob, probHeight = 0, color = "#ff3b1f", barColor = "#f3ead8" }: CutStripProps) {
  const first = Math.max(0, Math.floor(scroll) - 1), last = Math.min(frames.length, Math.ceil(scroll + width / frameWidth) + 1);
  const x = (k: number) => (k - scroll) * frameWidth;
  const items: ReactNode[] = [];
  for (let k = first; k < last; k++) {
    items.push(<Img key={k} src={frames[k]} style={{ position: "absolute", left: x(k), top: 0, width: frameWidth - 2, height, objectFit: "cover" }} />);
    if (prob && probHeight > 0) {
      const v = prob[k] ?? 0;
      items.push(<div key={`p${k}`} style={{ position: "absolute", left: x(k) + 2, top: height + probHeight * (1 - v) + 4, width: frameWidth - 6, height: Math.max(2, probHeight * v), background: v > 0.5 ? color : barColor, opacity: v > 0.5 ? 1 : 0.5 }} />);
    }
  }
  cuts.forEach((c, i) => {
    if (c.snap <= 0) return;
    const cx = x(c.at);
    if (cx < -20 || cx > width + 20) return;
    const k = Math.min(1, c.snap);
    const len = (height + probHeight + 8) * (k < 1 ? 1 - Math.pow(1 - k, 3) : 1);
    items.push(<div key={`c${i}`} style={{ position: "absolute", left: cx - 3, top: 0, width: 6, height: len, background: color, boxShadow: `0 0 ${14 * (2 - k)}px ${color}` }} />);
    if (k < 1) items.push(<div key={`f${i}`} style={{ position: "absolute", left: cx - frameWidth / 2, top: 0, width: frameWidth, height, background: color, opacity: 0.45 * (1 - k) }} />);
  });
  return <div style={{ position: "absolute", left: 0, top: 0, width, height: height + probHeight + 8, overflow: "hidden" }}>{items}</div>;
}

export interface WordSpan {
  start: number;
  end: number;
  text?: string;
}

export interface WordTrackProps {
  words: WordSpan[];
  /** now, seconds on the words' clock */
  time: number;
  /** seconds shown across `width` (the track scrolls to keep `time` at `at` of the width) */
  span: number;
  at?: number;
  width: number;
  height: number;
  color?: string;
  lit?: string;
}

/** Words as blocks on a time line, each as long as it is said; the one being said lights, those said stay bright. */
export function WordTrack({ words, time, span, at = 0.35, width, height, color = "#f3ead8", lit = "#ffd400" }: WordTrackProps) {
  const t0 = time - span * at;
  const x = (s: number) => ((s - t0) / span) * width;
  return (
    <div style={{ position: "absolute", left: 0, top: 0, width, height, overflow: "hidden" }}>
      {words.map((w, i) => {
        const x0 = x(w.start), x1 = x(w.end);
        if (x1 < 0 || x0 > width) return null;
        const now = time >= w.start && time < w.end;
        const said = time >= w.end;
        return <div key={i} style={{ position: "absolute", left: x0, top: now ? 0 : height * 0.2, width: Math.max(3, x1 - x0 - 3), height: now ? height : height * 0.6, borderRadius: height * 0.12, background: now ? lit : color, opacity: now ? 1 : said ? 0.85 : 0.3 }} />;
      })}
      <div style={{ position: "absolute", left: width * at - 1, top: 0, width: 2, height, background: lit }} />
    </div>
  );
}

export interface WaveformProps {
  /** peaks 0..1 */
  peaks: number[];
  width: number;
  height: number;
  /** bar pitch in pixels */
  bar?: number;
  color?: string;
  /** 0..1 of the way along: bars before it take `playedColor` */
  played?: number;
  playedColor?: string;
}

/** A mirrored bar waveform. */
export function Waveform({ peaks, width, height, bar = 6, color = "#f3ead8", played = 0, playedColor = "#ffd400" }: WaveformProps) {
  const n = Math.max(1, Math.floor(width / bar));
  return (
    <div style={{ position: "absolute", left: 0, top: 0, width, height }}>
      {Array.from({ length: n }, (_, k) => {
        const v = peaks.length ? peaks[Math.min(peaks.length - 1, Math.floor((k / n) * peaks.length))] : 0;
        const h = Math.max(2, v * height);
        return <div key={k} style={{ position: "absolute", left: k * bar, top: (height - h) / 2, width: Math.max(1, bar * 0.55), height: h, borderRadius: bar, background: k / n < played ? playedColor : color }} />;
      })}
    </div>
  );
}

/** The nearest of `grid` (ascending seconds) to `t`, and how far it moved: snapping a clip's edge to a beat. */
export function snapTo(grid: readonly number[], t: number): { at: number; moved: number } {
  let best = t, d = Infinity;
  for (const g of grid) {
    const e = Math.abs(g - t);
    if (e < d) {
      d = e;
      best = g;
    }
  }
  return { at: best, moved: best - t };
}
