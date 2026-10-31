// One tree's surface mode (docs/sdk.feature S3a, B3 H5): a page element with layer="surface" inside a Mesh3d is drawn
// on the mesh itself, over its UVs (each face of the cube shows the whole page), turned with the cube. Render equals
// its baked play (check.sh).
import { AbsoluteFill, Composition, registerRoot, useCurrentFrame } from "raea";
import { Mesh3d, Cuboid } from "raea/mesh";
import { Camera3d } from "raea/camera";
import { AmbientLight, DirectionalLight } from "raea/light";
import { Transform, Quat } from "raea/transform";
import { StandardMaterial } from "raea/pbr";

function Shot() {
  const frame = useCurrentFrame();
  return (
    <AbsoluteFill style={{ background: "#e8eef6" }}>
      <Camera3d transform={Transform.fromXyz(0, 1.2, 4).lookingAt([0, 0, 0])} />
      <AmbientLight brightness={300} />
      <DirectionalLight transform={Transform.fromXyz(3, 4, 5).lookingAt([0, 0, 0])} illuminance={4000} />
      <Mesh3d transform={Transform.fromXyz(0, 0, 0).withRotation(Quat.fromRotationY(0.6 + frame * 0.02))}>
        <Cuboid half_size={[0.8, 0.8, 0.8]} />
        <StandardMaterial base_color="#c84" />
        <div layer="surface" style={{ width: 256, height: 256, background: "#1d2a44", color: "white", font: "800 64px Inter", display: "flex", alignItems: "center", justifyContent: "center" }}>
          {frame}
        </div>
      </Mesh3d>
    </AbsoluteFill>
  );
}
registerRoot(() => <Composition id="surface" component={Shot} width={960} height={540} fps={30} durationInFrames={30} />);
