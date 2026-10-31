// One tree (docs/sdk.feature S3a): no Canvas3D; PascalCase entities sit in the page's tree, and a page element inside
// an entity (the div in the Mesh3d) is a page layer in the world at it, which the cube occludes. Render equals its
// baked play (check.sh).
import { AbsoluteFill, Composition, registerRoot } from "raea";
import { Mesh3d, Cuboid } from "raea/mesh";
import { Camera3d } from "raea/camera";
import { AmbientLight, DirectionalLight } from "raea/light";
import { Transform } from "raea/transform";
import { StandardMaterial } from "raea/pbr";

function Shot() {
  return (
    <AbsoluteFill style={{ background: "#e8eef6" }}>
      <Camera3d transform={Transform.fromXyz(0, 1, 5).lookingAt([0, 0, 0])} />
      <AmbientLight brightness={300} />
      <DirectionalLight transform={Transform.fromXyz(3, 4, 5).lookingAt([0, 0, 0])} illuminance={4000} />
      <Mesh3d transform={Transform.fromXyz(-1.2, 0, 0)}>
        <Cuboid half_size={[0.6, 0.6, 0.6]} />
        <StandardMaterial base_color="#c84" />
        <div style={{ width: 320, height: 200, background: "#1d2a44", color: "white", font: "800 48px Inter", display: "flex", alignItems: "center", justifyContent: "center" }}>HELLO</div>
      </Mesh3d>
    </AbsoluteFill>
  );
}
registerRoot(() => <Composition id="onetree" component={Shot} width={960} height={540} fps={30} durationInFrames={30} />);
