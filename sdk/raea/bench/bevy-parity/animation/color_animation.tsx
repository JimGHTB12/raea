// Bevy v0.20.0 examples/animation/color_animation.rs in raea's TSX (docs/control.feature FC0a): six 75 px sprites at
// y 275 to -275, sliding across 600 px on t = (sin(elapsed) + 1) / 2 and coloured through white, yellow, red and black:
// the top three by a cubic Bezier of the four in LinearRgba, Xyza and Oklaba, the bottom three by mixing the two
// neighbours in Hsla, Srgba and Oklcha, on Bevy's default clear colour, in Bevy's default 1280 x 720 window.
import { AbsoluteFill, Composition, registerRoot, useCurrentFrame } from "raea";
import { Canvas3D } from "raea/3d";
import { Hsla, LinearRgba, Oklaba, Oklcha, Srgba, Xyza } from "raea/color";
import { Shape2D } from "raea/sprite";

const f = Math.fround;
const COLORS = [new LinearRgba(1, 1, 1), new LinearRgba(1, 1, 0), new LinearRgba(1, 0, 0), new LinearRgba(0, 0, 0)];

/** bevy_math's CubicBezier position at t, by its characteristic matrix, on each channel `get` reads */
function bezier(ch: number[][], t: number): number[] {
  return ch[0]!.map((_, k) => {
    const [p0, p1, p2, p3] = ch.map((c) => c[k]!) as [number, number, number, number];
    const c = [p0, f(-3 * p0 + 3 * p1), f(3 * p0 - 6 * p1 + 3 * p2), f(-p0 + 3 * p1 - 3 * p2 + p3)];
    return f(f(f(c[0]! + f(c[1]! * t)) + f(c[2]! * f(t * t))) + f(c[3]! * f(f(t * t) * t)));
  });
}

/** animate_mixed: the neighbours either side of t over the four, mixed */
function mixed<T extends { mix(o: T, t: number): T }>(cs: T[], t: number): T {
  const n = cs.length - 1;
  const i = Math.min(Math.floor(f(t * n)), n - 1);
  return cs[i]!.mix(cs[i + 1]!, f(f(t * n) - i));
}

function Shot() {
  // elapsed: f updates of 16_666_667 ns (none on the first), as f32
  const e = f(useCurrentFrame() * 16_666_667e-9);
  const t = f(f(f(Math.sin(e)) + 1) / 2);
  const x = 640 + f(600 * f(t - 0.5));
  const lin = bezier(COLORS.map((c) => [c.red, c.green, c.blue, c.alpha]), t);
  const xyz = bezier(COLORS.map((c) => Xyza.from(c)).map((c) => [c.x, c.y, c.z, c.alpha]), t);
  const ok = bezier(COLORS.map((c) => Oklaba.from(c)).map((c) => [c.lightness, c.a, c.b, c.alpha]), t);
  const rows: [number, Srgba][] = [
    [275, new LinearRgba(lin[0]!, lin[1]!, lin[2]!, lin[3]).to_srgba()],
    [175, new Xyza(xyz[0]!, xyz[1]!, xyz[2]!, xyz[3]).to_srgba()],
    [75, new Oklaba(ok[0]!, ok[1]!, ok[2]!, ok[3]).to_srgba()],
    [-75, mixed(COLORS.map((c) => Hsla.from(c)), t).to_srgba()],
    [-175, mixed(COLORS.map((c) => c.to_srgba()), t)],
    [-275, mixed(COLORS.map((c) => Oklcha.from(c)), t).to_srgba()],
  ];
  return (
    <AbsoluteFill>
      <Canvas3D layered clearColor="rgb(43, 44, 47)">
        {rows.map(([y, c]) => <Shape2D key={y} shape="rect" size={[75, 75]} color={c.to_hex().slice(0, 7)} x={x} y={360 - y} />)}
      </Canvas3D>
    </AbsoluteFill>
  );
}

registerRoot(() => <Composition id="main" component={Shot} durationInFrames={301} fps={60} width={1280} height={720} />);
