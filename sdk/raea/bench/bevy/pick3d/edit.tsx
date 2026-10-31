// on* on 3D parts (docs/sdk.feature S3a): a click on the cube turns it green, the pointer over the sphere turns it
// orange and leaving turns it back; no Canvas3D. pick3d/check.sh renders a run of frames from input.json (the pointer
// over the cube, pressed and let go, then over the sphere, then away) and checks each part's colour on the way.
import { useState } from "react";
import { AbsoluteFill, Composition, registerRoot } from "raea";
import { Sphere } from "raea/3d";
import { StandardMaterial } from "raea/pbr";
import { Cuboid, Mesh3d } from "raea/mesh";
import { AmbientLight, DirectionalLight } from "raea/light";
import { Transform } from "raea/transform";
import { Camera3d } from "raea/camera";

function Shot() {
  const [on, setOn] = useState(false);
  const [hot, setHot] = useState(false);
  return (
    <AbsoluteFill style={{ background: "#e8eef6" }}>
      <Camera3d transform={Transform.fromXyz(0, 0, 6).lookingAt([0, 0, 0])} projection={{ Perspective: { fov: 0.698131700797732 } }} />
      <AmbientLight brightness={160} />
      <DirectionalLight transform={Transform.fromXyz(3, 4, 5).lookingAt([0, 0, 0])} illuminance={9600} shadow_maps_enabled />
      <Mesh3d transform={Transform.fromXyz(-1.5, 0, 0)} name="cube" onClick={() => setOn((v) => !v)}>
        <Cuboid half_size={[0.6, 0.6, 0.6]} />
        <StandardMaterial base_color={on ? "#22b573" : "#3a6cff"} unlit />
      </Mesh3d>
      <Mesh3d transform={Transform.fromXyz(1.5, 0, 0)} name="ball" onPointerOver={() => setHot(true)} onPointerOut={() => setHot(false)}>
        <Sphere radius={0.7} />
        <StandardMaterial base_color={hot ? "#ff8a1f" : "#9a9aa8"} unlit />
      </Mesh3d>
    </AbsoluteFill>
  );
}

registerRoot(() => <Composition id="pick3d" component={Shot} width={960} height={540} fps={30} durationInFrames={30} />);
