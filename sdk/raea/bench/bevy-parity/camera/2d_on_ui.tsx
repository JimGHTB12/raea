// Bevy v0.20.0 examples/camera/2d_on_ui.rs in raea's TSX (docs/control.feature FC0a): a full-window tailwind ROSE_400
// node centring a box (20% x 30%, at least 150 px, a 2 px white border rounded 25%) drawn by the default UI camera, and
// over it, by a second Camera2d (order 1, ClearColorConfig::None, RenderLayers 1), the sensei (textures/rpg/chars/
// sensei/sensei.png) as a 100 x 100 Sprite in the middle, turned each update by Rz(0.5 dt) * Ry(dt). The second camera
// over the UI is the layered <Canvas3D> after the page's node: the 3D lies over the elements before it. In Bevy's
// default 1280 x 720 window.
import { AbsoluteFill, Composition, registerRoot, staticFile, useCurrentFrame } from "raea";
import { Canvas3D, Sprite } from "raea/3d";
import { Srgba } from "raea/color";
import { Quat } from "raea/transform";
import { BorderRadius, Node, UiRect, percent, px } from "raea/ui";

// tailwind::ROSE_400
const ROSE_400 = new Srgba(251 / 255, 113 / 255, 133 / 255);

/** rotate_sprite over the updates up to frame f: rotation *= Rz(0.5 dt) * Ry(dt), dt a 1/60 s step (none on the first) */
function rotationAt(f: number): Quat {
  let q = Quat.IDENTITY;
  const dt = Math.fround(16_666_667e-9);
  for (let k = 1; k <= f; k++) q = Quat.mul(q, Quat.mul(Quat.fromRotationZ(dt * 0.5), Quat.fromRotationY(dt)));
  return q;
}

function Shot() {
  const q = rotationAt(useCurrentFrame());
  return (
    <AbsoluteFill>
      <Node width={percent(100)} height={percent(100)} display="Flex" justify_content="Center" align_items="Center" background_color={ROSE_400}>
        <Node height={percent(30)} width={percent(20)} min_height={px(150)} min_width={px(150)} border={UiRect.all(px(2))} border_radius={BorderRadius.all(percent(25))} border_color={new Srgba(1, 1, 1)} />
      </Node>
      <Canvas3D layered>
        <Sprite src={staticFile("textures/rpg/chars/sensei/sensei.png")} width={100} height={100} x={640} y={360} quat={q} />
      </Canvas3D>
    </AbsoluteFill>
  );
}

registerRoot(() => <Composition id="main" component={Shot} durationInFrames={61} fps={60} width={1280} height={720} />);
