// gap: G4
// Picking: the pointer at the frame's middle hits the cube named "target", so the cube turns green.
// probe: 32 32 g>150 r<100
import { AbsoluteFill, Composition, registerRoot } from "raea";
import { Canvas3D, pick } from "raea/3d";
import { Camera3d } from "raea/camera";
import { Transform } from "raea/transform";
import { StandardMaterial } from "raea/pbr";
import { Cuboid, Mesh3d } from "raea/mesh";

const CAMERA = { position: [0, 0, 5] as [number, number, number], lookAt: [0, 0, 0] as [number, number, number], fov: 40 };

function Shot() {
  const hit = pick({ x: 32, y: 32 }, CAMERA, [{ name: "target", box: { min: [-0.5, -0.5, -0.5], max: [0.5, 0.5, 0.5] } }], { width: 64, height: 64 });
  return (
    <AbsoluteFill style={{ background: "#000" }}>
      <Canvas3D layered>
        <Camera3d transform={Transform.fromXyz(...CAMERA.position).lookingAt(CAMERA.lookAt)} projection={{ Perspective: { fov: (CAMERA.fov * Math.PI) / 180 } }} />
        <Mesh3d name="target"><Cuboid half_size={[0.5, 0.5, 0.5]} /><StandardMaterial base_color={hit?.name === "target" ? "#00ff00" : "#ff0000"} unlit /></Mesh3d>
      </Canvas3D>
    </AbsoluteFill>
  );
}
registerRoot(() => <Composition id="main" component={Shot} durationInFrames={2} fps={30} width={64} height={64} />);
