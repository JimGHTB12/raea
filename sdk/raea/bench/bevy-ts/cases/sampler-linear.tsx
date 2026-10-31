// gap: SH5
// How a map is read (a texture field's { path, sampler }, raea-sys-sampler): checker2.png, a 2 x 2 checker (white
// top left), stretched over a 2 m square that fills most of the frame, filter "linear": smoothed (the control): the middle is grey.
// probe: 32 20 r>60 r<200
// probe: 32 44 r>60 r<200
import { AbsoluteFill, Composition, registerRoot } from "raea";
import { Canvas3D } from "raea/3d";
import { Mesh3d, Plane3d } from "raea/mesh";
import { Transform } from "raea/transform";
import { Camera3d } from "raea/camera";
import { StandardMaterial } from "raea/pbr";

function Shot() {
  return (
    <AbsoluteFill style={{ background: "#000" }}>
      <Canvas3D>
        <Camera3d transform={Transform.fromXyz(0, 0, 3).lookingAt([0, 0, 0])} projection={{ Perspective: { fov: 0.698131700797732 } }} />
        <Mesh3d>
          <Plane3d normal={[0, 0, 1]} half_size={[1, 1]} />
          <StandardMaterial base_color="#cccccc" base_color_texture={{ path: "checker2.png", sampler: { filter: "linear" } }} unlit />
        </Mesh3d>
      </Canvas3D>
    </AbsoluteFill>
  );
}
registerRoot(() => <Composition id="main" component={Shot} durationInFrames={2} fps={30} width={64} height={64} />);
