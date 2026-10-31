// on* on 3D parts under a camera in a page element (docs/sdk.feature S3a): pick3d's scene in a 480 x 270 box at
// (440, 240), so its view, and its picking, are the box's. check.sh renders input.json's pointer over the cube's and
// the ball's places in the box; a pick in the whole frame's px misses both.
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
      <div style={{ position: "absolute", left: 440, top: 240, width: 480, height: 270, background: "#e8eef6" }}>
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
      </div>
    </AbsoluteFill>
  );
}

registerRoot(() => <Composition id="pick3dbox" component={Shot} width={960} height={540} fps={30} durationInFrames={30} />);
