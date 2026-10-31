// Layers fixture: a layered Canvas3D between page elements. An orange disc and a label before the Canvas3D are under
// the 3D (seen around the sphere, hidden by it); a green bar after it is over the sphere. The page's own background is
// under everything. layers/check.py measures occlusion and that the page under the 3D is the page alone where nothing
// covers it; render equals its baked play (check.sh).
import { AbsoluteFill, Composition, interpolate, registerRoot, useCurrentFrame } from "raea";
import { Canvas3D, Sphere } from "raea/3d";
import { StandardMaterial } from "raea/pbr";
import { Mesh3d } from "raea/mesh";
import { AmbientLight, DirectionalLight } from "raea/light";
import { Transform } from "raea/transform";
import { Camera3d } from "raea/camera";

const FPS = 30;
const FRAMES = 60;

function Shot() {
  const f = useCurrentFrame();
  const x = interpolate(f, [0, FRAMES - 1], [-0.6, 0.6]);
  return (
    <AbsoluteFill style={{ background: "#e8eef6" }}>
      {/* under the 3D */}
      <div style={{ position: "absolute", left: 300, top: 90, width: 360, height: 360, borderRadius: 180, background: "#ff8a1f" }} />
      <div style={{ position: "absolute", left: 40, top: 40, font: "800 44px Inter", color: "#1d2a44" }}>UNDER</div>
      <Canvas3D layered>{/* layers:3d */}
        <Camera3d transform={Transform.fromXyz(0, 0, 6).lookingAt([0, 0, 0])} projection={{ Perspective: { fov: 0.698131700797732 } }} />
        <AmbientLight brightness={160} />
        <DirectionalLight transform={Transform.fromXyz(3, 4, 5).lookingAt([0, 0, 0])} illuminance={9600} shadow_maps_enabled />
        <Mesh3d transform={Transform.fromXyz(x, 0, 0)} name="ball">
          <Sphere radius={1.1} />
          <StandardMaterial base_color="#3a6cff" perceptual_roughness={0.4} />
        </Mesh3d>
      </Canvas3D>{/* layers:3d */}
      {/* over the 3D */}
      <div style={{ position: "absolute", left: 0, top: 250, width: 960, height: 40, background: "#22b573" }} />
      <div style={{ position: "absolute", right: 40, bottom: 40, font: "800 44px Inter", color: "#1d2a44" }}>OVER</div>
    </AbsoluteFill>
  );
}

registerRoot(() => <Composition id="layers" component={Shot} width={960} height={540} fps={FPS} durationInFrames={FRAMES} />);
