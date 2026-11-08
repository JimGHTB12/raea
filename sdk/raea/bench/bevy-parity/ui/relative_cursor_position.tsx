// Bevy v0.20.0 examples/ui/relative_cursor_position.rs in raea's TSX: the UI camera's viewport is 600 x 600 px at
// (200, 100); in it a full-size root centres a column of a 250 x 250 red-orange node (15 px under it) and its
// RelativeCursorPosition in FiraSans-Bold 33 px: "unknown" in red, as no pointer is over the window here. Bevy's
// Camera2d is the layered <Canvas3D> over the default clear colour. In Bevy's default 1280 x 720 window.
import { AbsoluteFill, Composition, registerRoot, staticFile } from "raea";
import { Canvas3D } from "raea/3d";
import { Node, Text, UiRect, Viewport, percent, px } from "raea/ui";
import { Srgba } from "raea/color";

function Shot() {
  return (
    <AbsoluteFill>
      <Canvas3D layered clearColor="rgb(43, 44, 47)" />
      <Viewport physical_position={[200, 100]} physical_size={[600, 600]}>
        <Node width={percent(100)} height={percent(100)} align_items="Center" justify_content="Center" flex_direction="Column">
          <Node width={px(250)} height={px(250)} margin={UiRect.bottom(px(15))} background_color={new Srgba(0.92, 0.14, 0.05)} />
          <Text font={staticFile("fonts/FiraSans-Bold.ttf")} font_size={{ Px: 33 }} color={new Srgba(0.9, 0.1, 0.1)}>unknown</Text>
        </Node>
      </Viewport>
    </AbsoluteFill>
  );
}

registerRoot(() => <Composition id="main" component={Shot} durationInFrames={11} fps={60} width={1280} height={720} />);
