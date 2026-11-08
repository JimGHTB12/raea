// Bevy v0.20.0 examples/ui/ui_target_camera.rs in raea's TSX: three Camera2ds (orders 0, 1, 2) cleared red, green and
// blue, each drawing one 100 x 100 black box with a 10 px yellow border and its order in 50 px of its own colour
// (UiTargetCamera), the boxes 67 px apart diagonally about the centre; the text over them has no target, so the default
// camera (the highest order) draws it. The cameras share the window's main texture, which only the first clears: red,
// the layered <Canvas3D>.
// Its clicks see no pointer here. In Bevy's default 1280 x 720 window.
import { AbsoluteFill, Composition, registerRoot } from "raea";
import { Canvas3D } from "raea/3d";
import { Node, Text, UiRect, UiTargetCamera, px } from "raea/ui";
import { Srgba } from "raea/color";

const BOX = 100;
const CAMERAS = [new Srgba(1, 0, 0), new Srgba(0, 0.5019608, 0), new Srgba(0, 0, 1)];

function Shot() {
  return (
    <AbsoluteFill>
      <Canvas3D layered clearColor="rgb(255, 0, 0)" />
      {CAMERAS.map((color, i) => (
        <UiTargetCamera key={i} order={i}>
          <Node align_self="Center" justify_self="Center" justify_content="Center" align_items="Center" left={px(0.67 * BOX * (i - 1))} top={px(0.67 * BOX * (i - 1))} width={px(BOX)} height={px(BOX)} border={UiRect.all(px(0.1 * BOX))} background_color={new Srgba(0, 0, 0)} border_color={new Srgba(1, 1, 0)}>
            <Text font_size={50} color={color}>{`${i}`}</Text>
          </Node>
          {i === CAMERAS.length - 1 && (
            <Text align_self="Center" justify_self="Center" justify_content="Center" bottom={px(2 * BOX)}>{"Each box is rendered by a different camera\n* left-click: increase the camera's order\n* right-click: decrease the camera's order"}</Text>
          )}
        </UiTargetCamera>
      ))}
    </AbsoluteFill>
  );
}

registerRoot(() => <Composition id="main" component={Shot} durationInFrames={11} fps={60} width={1280} height={720} />);
