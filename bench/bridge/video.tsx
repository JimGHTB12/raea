// B3 H11's video case (bench/bridge/run.sh --pixels): a <Video> inside a plain App's page, over its 3D, against raea's
// render of the same frame. The clip (can.mp4, copied beside the bundle) is at its frame for the composition's.
import { AbsoluteFill, Composition, Video, registerRoot } from "raea";
import { Canvas3D } from "raea/3d";
import { StandardMaterial } from "raea/pbr";
import { Cuboid, Mesh3d } from "raea/mesh";
import { AmbientLight } from "raea/light";
import { Transform } from "raea/transform";
import { Camera3d } from "raea/camera";

function Shot() {
  return (
    <AbsoluteFill style={{ backgroundColor: "#1b2230" }}>
      <Canvas3D>
        <Camera3d transform={Transform.fromXyz(0, 0, 5).lookingAt([0, 0, 0])} projection={{ Perspective: { fov: 0.733038285837618 } }} />
        <AmbientLight brightness={400} />
        <Mesh3d transform={Transform.fromXyz(2, 0, 0)}>
          <Cuboid half_size={[0.5, 0.5, 0.5]} />
          <StandardMaterial base_color="#ff7a1a" />
        </Mesh3d>
      </Canvas3D>
      <Video src="can.mp4" style={{ position: "absolute", left: 80, top: 60, width: 320, height: 568, objectFit: "cover" }} />
    </AbsoluteFill>
  );
}

registerRoot(() => <Composition id="bridgevideo" component={Shot} width={1280} height={720} fps={30} durationInFrames={60} />);
