// docs/review.feature R2e: every layer kind in one frame, overlapping, so the layer pass is checked against the oracle
// (hide one layer, render again, the pixels that change) where layers meet: a square drawn as a 3D mesh, as a React
// component (used twice) and as a page element, text, an image, a rotated element and a .map() list. The page is over
// the 3D, as a composition's always is. bench/layers/oracle.sh renders it.
import { AbsoluteFill, Composition, Img, registerRoot, useCurrentFrame } from "raea";
import { StandardMaterial } from "raea/pbr";
import { Cuboid, Mesh3d } from "raea/mesh";
import { DirectionalLight } from "raea/light";
import { Quat, Transform } from "raea/transform";
import { Camera3d } from "raea/camera";
import { Badge } from "./badge";

const Shot = () => {
  const f = useCurrentFrame();
  return (
    <AbsoluteFill style={{ background: "#16202e" }}>
      <Camera3d transform={Transform.fromXyz(0, 0, 6).lookingAt([0, 0, 0])} />
      <DirectionalLight illuminance={9000} transform={Transform.fromXyz(2, 4, 6).lookingAt([0, 0, 0])} />
      <Mesh3d transform={Transform.fromXyz(-0.6 + f * 0.01, 0.3, 0).withRotation(Quat.fromEuler("XYZ", 0.4, f * 0.02, 0))}>
        <Cuboid half_size={[0.7, 0.7, 0.7]} />
        <StandardMaterial base_color="#2fbf71" />
      </Mesh3d>
      <Mesh3d transform={Transform.fromXyz(1.4, -1.2, -1)}>
        <Cuboid half_size={[0.5, 0.5, 0.5]} />
        <StandardMaterial base_color="#e0a82e" />
      </Mesh3d>
      <Badge x={150 + f} y={150} color="#3366ff" />
      <Badge x={300} y={60} color="#cc3377" />
      <div style={{ position: "absolute", left: 210, top: 210, width: 90, height: 90, background: "#ff6a00" }} />
      <h1 style={{ position: "absolute", left: 24, top: 8, margin: 0, color: "#f4f4f4", fontSize: 30 }}>Layers</h1>
      <Img src="pic.png" style={{ position: "absolute", left: 20, top: 300, width: 96, height: 64 }} />
      <div style={{ position: "absolute", left: 330, top: 300, width: 80, height: 40, background: "#8a5cf6", transform: `rotate(${20 + f}deg)` }} />
      <ul style={{ position: "absolute", left: 20, top: 70, margin: 0, padding: 0, listStyle: "none" }}>
        {["one", "two", "three"].map((t) => (
          <li key={t} style={{ width: 70, height: 22, marginBottom: 6, background: "#2a3a52", color: "#cfe", fontSize: 14 }}>{t}</li>
        ))}
      </ul>
    </AbsoluteFill>
  );
};
registerRoot(() => <Composition id="layers" component={Shot} width={480} height={400} fps={30} durationInFrames={90} />);
