// Bevy v0.20.0 examples/ui/layout/z_index.rs in raea's TSX (docs/control.feature FC0a): a 180 x 100 gray node centred
// in the window holding five absolute boxes: red (no index), blue at ZIndex(2) and lime at ZIndex(-1), which rank
// among their siblings above their parent, purple at GlobalZIndex(1), over everything, and yellow at
// GlobalZIndex(-1), under the gray node itself. Bevy's Camera2d over its black ClearColor is the layered
// <Canvas3D>. In Bevy's default 1280 x 720 window.
import { AbsoluteFill, Composition, registerRoot } from "raea";
import { Canvas3D } from "raea/3d";
import { Node, percent, px } from "raea/ui";
import { Srgba } from "raea/color";

const H = 0.5019608;
const BOXES = [
  { left: 10, bottom: 40, width: 100, height: 50, color: new Srgba(1, 0, 0) },
  { left: 45, bottom: 30, width: 100, height: 50, color: new Srgba(0, 0, 1), z_index: 2 },
  { left: 70, bottom: 20, width: 100, height: 75, color: new Srgba(0, 1, 0), z_index: -1 },
  { left: 15, bottom: 10, width: 100, height: 60, color: new Srgba(H, 0, H), global_z_index: 1 },
  { left: -15, bottom: -15, width: 100, height: 125, color: new Srgba(1, 1, 0), global_z_index: -1 },
];

function Shot() {
  return (
    <AbsoluteFill>
      <Canvas3D layered clearColor="rgb(0, 0, 0)" />
      <Node width={percent(100)} height={percent(100)} align_items="Center" justify_content="Center">
        <Node width={px(180)} height={px(100)} background_color={new Srgba(H, H, H)}>
          {BOXES.map(({ left, bottom, width, height, color, ...z }, i) => (
            <Node key={i} position_type="Absolute" left={px(left)} bottom={px(bottom)} width={px(width)} height={px(height)} background_color={color} {...z} />
          ))}
        </Node>
      </Node>
    </AbsoluteFill>
  );
}

registerRoot(() => <Composition id="main" component={Shot} durationInFrames={11} fps={60} width={1280} height={720} />);
