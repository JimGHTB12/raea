// Bevy v0.20.0 examples/ui/styling/stacked_gradients.rs in raea's TSX (docs/control.feature FC0a): a full-window grid
// holding a full-size black node with six BackgroundGradients stacked, the first at the bottom: red fading to the top
// right, blue to the top left, a yellow conic band round the centre, a white-yellow sun (a 30 vh circle 5% right of
// the top centre) and two black fades at TAU/16 and 15 TAU/16, all interpolated in Oklab. Bevy's Camera2d over the
// default clear colour is the layered <Canvas3D>. In Bevy's default 1280 x 720 window.
import { AbsoluteFill, Composition, registerRoot } from "raea";
import { Canvas3D } from "raea/3d";
import { AngularColorStop, ColorStop, ConicGradient, LinearGradient, Node, RadialGradient, UiPosition, percent, vh } from "raea/ui";
import { Srgba } from "raea/color";

const TAU = 2 * Math.PI;
const RED = new Srgba(1, 0, 0), BLUE = new Srgba(0, 0, 1), YELLOW = new Srgba(1, 1, 0), BLACK = new Srgba(0, 0, 0);
const fade = (c: Srgba, a = 0) => c.with_alpha(a);

const GRADIENTS = [
  LinearGradient.to_top_right([ColorStop.auto(RED), ColorStop.auto(fade(RED))]),
  LinearGradient.to_top_left([ColorStop.auto(BLUE), ColorStop.auto(fade(BLUE))]),
  ConicGradient.new(UiPosition.CENTER, [fade(YELLOW), fade(YELLOW), YELLOW, fade(YELLOW), fade(YELLOW)].map(AngularColorStop.auto)),
  RadialGradient.new(UiPosition.TOP.at_x(percent(5)), { Circle: vh(30) }, [ColorStop.auto(new Srgba(1, 1, 1)), ColorStop.auto(YELLOW), ColorStop.auto(fade(YELLOW, 0.1)), ColorStop.auto(fade(YELLOW))]),
  LinearGradient.new(TAU / 16, [ColorStop.auto(BLACK), ColorStop.auto(fade(BLACK))]),
  LinearGradient.new((15 * TAU) / 16, [ColorStop.auto(BLACK), ColorStop.auto(fade(BLACK))]),
];

function Shot() {
  return (
    <AbsoluteFill>
      <Canvas3D layered clearColor="rgb(43, 44, 47)" />
      <Node display="Grid" width={percent(100)} height={percent(100)}>
        <Node width={percent(100)} height={percent(100)} background_color={BLACK} background_gradient={GRADIENTS} />
      </Node>
    </AbsoluteFill>
  );
}

registerRoot(() => <Composition id="main" component={Shot} durationInFrames={11} fps={60} width={1280} height={720} />);
