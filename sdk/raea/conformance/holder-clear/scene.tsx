// A 3D shot under a page element whose background turns from red to blue at frame 5 (holder-clear.test.tsx): the
// 3D clears to its holder's background, frame by frame.
import { Composition, registerRoot, useCurrentFrame } from "raea";
import { Camera3d } from "raea/camera";
import { Cuboid, Mesh3d } from "raea/mesh";
import { StandardMaterial } from "raea/pbr";
import { Transform } from "raea/transform";

const Shot = () => {
  const f = useCurrentFrame();
  return (
    <div style={{ position: "absolute", inset: 0, background: f < 5 ? "#ff0000" : "#0000ff" }}>
      <Camera3d transform={Transform.IDENTITY} />
      <Mesh3d transform={Transform.fromXyz(0, 0, -5)}>
        <Cuboid half_size={[0.5, 0.5, 0.5]} />
        <StandardMaterial base_color="#ffffff" />
      </Mesh3d>
    </div>
  );
};

registerRoot(() => <Composition id="main" component={Shot} durationInFrames={10} fps={30} width={160} height={90} />);
