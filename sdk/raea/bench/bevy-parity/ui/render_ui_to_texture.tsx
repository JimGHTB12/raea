// Bevy v0.20.0 examples/ui/render_ui_to_texture.rs in raea's TSX (docs/control.feature FC0a): a bevy_ui tree drawn by
// a Camera2d (order -1) into a 512 x 512 image, the base_color_texture (reflectance 0.02) of a default 1 m Cuboid at
// z 1.5. The tree is a css GRAY root filling the image, centring an absolute css BLUE box (20 px padding, 10 px
// corners) holding "Drag Me!" in white 40 px Bevy's default font. The cube is turned PI about X, then about world X at
// 0.1 rad/s and world Y at 0.07 rad/s each update (rotate_x, then rotate_y). A default DirectionalLight and the camera
// are at z 5, looking at the origin, in Bevy's default 1280 x 720 window. The diegetic pointer (a ray cast into the
// cube's UVs) hovers and drags the box only under a cursor, so it stays blue where it was laid out.
import { AbsoluteFill, Composition, registerRoot, useCurrentFrame } from "raea";
import { Canvas3D } from "raea/3d";
import { Quat, Transform } from "raea/transform";
import { Camera3d } from "raea/camera";
import { Cuboid, Mesh3d } from "raea/mesh";
import { StandardMaterial } from "raea/pbr";
import { DirectionalLight } from "raea/light";
import { BorderRadius, Node, Text, UiRect, UiTexture, percent, px, targetImage } from "raea/ui";
import { Srgba } from "raea/color";

const IMAGE = targetImage("ui", 512, 512);
const DT = 1 / 60;

// the cube's rotation after `frame` updates of rotator_system, each a delta of 1/60 s
function turned(frame: number) {
  let q = Quat.fromRotationX(Math.PI);
  for (let i = 0; i < frame; i++) q = Quat.mul(Quat.fromRotationY(0.07 * DT), Quat.mul(Quat.fromRotationX(0.1 * DT), q));
  return q;
}

function Shot() {
  const frame = useCurrentFrame();
  return (
    <AbsoluteFill>
      <Canvas3D layered clearColor="rgb(43, 44, 47)">
        <DirectionalLight />
        <Mesh3d transform={{ translation: [0, 0, 1.5], rotation: turned(frame) }}>
          <Cuboid half_size={[0.5, 0.5, 0.5]} />
          <StandardMaterial base_color_texture={IMAGE} reflectance={0.02} />
        </Mesh3d>
        <Camera3d transform={Transform.fromXyz(0, 0, 5).lookingAt([0, 0, 0])} />
      </Canvas3D>
      <UiTexture target={IMAGE}>
        <Node width={percent(100)} height={percent(100)} flex_direction="Column" justify_content="Center" align_items="Center" background_color={new Srgba(0.5, 0.5, 0.5)}>
          <Node position_type="Absolute" align_items="Center" padding={UiRect.all(px(20))} border_radius={BorderRadius.all(px(10))} background_color={new Srgba(0, 0, 1)}>
            <Text font_size={{ Px: 40 }} color={new Srgba(1, 1, 1)}>Drag Me!</Text>
          </Node>
        </Node>
      </UiTexture>
    </AbsoluteFill>
  );
}

registerRoot(() => <Composition id="main" component={Shot} durationInFrames={11} fps={60} width={1280} height={720} />);
