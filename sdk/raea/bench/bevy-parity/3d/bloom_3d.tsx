// Bevy v0.20.0 examples/3d/bloom_3d.rs in raea's TSX (docs/control.feature FC0a): a 10 x 10 grid of spheres bouncing on
// a sine, a quarter of them emissive in Bevy's units (blue 150, white 1000, red 50: LinearRgba, as Bevy writes them)
// and the rest black, under Bloom::NATURAL with TonyMcMapface on black, and the example's settings text. Each sphere's
// material is the example's: Rust's DefaultHasher (SipHash-1-3, keys 0) of its (x, z), plus 3, mod 6.
import { AbsoluteFill, Composition, registerRoot, staticFile, useCurrentFrame } from "raea";
import { Bloom, Sphere, ToneMapping } from "raea/3d";
import { Camera3d } from "raea/camera";
import { LinearRgba } from "raea/color";
import { Mesh3d } from "raea/mesh";
import { StandardMaterial } from "raea/pbr";
import { Transform } from "raea/transform";

const M = (1n << 64n) - 1n;
const rotl = (x: bigint, b: bigint) => ((x << b) | (x >> (64n - b))) & M;
/** Rust's DefaultHasher over `bytes`: SipHash-1-3 with zero keys, the length in the last block */
function sip13(bytes: number[]): bigint {
  let [v0, v1, v2, v3] = [0x736f6d6570736575n, 0x646f72616e646f6dn, 0x6c7967656e657261n, 0x7465646279746573n];
  const round = () => {
    v0 = (v0 + v1) & M; v1 = rotl(v1, 13n); v1 ^= v0; v0 = rotl(v0, 32n);
    v2 = (v2 + v3) & M; v3 = rotl(v3, 16n); v3 ^= v2;
    v0 = (v0 + v3) & M; v3 = rotl(v3, 21n); v3 ^= v0;
    v2 = (v2 + v1) & M; v1 = rotl(v1, 17n); v1 ^= v2; v2 = rotl(v2, 32n);
  };
  const word = (at: number, n: number) => { let w = 0n; for (let i = 0; i < n; i++) w |= BigInt(bytes[at + i]!) << BigInt(8 * i); return w; };
  const full = bytes.length - (bytes.length % 8);
  for (let at = 0; at < full; at += 8) { const m = word(at, 8); v3 ^= m; round(); v0 ^= m; }
  const last = word(full, bytes.length - full) | (BigInt(bytes.length & 0xff) << 56n);
  v3 ^= last; round(); v0 ^= last;
  v2 ^= 0xffn; round(); round(); round();
  return v0 ^ v1 ^ v2 ^ v3;
}
const le32 = (n: number) => [n & 0xff, (n >> 8) & 0xff, (n >> 16) & 0xff, (n >>> 24) & 0xff];

// the example's materials (emissive in Bevy's units; base colour white but for the black one) and each one's scale
const KINDS = [
  { emissive: new LinearRgba(0, 0, 150), scale: 0.5 },
  { emissive: new LinearRgba(1000, 1000, 1000), scale: 0.1 },
  { emissive: new LinearRgba(50, 0, 0), scale: 1 },
];
const GRID: { x: number; z: number; kind: number }[] = [];
for (let x = -5; x < 5; x++) for (let z = -5; z < 5; z++) GRID.push({ x, z, kind: Number(((sip13([...le32(x), ...le32(z)]) + 3n) & M) % 6n) });

const FONT = `@font-face { font-family: "Fira Mono"; src: url("${staticFile("fonts/FiraMono-Medium.ttf")}"); }`;
const TEXT = [
  "Bloom (Toggle: Space)", "(Q/A) Intensity: 0.15", "(W/S) Low-frequency boost: 0.70", "(E/D) Low-frequency boost curvature: 0.95",
  "(R/F) High-pass frequency: 1.00", "(T/G) Mode: Energy-conserving", "(Y/H) Threshold: 0.00", "(U/J) Threshold softness: 0.00",
  "(I/K) Horizontal Scale: 1.00",
].join("\n");

function Shot() {
  // bounce_spheres: y = sin(x + z + elapsed), elapsed f updates of 16_666_667 ns (none on the first), as f32
  const t = Math.fround(useCurrentFrame() * 16_666_667e-9);
  return (
    <AbsoluteFill style={{ background: "#000000" }}>
      <style>{FONT}</style>
      <Camera3d transform={Transform.fromXyz(-2, 2.5, 5).lookingAt([0, 0, 0])} />
      <ToneMapping mode="tony" />
      <Bloom intensity={1} threshold={0} />
      {GRID.map(({ x, z, kind }) => {
        const k = KINDS[kind];
        const s = k?.scale ?? 1.5;
        const [px, pz] = [x * 2, z * 2];
        return (
          <Mesh3d key={`${x},${z}`} transform={{ translation: [px, Math.fround(Math.sin(Math.fround(Math.fround(px + pz) + t))), pz], scale: [s, s, s] }}>
            <Sphere radius={0.4} />
            {k ? <StandardMaterial emissive={k.emissive} /> : <StandardMaterial base_color="#000000" />}
          </Mesh3d>
        );
      })}
      {/* the text ends in a newline: an empty last line in Bevy's text, which HTML drops, so its height as padding */}
      <div style={{ position: "absolute", left: 12, bottom: 12, paddingBottom: 24, fontFamily: "Fira Mono", fontSize: 20, lineHeight: 1.2, color: "#fff", whiteSpace: "pre" }}>{TEXT}</div>
    </AbsoluteFill>
  );
}

registerRoot(() => <Composition id="main" component={Shot} durationInFrames={61} fps={60} width={1280} height={720} />);
