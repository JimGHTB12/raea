// A 3D scene whose holder takes its background from a class (a brand utility such as `bg-brand-bg`), not an inline
// style: the scene must still show through (motion.rs see_through). Before 2026-10-18 the class's opaque background
// painted over the 3D and every frame was that colour alone.
import { AbsoluteFill, Composition, registerRoot, useCurrentFrame } from "raea";

import { StandardMaterial } from "raea/pbr";
import { Cuboid, Mesh3d } from "raea/mesh";
import { AmbientLight, DirectionalLight } from "raea/light";
import { Quat, Transform } from "raea/transform";
import { Camera3d } from "raea/camera";

const css = ".bg-stage { background-color: #1b2230; } .title { color: #f4efe6; font: 700 64px Inter, sans-serif; }";

function Shot() {
  const f = useCurrentFrame();
  return (
    <AbsoluteFill className="bg-stage">
      <style>{css}</style>
      <>
        <Camera3d transform={Transform.fromXyz(0, 1.2, 5).lookingAt([0, 0.4, 0])} projection={{ Perspective: { fov: 0.733038285837618 } }} />
        <AmbientLight brightness={200} />
        <DirectionalLight transform={Transform.fromXyz(2, 4, 5).lookingAt([0, 0, 0])} illuminance={9600} shadow_maps_enabled />
        <Mesh3d transform={Transform.fromXyz(0, 0.5, 0).withRotation(Quat.fromEuler("XYZ", 0.4, f * 0.05, 0))} name="cube">
          <Cuboid half_size={[0.6, 0.6, 0.6]} />
          <StandardMaterial base_color="#ff7a1a" perceptual_roughness={0.5} />
        </Mesh3d>
      </>
      <div className="title" style={{ position: "absolute", left: 60, top: 40 }}>CLASSED</div>
    </AbsoluteFill>
  );
}

registerRoot(() => <Composition id="classbg" component={Shot} width={1280} height={720} fps={30} durationInFrames={60} />);
