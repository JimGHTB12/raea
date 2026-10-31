// The black-frame redraw's 3D case: a box under a page of text (the 3D path, bevy_over_3d), so a redraw draws both
// again. check.sh renders it as is and with a GPU fault injected (RAEA_BLACK_FAULT).
import { AbsoluteFill, Composition, registerRoot } from "raea";
import { Canvas3D } from "raea/3d";
import { Transform } from "raea/transform";
import { Camera3d } from "raea/camera";
import { Cuboid, Mesh3d } from "raea/mesh";
import { StandardMaterial } from "raea/pbr";

function Shot() {
  return (
    <AbsoluteFill style={{ background: "#efe0b4" }}>
      <Canvas3D>
        <Camera3d transform={Transform.fromXyz(0, 0, 5).lookingAt([0, 0, 0])} />
        <Mesh3d transform={Transform.fromXyz(0, 0, 0)}><Cuboid half_size={[1, 1, 1]} /><StandardMaterial base_color="#3a7d44" unlit /></Mesh3d>
      </Canvas3D>
      <AbsoluteFill style={{ padding: 16, display: "flex", flexWrap: "wrap", gap: 6, alignContent: "flex-start", fontFamily: "sans-serif" }}>
        {Array.from({ length: 30 }, (_, i) => (
          <div key={i} style={{ padding: "4px 8px", borderRadius: 6, background: "#2d4a6b", color: "#fff", fontSize: 14 }}>label {i}</div>
        ))}
      </AbsoluteFill>
    </AbsoluteFill>
  );
}
registerRoot(() => <Composition id="main" component={Shot} durationInFrames={2} fps={30} width={640} height={360} />);
