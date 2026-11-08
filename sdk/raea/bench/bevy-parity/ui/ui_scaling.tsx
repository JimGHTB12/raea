// Bevy v0.20.0 examples/ui/ui_scaling.rs in raea's TSX: an antique-white box over the middle quarter of the window
// (absolute, a quarter in from the left and top, half as wide and tall; children spaced around, centred), holding a
// 40 px red square with "Size!" in black 13 px text, a blue box 15% of the parent each way, and a 30 px ImageNode of
// branding/icon.png. UiScale eases to its target only when the up or down arrow is pressed, so it holds at 1. Bevy's
// Camera2d over the default clear colour is the layered <Canvas3D>. In Bevy's default 1280 x 720 window.
import { AbsoluteFill, Composition, registerRoot, staticFile } from "raea";
import { Canvas3D } from "raea/3d";
import { ImageNode, Node, Text, percent, px } from "raea/ui";
import { Srgba } from "raea/color";

function Shot() {
  return (
    <AbsoluteFill>
      <Canvas3D layered clearColor="rgb(43, 44, 47)" />
      <Node width={percent(50)} height={percent(50)} position_type="Absolute" left={percent(25)} top={percent(25)} justify_content="SpaceAround" align_items="Center" background_color={new Srgba(250 / 255, 235 / 255, 215 / 255)}>
        <Node width={px(40)} height={px(40)} background_color={new Srgba(1, 0, 0)}>
          <Text font_size={{ Px: 13 }} color={new Srgba(0, 0, 0)}>Size!</Text>
        </Node>
        <Node width={percent(15)} height={percent(15)} background_color={new Srgba(0, 0, 1)} />
        <ImageNode image={staticFile("branding/icon.png")} width={px(30)} height={px(30)} />
      </Node>
    </AbsoluteFill>
  );
}

registerRoot(() => <Composition id="main" component={Shot} durationInFrames={11} fps={60} width={1280} height={720} />);
