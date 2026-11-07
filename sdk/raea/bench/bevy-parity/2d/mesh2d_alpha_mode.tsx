// Bevy v0.20.0 examples/2d/mesh2d_alpha_mode.rs in raea's TSX (docs/control.feature FC0a): a Camera2d and six 256 px
// Rectangles with branding/icon.png as their ColorMaterial texture. On the left, three Opaque ones at x -400, -300 and
// -200 (z 0, 1, -1) tinted css WHITE, BLUE and GREEN. On the right, one Mask(0.5) in WHITE at x 200 (z 0) and two Blend
// ones in BLUE and GREEN at alpha 0.7, at x 300 and 400 (z 1, -1). On Bevy's default clear colour, in Bevy's default
// 1280 x 720 window.
import { AbsoluteFill, Composition, registerRoot, staticFile } from "raea";
import { Canvas3D } from "raea/3d";
import { Transform } from "raea/transform";
import { ColorMaterial, Mesh2d, Rectangle, type AlphaMode2d } from "raea/sprite";
import { Srgba } from "raea/color";

// bevy_color css WHITE, BLUE and GREEN
const WHITE = new Srgba(1, 1, 1, 1), BLUE = new Srgba(0, 0, 1, 1), GREEN = new Srgba(0, 0.5019608, 0, 1);

const ICON = staticFile("branding/icon.png");
const SQUARES: [number, number, Srgba, AlphaMode2d][] = [
  [-400, 0, WHITE, "Opaque"],
  [-300, 1, BLUE, "Opaque"],
  [-200, -1, GREEN, "Opaque"],
  [200, 0, WHITE, { Mask: 0.5 }],
  [300, 1, BLUE.with_alpha(0.7), "Blend"],
  [400, -1, GREEN.with_alpha(0.7), "Blend"],
];

function Shot() {
  return (
    <AbsoluteFill>
      <Canvas3D layered clearColor="rgb(43, 44, 47)">
        {SQUARES.map(([x, z, color, alpha_mode], i) => (
          <Mesh2d key={i} transform={Transform.fromXyz(x, 0, z)}>
            <Rectangle half_size={[128, 128]} />
            <ColorMaterial color={color} texture={ICON} alpha_mode={alpha_mode} />
          </Mesh2d>
        ))}
      </Canvas3D>
    </AbsoluteFill>
  );
}

registerRoot(() => <Composition id="main" component={Shot} durationInFrames={11} fps={60} width={1280} height={720} />);
