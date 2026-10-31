// raea/arena: the frame a game is drawn in, and the words on it (docs/handoff/feed.md, games). The field is
// drawn in the game's own units and scaled whole into the stage's middle, the way bench Pong scales its court, so it
// plays at any stage size; the score, a banner and the controls hint each have a band of their own, sized from the
// stage, so no word ever lands on another or on the score.
//
//   <Arena width={1600} height={900}
//     score={<Score items={[{ label: "you", value: g.you, accent: true }, { label: "rally", value: g.rally }, { label: "house", value: g.house }]} />}
//     banner={g.phase === "serve" ? <Banner text="space to serve" sub="first to 5" /> : null}
//     hint={<Hint keys={[{ keys: "↑ ↓", does: "move" }, { keys: "space", does: "serve" }]} />}>
//     <div style={{ position: "absolute", left: g.x * 1600, top: 120, width: 40, height: 200, background: "var(--brand-accent)" }} />
//   </Arena>
//
// Children are positioned in the field's units (0..width across, 0..height down): never in the stage's pixels.
import { createContext, useContext, type ReactNode } from "react";
import { useVideoConfig } from "@raea/app/clock";
import { AbsoluteFill } from "./timeline";

const TEXT = "var(--brand-text, #f4efe6)";
const DIM = "var(--brand-text-dim, rgba(244,239,230,0.6))";
const ACCENT = "var(--brand-accent, #ff5a36)";
const PLATE = "var(--brand-plate, rgba(10,10,12,0.72))";
const HEAD = "var(--brand-font-heading, 'Avenir Next Condensed')";
const BODY = "var(--brand-font-body, 'Avenir Next')";

/** the bands' share of the stage's height: the score's at the top, the hint's at the bottom */
const TOP = 0.13;
const BOTTOM = 0.08;
/** the space between a band and the field, as a share of the stage's height */
const GAP = 0.015;

/** a font size for `chars` characters that fits `width` (about 0.62 em a character in a heavy face) under `max` */
const fit = (chars: number, width: number, max: number) => Math.min(max, width / Math.max(1, chars * 0.62));

interface Bands { w: number; h: number; top: number; bottom: number }
const BandContext = createContext<Bands>({ w: 1920, h: 1080, top: TOP * 1080, bottom: BOTTOM * 1080 });

export interface ArenaProps {
  /** the field's own size, in the units its children are drawn in */
  width: number;
  height: number;
  /** the stage behind everything (the field's own ground is a child) */
  background?: string;
  score?: ReactNode;
  banner?: ReactNode;
  hint?: ReactNode;
  children?: ReactNode;
}

/** The game's frame: the field scaled whole between the score's band and the hint's, the banner over its middle. */
export function Arena({ width, height, background = "var(--brand-background, #121014)", score, banner, hint, children }: ArenaProps) {
  const { width: W, height: H } = useVideoConfig();
  const top = TOP * H, bottom = BOTTOM * H, gap = GAP * H;
  const room = { w: W * 0.96, h: H - top - bottom - 2 * gap };
  const scale = Math.min(room.w / width, room.h / height);
  const left = (W - width * scale) / 2, y = top + gap + (room.h - height * scale) / 2;
  return (
    <BandContext.Provider value={{ w: W, h: H, top, bottom }}>
      <AbsoluteFill style={{ background, overflow: "hidden" }}>
        {/* the field is a window: what's drawn past its edges (a level wider than it, a player falling off) is cut */}
        <div style={{ position: "absolute", left, top: y, width, height, transform: `scale(${scale})`, transformOrigin: "0 0", overflow: "hidden" }}>{children}</div>
        <div style={{ position: "absolute", left: 0, top: 0, width: W, height: top }}>{score}</div>
        {banner ? <div style={{ position: "absolute", left: 0, top: y, width: W, height: height * scale, display: "flex", alignItems: "center", justifyContent: "center" }}>{banner}</div> : null}
        <div style={{ position: "absolute", left: 0, top: H - bottom, width: W, height: bottom }}>{hint}</div>
      </AbsoluteFill>
    </BandContext.Provider>
  );
}

export interface ScoreItem {
  label: string;
  value: number | string;
  /** the player's own, lit in the accent */
  accent?: boolean;
}

/** The score bar: each item a cell of its own across the top band, its label small over its value. */
export function Score({ items }: { items: ScoreItem[] }) {
  const { w, top } = useContext(BandContext);
  const cell = (w * 0.9) / Math.max(1, items.length);
  return (
    <div style={{ display: "flex", width: w * 0.9, height: top, marginLeft: w * 0.05, alignItems: "center" }}>
      {items.map((it, i) => (
        <div key={i} style={{ width: cell, display: "flex", flexDirection: "column", alignItems: "center", justifyContent: "center", overflow: "hidden" }}>
          <div style={{ fontFamily: HEAD, fontWeight: 700, letterSpacing: "0.12em", textTransform: "uppercase", color: it.accent ? ACCENT : DIM, fontSize: fit(it.label.length, cell * 0.8, top * 0.2), lineHeight: 1.1 }}>{it.label}</div>
          <div style={{ fontFamily: HEAD, fontWeight: 800, color: TEXT, fontSize: fit(String(it.value).length, cell * 0.8, top * 0.5), lineHeight: 1 }}>{it.value}</div>
        </div>
      ))}
    </div>
  );
}

/** A message over the field's middle ("space to serve", "you win"), on a plate so it reads over play; one at a time. */
export function Banner({ text, sub }: { text: string; sub?: string }) {
  const { w, h } = useContext(BandContext);
  const width = w * 0.6;
  return (
    <div style={{ maxWidth: width, padding: `${h * 0.02}px ${h * 0.04}px`, background: PLATE, borderRadius: h * 0.015, display: "flex", flexDirection: "column", alignItems: "center" }}>
      <div style={{ fontFamily: HEAD, fontWeight: 800, textTransform: "uppercase", color: TEXT, fontSize: fit(text.length, width * 0.9, h * 0.08), lineHeight: 1.05, whiteSpace: "nowrap" }}>{text}</div>
      {sub ? <div style={{ fontFamily: BODY, fontWeight: 600, letterSpacing: "0.1em", textTransform: "uppercase", color: DIM, fontSize: fit(sub.length, width * 0.9, h * 0.03), marginTop: h * 0.01, whiteSpace: "nowrap" }}>{sub}</div> : null}
    </div>
  );
}

/** The controls along the bottom band: each key in a box, what it does beside it. */
export function Hint({ keys }: { keys: { keys: string; does: string }[] }) {
  const { w, bottom } = useContext(BandContext);
  const size = Math.min(bottom * 0.32, fit(keys.reduce((n, k) => n + k.keys.length + k.does.length + 4, 0), w * 0.8, bottom * 0.32));
  return (
    <div style={{ display: "flex", width: w, height: bottom, alignItems: "center", justifyContent: "center", gap: size * 1.6 }}>
      {keys.map((k, i) => (
        <div key={i} style={{ display: "flex", alignItems: "center", gap: size * 0.5, fontFamily: BODY, fontWeight: 600, fontSize: size, color: DIM, whiteSpace: "nowrap" }}>
          <span style={{ color: TEXT, border: `${Math.max(1, size * 0.08)}px solid ${DIM}`, borderRadius: size * 0.3, padding: `${size * 0.1}px ${size * 0.45}px` }}>{k.keys}</span>
          <span>{k.does}</span>
        </div>
      ))}
    </div>
  );
}
