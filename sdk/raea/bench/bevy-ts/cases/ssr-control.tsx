// gap: control
// SSR control: the ssr case without the camera components; the floor below the cube stays dark.
// DeferredPrepass, Msaa off and ScreenSpaceReflections through the TypeRegistry): a mirror-smooth dark floor under a
// red emissive cube reflects it, so the floor just below the cube turns red. ssr-control.tsx is the same without them.
// probe: 32 44 r<120
import { AbsoluteFill, Composition, registerRoot } from "raea";
import { Canvas3D } from "raea/3d";
import { LinearRgba, Srgba } from "raea/color";
import { StandardMaterial } from "raea/pbr";
import { Cuboid, Mesh3d } from "raea/mesh";
import { Transform } from "raea/transform";
import { Camera3d } from "raea/camera";

function Shot() {
  return (
    <AbsoluteFill style={{ background: "#000" }}>
      <Canvas3D layered>
        <Camera3d transform={Transform.fromXyz(0, 1.2, 4).lookingAt([0, 0.3, 0])} projection={{ Perspective: { fov: 0.698131700797732 } }} />
        <Mesh3d transform={Transform.fromXyz(0, 0.6, 0)} name="cube"><Cuboid half_size={[0.5, 0.5, 0.5]} /><StandardMaterial base_color="#ff0000" emissive={LinearRgba.from(Srgba.hex("#ff0000")).mul(4)} /></Mesh3d>
        <Mesh3d name="floor"><Cuboid half_size={[4, 0.05, 4]} /><StandardMaterial base_color="#ffffff" perceptual_roughness={0.2} metallic={1} /></Mesh3d>
      </Canvas3D>
    </AbsoluteFill>
  );
}
registerRoot(() => <Composition id="main" component={Shot} durationInFrames={2} fps={30} width={64} height={64} />);
