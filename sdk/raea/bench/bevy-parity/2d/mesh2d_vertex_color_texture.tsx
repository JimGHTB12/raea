// Bevy v0.20.0 examples/2d/mesh2d_vertex_color_texture.rs in raea's TSX (docs/control.feature FC0a): a Camera2d and
// two Rectangle::default() quads scaled 128, their vertices coloured linear RED, GREEN, BLUE and WHITE (top right, top
// left, bottom left, bottom right) by Mesh::ATTRIBUTE_COLOR. The one at x -96 has ColorMaterial::default(); the one at
// x 96 has branding/banner.png as its texture, which the vertex colours tint. On Bevy's default clear colour, in Bevy's
// default 1280 x 720 window.
import { AbsoluteFill, Composition, registerRoot, staticFile } from "raea";
import { Canvas3D } from "raea/3d";
import { Transform } from "raea/transform";
import { ColorMaterial, Mesh2d, Rectangle } from "raea/sprite";
import { LinearRgba } from "raea/color";

// LinearRgba::RED, GREEN, BLUE and WHITE, one per corner of the quad
const COLORS = [new LinearRgba(1, 0, 0, 1), new LinearRgba(0, 1, 0, 1), new LinearRgba(0, 0, 1, 1), new LinearRgba(1, 1, 1, 1)];
const BANNER = staticFile("branding/banner.png");

function Shot() {
  return (
    <AbsoluteFill>
      <Canvas3D layered clearColor="rgb(43, 44, 47)">
        <Mesh2d transform={Transform.fromXyz(-96, 0, 0).withScale(128)}>
          <Rectangle vertex_colors={COLORS} />
          <ColorMaterial />
        </Mesh2d>
        <Mesh2d transform={Transform.fromXyz(96, 0, 0).withScale(128)}>
          <Rectangle vertex_colors={COLORS} />
          <ColorMaterial texture={BANNER} />
        </Mesh2d>
      </Canvas3D>
    </AbsoluteFill>
  );
}

registerRoot(() => <Composition id="main" component={Shot} durationInFrames={11} fps={60} width={1280} height={720} />);
