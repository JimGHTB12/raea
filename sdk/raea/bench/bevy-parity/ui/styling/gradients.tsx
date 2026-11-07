// Bevy v0.20.0 examples/ui/styling/gradients.rs in raea's TSX: three rows, each of 24 linear gradients (70 x 70,
// 35 x 70 and 70 x 35, at the eight angles a turn apart by an eighth) and three squares as tall as the row (a linear
// gradient turning at 0.5 rad/s, a ClosestSide radial one and a conic one), every node 20 px round with a yellow,
// white and orange border gradient: white to black (4 px borders), red, blue and lime (4 px), and the rainbow in hard
// stops (none). Under them "Current Space / Oklaba" between the Previous and Next buttons. In Oklab, Bevy's default;
// its buttons see no pointer here. Bevy's Camera2d over the default clear colour is the layered <Canvas3D>. In Bevy's
// default 1280 x 720 window, at frame 10 (its updates of 1/60 s turned the squares' linear gradients).
import { AbsoluteFill, Composition, registerRoot, useCurrentFrame } from "raea";
import { Canvas3D } from "raea/3d";
import { AngularColorStop, BorderRadius, ColorStop, ConicGradient, LinearGradient, Node, RadialGradient, Text, UiPosition, UiRect, percent, px, type Gradient } from "raea/ui";
import { Srgba } from "raea/color";

const TAU = 2 * Math.PI;
const [RED, BLUE, LIME, ORANGE, YELLOW] = [new Srgba(1, 0, 0), new Srgba(0, 0, 1), new Srgba(0, 1, 0), new Srgba(1, 0.64705884, 0), new Srgba(1, 1, 0)];
const [GREEN, INDIGO, VIOLET] = [new Srgba(0, 0.5019608, 0), new Srgba(0.29411766, 0, 0.50980395), new Srgba(0.93333334, 0.50980395, 0.93333334)];
const [WHITE, BLACK] = [new Srgba(1, 1, 1), new Srgba(0, 0, 0)];

const RAINBOW = [RED, ORANGE, YELLOW, GREEN, BLUE, INDIGO, VIOLET];
const ROWS: [number, ColorStop[]][] = [
  [4, [ColorStop.new(WHITE, percent(15)), ColorStop.new(BLACK, percent(85))]],
  [4, [ColorStop.auto(RED), ColorStop.auto(BLUE), ColorStop.auto(LIME)]],
  // each colour from i/7 to (i+1)/7 of the way
  [0, [ColorStop.auto(RED), ...RAINBOW.flatMap((c, i) => [i > 0 ? ColorStop.new(c, percent((100 * i) / 7)) : null, i < 6 ? ColorStop.new(c, percent((100 * (i + 1)) / 7)) : null]).filter((s): s is ColorStop => s !== null), ColorStop.auto(VIOLET)]],
];
const BORDER: Gradient[] = [LinearGradient.new((3 * TAU) / 8, [ColorStop.auto(YELLOW), ColorStop.auto(WHITE), ColorStop.auto(ORANGE)])];

/** The squares' linear angle after frame f's updates: each turns it 0.5 x its delta_secs (none on the first), in f32 */
function turnAt(f: number): number {
  let a = 0;
  for (let k = 1; k <= f; k++) a = Math.fround(a + Math.fround(0.5 * Math.fround(16_666_667e-9)));
  return a;
}

function Row({ b, stops, turn }: { b: number; stops: ColorStop[]; turn: number }) {
  const rounded = { border: UiRect.all(px(b)), border_radius: BorderRadius.all(px(20)), border_gradient: BORDER };
  const square = { ...rounded, aspect_ratio: 1, height: percent(100), margin: UiRect.left(px(20)) };
  return (
    <Node>
      <Node flex_direction="Column" row_gap={px(5)}>
        {[[70, 70], [35, 70], [70, 35]].map(([w, h]) => (
          <Node key={`${w}x${h}`} column_gap={px(10)}>
            {Array.from({ length: 8 }, (_, i) => <Node key={i} width={px(w)} height={px(h)} {...rounded} background_gradient={[LinearGradient.new((i * TAU) / 8, stops)]} />)}
          </Node>
        ))}
      </Node>
      <Node>
        <Node {...square} background_gradient={[LinearGradient.new(turn, stops)]} />
        <Node {...square} background_gradient={[RadialGradient.new(UiPosition.CENTER, "ClosestSide", stops)]} />
        <Node {...square} background_gradient={[ConicGradient.new(UiPosition.CENTER, stops.map((s) => AngularColorStop.auto(s.color)))]} />
      </Node>
    </Node>
  );
}

function Button({ caption }: { caption: string }) {
  return (
    <Node border={UiRect.all(px(2))} padding={UiRect.axes(px(8), px(4))} justify_content="Center" align_items="Center" border_radius={BorderRadius.MAX} border_color={WHITE} background_color={BLACK}>
      <Text>{caption}</Text>
    </Node>
  );
}

function Shot() {
  const turn = turnAt(useCurrentFrame());
  return (
    <AbsoluteFill>
      <Canvas3D layered clearColor="rgb(43, 44, 47)" />
      <Node flex_direction="Column" row_gap={px(20)} margin={UiRect.all(px(20))}>
        {ROWS.map(([b, stops], i) => <Row key={i} b={b} stops={stops} turn={turn} />)}
        <Node flex_direction="Row" align_items="Center" justify_content="Center" column_gap={px(20)}>
          <Node align_items="Center" justify_content="Center">
            <Text>{"Current Space\nOklaba"}</Text>
          </Node>
          <Button caption="Previous" />
          <Button caption="Next" />
        </Node>
      </Node>
    </AbsoluteFill>
  );
}

registerRoot(() => <Composition id="main" component={Shot} durationInFrames={11} fps={60} width={1280} height={720} />);
