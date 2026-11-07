// A white box filling the view, lit only by an AmbientLight: brightness 0 in "dark", Bevy's 80 in "lit"
// (ambient-zero.test.tsx).
import { Composition, registerRoot } from "raea";
import { Camera3d } from "raea/camera";
import { AmbientLight } from "raea/light";
import { Cuboid, Mesh3d } from "raea/mesh";
import { StandardMaterial } from "raea/pbr";
import { Transform } from "raea/transform";

const Shot = ({ brightness }: { brightness: number }) => (
  <>
    <Camera3d transform={Transform.IDENTITY} />
    <AmbientLight brightness={brightness} />
    <Mesh3d transform={Transform.fromXyz(0, 0, -5)}>
      <Cuboid half_size={[3, 3, 3]} />
      <StandardMaterial base_color="#ffffff" />
    </Mesh3d>
  </>
);

registerRoot(() => (
  <>
    <Composition id="dark" component={() => <Shot brightness={0} />} durationInFrames={1} fps={30} width={160} height={90} />
    <Composition id="lit" component={() => <Shot brightness={80} />} durationInFrames={1} fps={30} width={160} height={90} />
  </>
));
