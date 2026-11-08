// Bevy v0.20.0 examples/3d/split_screen.rs in raea's TSX (docs/control.feature FC0a, FC2): four Camera3ds, orders 0 to
// 3, each drawing into its own quarter of the window (Camera.viewport, 640 x 360 at (col, row) x 640 x 360, as its
// set_camera_viewports system sets them on the window's first WindowResized), all looking at the origin from (0, 200,
// -150), (150, 150, 50), (100, 150, -150) and (-100, 80, 150). The scene: a 100 m plane of srgb(0.3, 0.5, 0.3), the
// fox (models/animated/Fox.glb's scene, unanimated) and a shadow-casting DirectionalLight turned by euler ZYX (0, 1,
// -PI/4), its CascadeShadowConfigBuilder's 2 cascades, the first to 200 m, out to 280 m. Each camera's own UI
// (UiTargetCamera, laid out in its viewport): "Player N" in Bevy's default font with 12 px margins, and a row of two
// 40 px FeathersButtons "<" and ">" spread apart (SpaceBetween) inside 20 px padding, under feathers' dark UiTheme.
// The buttons turn their camera only under input. In Bevy's default 1280 x 720 window.
import { Composition, registerRoot, staticFile } from "raea";
import { Model } from "raea/3d";
import { Quat, Transform } from "raea/transform";
import { Camera3d } from "raea/camera";
import { Mesh3d, Plane3d } from "raea/mesh";
import { StandardMaterial } from "raea/pbr";
import { DirectionalLight, Shadows } from "raea/light";
import { Node, Text, UiRect, UiTargetCamera, Viewport, percent, px } from "raea/ui";
import { FeathersButton, UiTheme, caption, create_dark_theme } from "raea/feathers";

const PLAYERS: [string, [number, number, number]][] = [
  ["Player 1", [0, 200, -150]],
  ["Player 2", [150, 150, 50]],
  ["Player 3", [100, 150, -150]],
  ["Player 4", [-100, 80, 150]],
];
// the window's physical size / 2
const SIZE: [number, number] = [640, 360];
const placeOf = (i: number): [number, number] => [(i % 2) * SIZE[0], Math.floor(i / 2) * SIZE[1]];

function Shot() {
  return (
    <>
      <Mesh3d>
        <Plane3d half_size={[50, 50]} />
        <StandardMaterial base_color="rgb(76.5, 127.5, 76.5)" />
      </Mesh3d>
      <Model src={staticFile("models/animated/Fox.glb")} />
      <Shadows mapSize={2048} cascades={2} firstCascade={200} distance={280} overlap={0.2} />
      <DirectionalLight shadow_maps_enabled transform={{ translation: [0, 0, 0], rotation: Quat.fromEuler("ZYX", 0, 1, -Math.PI / 4), scale: [1, 1, 1] }} />
      {PLAYERS.map(([, at], i) => (
        <Camera3d key={i} order={i} viewport={{ physical_position: placeOf(i), physical_size: SIZE }} transform={Transform.fromXyz(...at).lookingAt([0, 0, 0])} />
      ))}
      <UiTheme theme={create_dark_theme()}>
        {PLAYERS.map(([name], i) => (
          <UiTargetCamera key={i} order={i}>
            <Viewport physical_position={placeOf(i)} physical_size={SIZE}>
              <Node width={percent(100)} height={percent(100)}>
                <Text margin={UiRect.all(px(12))}>{name}</Text>
                <Node position_type="Absolute" width={percent(100)} height={percent(100)} display="Flex" flex_direction="Row" justify_content="SpaceBetween" align_items="Center" padding={UiRect.all(px(20))}>
                  <FeathersButton width={px(40)}>{caption("<")}</FeathersButton>
                  <FeathersButton width={px(40)}>{caption(">")}</FeathersButton>
                </Node>
              </Node>
            </Viewport>
          </UiTargetCamera>
        ))}
      </UiTheme>
    </>
  );
}

registerRoot(() => <Composition id="main" component={Shot} durationInFrames={11} fps={60} width={1280} height={720} />);
