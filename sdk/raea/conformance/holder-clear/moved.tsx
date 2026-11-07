// The 3D moved at frame 5 from under a red page element to under a blue one (holder-clear.test.tsx): the holders are
// worked out again when the page's elements move, so the 3D clears to its new holder's background.
import { Composition, registerRoot, useCurrentFrame } from "raea";
import { Camera3d } from "raea/camera";
import { Cuboid, Mesh3d } from "raea/mesh";
import { StandardMaterial } from "raea/pbr";
import { Transform } from "raea/transform";

const Cube = () => (
  <>
    <Camera3d transform={Transform.IDENTITY} />
    <Mesh3d transform={Transform.fromXyz(0, 0, -5)}>
      <Cuboid half_size={[0.5, 0.5, 0.5]} />
      <StandardMaterial base_color="#ffffff" />
    </Mesh3d>
  </>
);

const Shot = () => {
  const f = useCurrentFrame();
  return (
    <>
      <div key="red" style={{ position: "absolute", left: 0, top: 0, width: 0, height: 0, background: "#ff0000" }}>{f < 5 ? <Cube /> : null}</div>
      <div key="blue" style={{ position: "absolute", left: 0, top: 0, width: 0, height: 0, background: "#0000ff" }}>{f < 5 ? null : <Cube />}</div>
    </>
  );
};

registerRoot(() => <Composition id="main" component={Shot} durationInFrames={10} fps={30} width={160} height={90} />);
