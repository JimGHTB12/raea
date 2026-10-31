// gap: G12
// A two-frame sprite sheet (left half red, right half blue) on an ImagePlane showing frame 1: blue across the plane.
// probe: 20 32 b>200 r<60
// probe: 44 32 b>200 r<60
import { AbsoluteFill, Composition, registerRoot } from "raea";
import { Canvas3D, ImagePlane } from "raea/3d";
import { Transform } from "raea/transform";
import { Camera3d } from "raea/camera";

const SHEET = "data:image/svg+xml;utf8,<svg xmlns='http://www.w3.org/2000/svg' width='16' height='8'><rect width='8' height='8' fill='%23ff0000'/><rect x='8' width='8' height='8' fill='%230000ff'/></svg>";

function Shot() {
  return (
    <AbsoluteFill style={{ background: "#000" }}>
      <Canvas3D layered>
        <Camera3d transform={Transform.fromXyz(0, 0, 3).lookingAt([0, 0, 0])} projection={{ Perspective: { fov: 0.698131700797732 } }} />
        <ImagePlane name="sprite" src={SHEET} width={2} height={2} frame={1} grid={[2, 1]} />
      </Canvas3D>
    </AbsoluteFill>
  );
}
registerRoot(() => <Composition id="main" component={Shot} durationInFrames={2} fps={30} width={64} height={64} />);
