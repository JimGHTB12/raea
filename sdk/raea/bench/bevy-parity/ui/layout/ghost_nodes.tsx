// Bevy v0.20.0 examples/ui/layout/ghost_nodes.rs in raea's TSX: a root under a GhostNode (so a root itself) holding
// "This text node is rendered under a ghost root", and a full-window root centring a row of three 150 x 65 buttons
// (5 px black borders, fully round, 0.15 grey, "0" in FiraSans-Bold 33 px, 0.9 grey), the first two under a GhostNode
// and so laid out in the row as its own children. Their counters see no pointer here. Bevy's Camera2d over the
// default clear colour is the layered <Canvas3D>. In Bevy's default 1280 x 720 window.
import { AbsoluteFill, Composition, registerRoot, staticFile } from "raea";
import { Canvas3D } from "raea/3d";
import { BorderRadius, GhostNode, Node, Text, UiRect, percent, px } from "raea/ui";
import { Srgba } from "raea/color";

const FONT = staticFile("fonts/FiraSans-Bold.ttf");

const Label = ({ text }: { text: string }) => <Text font={FONT} font_size={{ Px: 33 }} color={new Srgba(0.9, 0.9, 0.9)}>{text}</Text>;

function Button() {
  return (
    <Node width={px(150)} height={px(65)} border={UiRect.all(px(5))} justify_content="Center" align_items="Center" border_radius={BorderRadius.MAX} border_color={new Srgba(0, 0, 0)} background_color={new Srgba(0.15, 0.15, 0.15)}>
      <Label text="0" />
    </Node>
  );
}

function Shot() {
  return (
    <AbsoluteFill>
      <Canvas3D layered clearColor="rgb(43, 44, 47)" />
      <GhostNode>
        <Node><Label text="This text node is rendered under a ghost root" /></Node>
      </GhostNode>
      <Node width={percent(100)} height={percent(100)} align_items="Center" justify_content="Center">
        <Node>
          <GhostNode>
            <Button />
            <Button />
          </GhostNode>
          <Button />
        </Node>
      </Node>
    </AbsoluteFill>
  );
}

registerRoot(() => <Composition id="main" component={Shot} durationInFrames={11} fps={60} width={1280} height={720} />);
