// Bevy v0.20.0 examples/ui/layout/fixed_node.rs in raea's TSX (docs/control.feature FC0a): a full-window css BLUE node
// centring its children, and its one child a 100 x 100 css YELLOW FixedNode, laid out as a root (so at the window's
// top left, not the centre). Its pointer observers (red while hovered) see no pointer here. Bevy's Camera2d over the
// default clear colour is the layered <Canvas3D>. In Bevy's default 1280 x 720 window.
import { AbsoluteFill, Composition, registerRoot } from "raea";
import { Canvas3D } from "raea/3d";
import { Node, percent, px } from "raea/ui";
import { Srgba } from "raea/color";

const BLUE = new Srgba(0, 0, 1), YELLOW = new Srgba(1, 1, 0);

function Shot() {
  return (
    <AbsoluteFill>
      <Canvas3D layered clearColor="rgb(43, 44, 47)" />
      <Node width={percent(100)} height={percent(100)} align_items="Center" justify_content="Center" background_color={BLUE}>
        <Node fixed width={px(100)} height={px(100)} background_color={YELLOW} />
      </Node>
    </AbsoluteFill>
  );
}

registerRoot(() => <Composition id="main" component={Shot} durationInFrames={11} fps={60} width={1280} height={720} />);
