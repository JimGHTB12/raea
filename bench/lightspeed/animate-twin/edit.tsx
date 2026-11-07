// The ANIMATE fixture's twin, written the way agents write today: interpolate and spring per frame, the same timings.
import { AbsoluteFill, Composition, Easing, Sequence, interpolate, registerRoot, spring, useCurrentFrame, useVideoConfig } from "raea";
const INK = "#14121f", PAPER = "#fbf7ef", ACCENT = "#ff5a7a";
const OUT = Easing.out(Easing.cubic), IN = Easing.in(Easing.cubic);
const c = { extrapolateLeft: "clamp", extrapolateRight: "clamp" } as const;
const GROUPS: { from: number; words: [string, number, number][] }[] = [
  { from: 0, words: [["Your", 0, 8], ["afternoon", 8, 22], ["slump", 22, 40]] },
  { from: 44, words: [["has", 0, 6], ["a", 6, 10], ["new", 10, 20]] },
  { from: 70, words: [["enemy.", 0, 30]] },
  { from: 110, words: [["Steeped", 0, 14], ["twenty", 14, 26], ["hours,", 26, 46]] },
  { from: 160, words: [["smooth,", 0, 18], ["never", 18, 28], ["bitter.", 28, 52]] },
  { from: 220, words: [["Twelve", 0, 12], ["grams", 12, 22], ["protein,", 22, 44]] },
  { from: 270, words: [["zero", 0, 12], ["added", 12, 22], ["sugar.", 22, 48]] },
  { from: 330, words: [["Crack", 0, 12], ["one", 12, 18], ["open.", 18, 44]] },
  { from: 390, words: [["Finish", 0, 14], ["the", 14, 18], ["list.", 18, 46]] },
  { from: 450, words: [["First", 0, 12], ["case", 12, 22], ["half", 22, 30]] },
  { from: 486, words: [["off.", 0, 40]] },
];
function Caption({ words, length }: { words: [string, number, number][]; length: number }) {
  const f = useCurrentFrame();
  const s = interpolate(f, [0, 4, 8], [0.92, 1.03, 1], { ...c, easing: OUT });
  const o = Math.min(interpolate(f, [0, 3], [0, 1], { ...c, easing: OUT }), interpolate(f, [length - 6, length], [1, 0], { ...c, easing: IN }));
  return (
    <AbsoluteFill style={{ justifyContent: "flex-end", alignItems: "center", paddingBottom: 220 }}>
      <div style={{ display: "flex", opacity: o, transform: `scale(${s})`, backgroundColor: PAPER, borderRadius: 32, padding: "18px 28px", fontFamily: "Helvetica", fontWeight: 900, fontSize: 64, color: INK }}>
        {words.map(([w, a, b], i) => {
          const k = interpolate(f, [a, a + 3, b, b + 3], [0, 1, 1, 0], { ...c, easing: OUT });
          return <span key={i} style={{ padding: "2px 12px", margin: "0 2px", borderRadius: 999, backgroundColor: `rgba(255, 90, 122, ${k})` }}>{w}</span>;
        })}
      </div>
    </AbsoluteFill>
  );
}
const chip = { display: "inline-flex", alignItems: "center", gap: 14, backgroundColor: INK, color: PAPER, borderRadius: 999, padding: "12px 30px 12px 14px", fontFamily: "Helvetica", fontWeight: 900, fontSize: 48, marginBottom: 14 } as const;
const num = { minWidth: 120, textAlign: "center", backgroundColor: ACCENT, color: INK, borderRadius: 999, padding: "4px 18px" } as const;
function Badge({ i, children }: { i: number; children: React.ReactNode }) {
  const f = useCurrentFrame();
  const at = 110 + i * 24, ex = 430 + i * 2;
  const x = interpolate(f, [at, at + 8], [-64, 0], { ...c, easing: OUT }) + interpolate(f, [ex, ex + 8], [0, -64], { ...c, easing: IN });
  const o = Math.min(interpolate(f, [at, at + 5], [0, 1], { ...c, easing: OUT }), interpolate(f, [ex + 3, ex + 8], [1, 0], { ...c, easing: IN }));
  return <div style={{ opacity: o, transform: `translateX(${x}px)` }}>{children}</div>;
}
function Badges() {
  const f = useCurrentFrame();
  const n = Math.round(interpolate(f, [124, 154], [0, 20], { ...c, easing: OUT }));
  return (
    <div style={{ position: "absolute", left: 56, top: 120, display: "flex", flexDirection: "column", alignItems: "flex-start" }}>
      <Badge i={0}><div style={chip}><span style={num}>{n}</span> HRS STEEPED</div></Badge>
      <Badge i={1}><div style={chip}><span style={num}>12</span> G PROTEIN</div></Badge>
      <Badge i={2}><div style={chip}><span style={num}>0</span> G SUGAR</div></Badge>
    </div>
  );
}
function Card() {
  const f = useCurrentFrame();
  const { fps } = useVideoConfig();
  const y = interpolate(f, [0, 10], [80, 0], { ...c, easing: OUT });
  const s = spring({ frame: f, fps, from: 0.86, to: 1, durationInFrames: 18, config: { damping: 15, stiffness: 120 } });
  const t = interpolate(f, [18, 22, 26], [0.6, 1.08, 1], { ...c, easing: OUT });
  const fillW = interpolate(f, [20, 50], [0, 100], { ...c, easing: OUT });
  const wipe = interpolate(f, [18, 30], [100, 0], { ...c, easing: Easing.inOut(Easing.cubic) });
  const cta = interpolate(f, [24, 30], [0, 1], { ...c, easing: OUT });
  return (
    <AbsoluteFill style={{ alignItems: "center", justifyContent: "center", paddingBottom: 200 }}>
      <div style={{ opacity: interpolate(f, [0, 5], [0, 1], { ...c, easing: OUT }), transform: `translateY(${y}px) scale(${s})`, width: 860, backgroundColor: PAPER, borderRadius: 44, padding: 52, display: "flex", flexDirection: "column", gap: 26 }}>
        <div style={{ opacity: interpolate(f, [18, 21], [0, 1], c), transform: `scale(${t})`, fontFamily: "Helvetica", fontWeight: 900, fontSize: 96, color: INK, lineHeight: 1 }}>First case half off</div>
        <div style={{ height: 26, borderRadius: 13, backgroundColor: "#e9e2d6", overflow: "hidden" }}><div style={{ width: `${fillW}%`, height: 26, borderRadius: 13, backgroundColor: ACCENT }} /></div>
        <div style={{ height: 8, backgroundColor: INK, clipPath: `inset(0% ${wipe}% 0% 0%)` }} />
        <div style={{ opacity: cta, fontFamily: "Helvetica", fontSize: 44, color: INK }}>brightside.coffee</div>
      </div>
    </AbsoluteFill>
  );
}
export default function Edit() {
  return (
    <AbsoluteFill style={{ backgroundColor: "#3b2f5c" }}>
      <AbsoluteFill style={{ background: "linear-gradient(160deg, #3b2f5c, #1d1730)" }} />
      <Badges />
      <Sequence from={450} durationInFrames={150} name="card"><Card /></Sequence>
      {GROUPS.map((g, i) => {
        const length = (GROUPS[i + 1]?.from ?? 600) - g.from;
        return <Sequence key={i} from={g.from} durationInFrames={length}><Caption words={g.words} length={length} /></Sequence>;
      })}
    </AbsoluteFill>
  );
}
registerRoot(() => <Composition id="twin" component={Edit} width={1080} height={1920} fps={30} durationInFrames={600} />);
