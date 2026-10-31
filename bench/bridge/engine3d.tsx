// docs/sdk.feature S2d's 3D World split (on by default; RAEA_ENGINE_READS=0 turns it off): React's useQuery reads the
// Engine's World, where the composition's <Mesh3d> is. The cube moves to x = 0.5 + frame / 100, a value no other entity
// has; the marker is green when the query's answer has a Transform at most 5 frames behind it (reads are a frame or two
// behind), red when it answers without one, blue before any answer (React's App's World has no mesh, so without the
// setting it stays blue or red). 10-25: a 40-frame render with the setting is blue at frame 0 and green from frame 1,
// the same frames twice; without it, blue throughout. A render of frame 20 alone (--frame 20) was red: React laid out
// frame 20's page before the Engine stepped 0..20. Since bcb743d7 the Engine steps frame f - 1 and its pushes reach
// React before f is asked for (motion.rs engine_before): --frame 20 is green, as are frames 1-39 of the PNG-frames and
// MP4 renders.
import { AbsoluteFill, Composition, registerRoot, useCurrentFrame } from "raea";
import { component } from "raea/bevy";
import { entity, read, useQuery } from "raea/ecs";
import { Mesh3d, Cuboid } from "raea/mesh";
import { Camera3d } from "raea/camera";
import { AmbientLight } from "raea/light";
import { Transform } from "raea/transform";
import { StandardMaterial } from "raea/pbr";

const Tf = component<{ translation: [number, number, number] }>("bevy_transform::components::transform::Transform");

function Shot() {
  const frame = useCurrentFrame();
  const x = 0.5 + frame / 100;
  const rows = useQuery(entity, read(Tf));
  const seen = rows.some(([, t]) => x - t.translation[0] >= -1e-4 && x - t.translation[0] <= 0.05 + 1e-4);
  const marker = rows.length === 0 ? "#0040ff" : seen ? "#00c040" : "#ff2020";
  return (
    <AbsoluteFill style={{ background: "#202020" }}>
      <Camera3d transform={Transform.fromXyz(0, 0, 6).lookingAt([0, 0, 0])} />
      <AmbientLight brightness={800} />
      <Mesh3d transform={Transform.fromXyz(x, 0, 0)}>
        <Cuboid half_size={[0.4, 0.4, 0.4]} />
        <StandardMaterial base_color="#c84" />
      </Mesh3d>
      <div id="marker" style={{ position: "absolute", left: 0, top: 0, width: 40, height: 40, background: marker }} />
    </AbsoluteFill>
  );
}

registerRoot(() => <Composition id="engine3d" component={Shot} width={320} height={180} fps={30} durationInFrames={40} />);
