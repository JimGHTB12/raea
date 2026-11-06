// gap: PL7
// bevy_vector_shapes through raea/plugins/vector_shapes (docs/plugins.feature PL7): two retained shapes, each a bare
// Entity carrying the components ShapeBundle would (ShapeMaterial, ShapeFill, the shape, Shape3d for the 3D
// pipeline), facing the camera against a black frame: a filled red disc on the left, a filled cyan rectangle on the
// right, and black between and above them.
// build: plugins
// frame: 1
// probe: 16 32 r>150 g<80 b<80
// probe: 48 32 g>150 b>150 r<80
// probe: 32 4 r<20 g<20 b<20
import { AbsoluteFill, Composition, registerRoot } from "raea";
import { Entity } from "raea/3d";
import { Camera3d } from "raea/camera";
import { Transform } from "raea/transform";
import { DiscComponent, RectangleComponent, Shape3d, ShapeFill, ShapeMaterial } from "raea/plugins/vector_shapes";

const material = { alpha_mode: "Blend", disable_laa: false, pipeline: "Shape3d", canvas: null, texture: null } as const;
const srgb = (red: number, green: number, blue: number) => ({ Srgba: { red, green, blue, alpha: 1 } });

function Shot() {
  return (
    <AbsoluteFill style={{ background: "#000" }}>
      <>
        <Camera3d transform={Transform.fromXyz(0, 0, 5).lookingAt([0, 0, 0])} projection={{ Perspective: { fov: 0.785398 } }} />
        <Entity position={[-1, 0, 0]}>
          <ShapeMaterial value={material} />
          <ShapeFill value={{ color: srgb(1, 0, 0), ty: "Fill" }} />
          <DiscComponent value={{ alignment: "Flat", cap: "None", arc: false, radius: 0.7, start_angle: 0, end_angle: 0 }} />
          <Shape3d value={{}} />
        </Entity>
        <Entity position={[1, 0, 0]}>
          <ShapeMaterial value={material} />
          <ShapeFill value={{ color: srgb(0, 1, 1), ty: "Fill" }} />
          <RectangleComponent value={{ alignment: "Flat", size: [1.2, 1.2], corner_radii: [0, 0, 0, 0] }} />
          <Shape3d value={{}} />
        </Entity>
      </>
    </AbsoluteFill>
  );
}
registerRoot(() => <Composition id="main" component={Shot} durationInFrames={2} fps={30} width={64} height={64} />);
