// A page past raea's smaller Vello bump buffers on its first frame (70 clipped, 3D-transformed, blurred shards of a
// chrome block under a grain overlay at 1920 x 1080): Vello then draws nothing and says so only in a read back.

import { AbsoluteFill, Composition, registerRoot, useCurrentFrame } from "raea";

const rnd = (i: number, k: number) => { const s = Math.sin(i * 127.1 + k * 311.7) * 43758.5453; return s - Math.floor(s); };

function Chrome() {
  return (
    <div style={{ position: "absolute", left: 470, top: 214, width: 980, height: 640, borderRadius: 14, background: "linear-gradient(160deg, #3a4150, #1b1f27)", boxShadow: "0 30px 80px rgba(0,0,0,0.6)" }}>
      <div style={{ height: 46, background: "linear-gradient(#4a5160, #2c313b)", borderRadius: "14px 14px 0 0" }} />
      <div style={{ position: "absolute", left: 40, top: 90, width: 400, height: 24, background: "#8fa3c0", borderRadius: 6 }} />
      <div style={{ position: "absolute", left: 40, top: 140, width: 600, height: 300, background: "radial-gradient(circle at 30% 30%, #6b7fa0, #1e2533)", borderRadius: 10 }} />
    </div>
  );
}

function Scene() {
  const f = useCurrentFrame();
  const t = f / 30;
  const shards = Array.from({ length: 70 }, (_, i) => {
    const x = 470 + rnd(i, 1) * 980, y = 214 + rnd(i, 2) * 640;
    const pts = [0, 1, 2].map((k) => `${x + (rnd(i, k + 3) - 0.5) * 220}px ${y + (rnd(i, k + 6) - 0.5) * 220}px`).join(", ");
    const d = 0.5 + t;
    return (
      <div key={i} style={{ position: "absolute", inset: 0, clipPath: `polygon(${pts})`, transform: `perspective(1200px) translate3d(${(rnd(i, 9) - 0.5) * 900 * d}px, ${(rnd(i, 10) - 0.5) * 600 * d}px, ${rnd(i, 11) * 300 * d}px) rotateX(${rnd(i, 12) * 90 * d}deg) rotateY(${rnd(i, 13) * 90 * d}deg)`, filter: `blur(${1 + rnd(i, 14) * 2}px) brightness(${1 + rnd(i, 15) * 0.5})` }}>
        <Chrome />
      </div>
    );
  });
  return (
    <AbsoluteFill style={{ background: "#0b0d12", overflow: "hidden" }}>
      {shards}
      <AbsoluteFill style={{ opacity: 0.05, backgroundImage: "repeating-radial-gradient(circle at 17% 32%, #fff 0px, #000 2px, #fff 3px)", backgroundSize: "7px 7px" }} />
    </AbsoluteFill>
  );
}
registerRoot(() => <Composition id="shards" component={Scene} width={1920} height={1080} fps={30} durationInFrames={3} />);
