// Bevy v0.20.0 examples/movement/smooth_follow.rs in raea's TSX (docs/control.feature FC0a): a 12 m plane of
// srgb(0.3, 0.15, 0.3) at y = -2.5, a blue 0.3 m sphere walking at 5 m/s to random points in a 4 m box and a red one
// smooth_nudging after it (bench/bevy-parity/lib/smooth_follow.ts, bevy_math's run bit for bit), a shadowed
// 15 000 000 lm PointLight at (4, 8, 4), and the camera at (-2, 3, 5) looking at the origin, in Bevy's default
// 1280 x 720 window.
import { AbsoluteFill, Composition, registerRoot, useCurrentFrame } from "raea";
import { Canvas3D } from "raea/3d";
import { Transform } from "raea/transform";
import { Camera3d } from "raea/camera";
import { Mesh3d, Plane3d, Sphere } from "raea/mesh";
import { StandardMaterial } from "raea/pbr";
import { PointLight } from "raea/light";
import { smoothFollow } from "../../../../../bench/bevy-parity/lib/smooth_follow";

function Shot() {
  const [target, follow] = smoothFollow(useCurrentFrame());
  return (
    <AbsoluteFill>
      <Canvas3D layered clearColor="rgb(43, 44, 47)">
        <Mesh3d transform={Transform.fromXyz(0, -2.5, 0)}>
          <Plane3d half_size={[6, 6]} />
          <StandardMaterial base_color="rgb(76.5, 38.25, 76.5)" />
        </Mesh3d>
        <Mesh3d transform={Transform.fromTranslation(target)}>
          <Sphere radius={0.3} />
          <StandardMaterial base_color="rgb(76.5, 38.25, 229.5)" />
        </Mesh3d>
        <Mesh3d transform={Transform.fromTranslation(follow)}>
          <Sphere radius={0.3} />
          <StandardMaterial base_color="rgb(229.5, 76.5, 76.5)" />
        </Mesh3d>
        <PointLight intensity={15_000_000} shadow_maps_enabled transform={Transform.fromXyz(4, 8, 4)} />
        <Camera3d transform={Transform.fromXyz(-2, 3, 5).lookingAt([0, 0, 0])} />
      </Canvas3D>
    </AbsoluteFill>
  );
}

registerRoot(() => <Composition id="main" component={Shot} durationInFrames={181} fps={60} width={1280} height={720} />);
