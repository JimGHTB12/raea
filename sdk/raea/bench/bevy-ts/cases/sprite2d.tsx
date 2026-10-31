// gap: 2D (docs/sdk.feature, the unmapped: ColorMaterial, SpriteMeshMaterial, TextureAtlasLayout)
// 2D over the 3D: a white unlit box fills the middle of the 3D, and over it a <Sprite> of a two-cell sheet (left half
// red, right half blue) showing cell 1 at (20, 32), 20 px wide, and a green <Shape2D> circle at (46, 32). The sprite is
// blue over the box (its sheet cut by a TextureAtlasLayout, drawn with its SpriteMeshMaterial), the circle green (a
// Mesh2d with a ColorMaterial), and the box's white shows between them: the 2D camera draws after the 3D without
// clearing it (the box's white is tone-mapped to about 196). The corner is the 3D's own clear. The sheet is sampled
// nearest, so no red from cell 0 bleeds down the sprite's left edge.
// probe: 10 32 b>200 r<60 g<60
// probe: 20 32 b>200 r<60 g<60
// probe: 46 32 g>200 r<60 b<60
// probe: 33 32 r>150 g>150 b>150
// probe: 2 2 r<40 g<40 b<40
import { AbsoluteFill, Composition, registerRoot } from "raea";
import { Canvas3D, Shape2D, Sprite } from "raea/3d";
import { Cuboid, Mesh3d } from "raea/mesh";
import { Transform } from "raea/transform";
import { Camera3d } from "raea/camera";
import { StandardMaterial } from "raea/pbr";

const SHEET = "data:image/svg+xml;utf8,<svg xmlns='http://www.w3.org/2000/svg' width='16' height='8'><rect width='8' height='8' fill='%23ff0000'/><rect x='8' width='8' height='8' fill='%230000ff'/></svg>";

function Shot() {
  return (
    <AbsoluteFill style={{ background: "#000" }}>
      <Canvas3D layered>
        <Camera3d transform={Transform.fromXyz(0, 0, 3).lookingAt([0, 0, 0])} projection={{ Perspective: { fov: 0.698131700797732 } }} />
        <Mesh3d name="box"><Cuboid half_size={[0.7, 0.4, 0.05]} /><StandardMaterial base_color="#ffffff" unlit /></Mesh3d>
        <Sprite name="sprite" src={SHEET} grid={[2, 1]} frame={1} x={20} y={32} width={20} />
        <Shape2D name="dot" shape="circle" size={[14, 14]} color="#00ff00" x={46} y={32} />
      </Canvas3D>
    </AbsoluteFill>
  );
}
registerRoot(() => <Composition id="main" component={Shot} durationInFrames={2} fps={30} width={64} height={64} />);
