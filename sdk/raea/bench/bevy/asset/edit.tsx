// An asset loaded before the frame that uses it (docs/sdk.feature S2b useAsset): dot.png's size (64 x 32) read with
// useAsset sizes the bar (four px a px of it: 256 x 128), beside a cube. check.sh renders it and plays it baked:
// render = play; the bar's edge at x 40 + 256 says the size was there on the first frame.
import { AbsoluteFill, Composition, registerRoot } from "raea";
import { asset, useAsset } from "@raea/app/ecs";

import { StandardMaterial } from "raea/pbr";
import { Cuboid, Mesh3d } from "raea/mesh";
import { AmbientLight } from "raea/light";
import { Transform } from "raea/transform";
import { Camera3d } from "raea/camera";

const dot = asset<"image">("dot.png");

function Shot() {
  const d = useAsset(dot);
  return (
    <AbsoluteFill style={{ background: "#e8eef6" }}>
      <Camera3d transform={Transform.fromXyz(0, 0, 6).lookingAt([0, 0, 0])} projection={{ Perspective: { fov: 0.698131700797732 } }} />
      <AmbientLight brightness={160} />
      <Mesh3d transform={Transform.fromXyz(1.5, 0, 0)}>
        <Cuboid half_size={[0.6, 0.6, 0.6]} />
        <StandardMaterial base_color="#3a6cff" unlit />
      </Mesh3d>
      <div style={{ position: "absolute", left: 40, top: 40, width: (d.width ?? 0) * 4, height: (d.height ?? 0) * 4, background: "#ff6a00" }} />
    </AbsoluteFill>
  );
}

registerRoot(() => <Composition id="asset" component={Shot} width={960} height={540} fps={30} durationInFrames={30} />);
