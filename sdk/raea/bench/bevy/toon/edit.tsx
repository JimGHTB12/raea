// Toon fixture: MeshToonMaterial with ink outlines on a Manifold banana (an ellipsoid bent by a warp), a
// sphere, a torus and a box (hard edges: the outline must stay closed), and a second, far sphere like the first (three
// bands; its outline must be as wide in pixels). One key light with shadows and an ambient fill. The banana turns, the box's
// colour blends in Oklab through an `animate` track and its outline thickens; the torus has three bands and a
// highlight. Render and the baked play with its 3D must agree (check.sh); toon/check.py measures the bands, the
// outline and determinism.
import { AbsoluteFill, Composition, registerRoot, track, useCurrentFrame } from "raea";
import { Sphere, Torus, manifold } from "raea/3d";
import { Cuboid, Mesh3d } from "raea/mesh";
import { AmbientLight, DirectionalLight } from "raea/light";
import { Quat, Transform } from "raea/transform";
import { Camera3d } from "raea/camera";
import { ExtendedMaterial, StandardMaterial, ToonExtension } from "raea/pbr";

const FPS = 30;
const FRAMES = 60;

// the banana: a smoothed sphere stretched long, refined, bent along an arc and pinched at the ends (warp), with a stem
const BANANA = manifold.sphere(1).smooth({ refine: 4 }).scale(1.25, 0.3, 0.3).refineToLength(0.06).warp((x, y, z) => {
  const t = x / 1.25;
  const pinch = 1 - 0.35 * t * t * t * t;
  return [x, y * pinch + 0.55 * t * t - 0.3, z * pinch];
}).union(manifold.cylinder(0.055, 0.32).rotate(0, 90, 0).rotate(0, 0, 28).move(1.18, 0.3, 0));

const BOX = {
  base: {
    base_color: track({ 0: "#7bd389", 59: "#5a6cff" }, "in-out-cubic"),
  },
  extension: {
    outline_width: track({ 0: 2, 59: 4 }, "linear"),
  },
};

function Shot() {
  const f = useCurrentFrame();
  return (
    <AbsoluteFill style={{ background: "#f3efe6" }}>
      <>
        <Camera3d transform={Transform.fromXyz(0, 0.9, 7.5).lookingAt([0, 0.2, 0])} projection={{ Perspective: { fov: 0.698131700797732 } }} />
        <AmbientLight brightness={140} />
        <DirectionalLight transform={Transform.fromXyz(4, 5, 3).lookingAt([0, 0, 0])} illuminance={9600} shadow_maps_enabled />
        <Mesh3d transform={Transform.fromXyz(-2.2, 0.2, 0).withRotation(Quat.fromEuler("XYZ", 0.25, f * 0.05, 0.2))} name="banana" shape={BANANA}>
          <ExtendedMaterial base={<StandardMaterial base_color="#ffd23f" />} extension={<ToonExtension steps={2} rim={{ intensity: 0.7, color: "#fff6c8", width: 0.22 }} outline={{ width: 3 }} />} />
        </Mesh3d>
        <Mesh3d transform={Transform.fromXyz(0.2, 0.3, 0)} name="near">
          <Sphere radius={0.75} />
          <ExtendedMaterial base={<StandardMaterial base_color="#3aa0ff" />} extension={<ToonExtension steps={2} rim={{ intensity: 0.6, width: 0.2 }} outline={{ width: 3 }} />} />
        </Mesh3d>
        <Mesh3d transform={Transform.fromXyz(1.4, 1.6, -9)} name="far">
          <Sphere radius={0.75} />
          <ExtendedMaterial base={<StandardMaterial base_color="#3aa0ff" />} extension={<ToonExtension steps={3} outline={{ width: 3 }} />} />
        </Mesh3d>
        <Mesh3d transform={Transform.fromXyz(2.4, 0.1, 0.3).withRotation(Quat.fromEuler("XYZ", 0.9 + f * 0.03, 0.4, 0))} name="torus">
          <Torus major_radius={0.6} minor_radius={0.2} />
          <ExtendedMaterial base={<StandardMaterial base_color="#ff5a7a" />} extension={<ToonExtension steps={3} highlight={{ size: 0.25 }} outline={{ width: 2, color: "#2a0a14" }} />} />
        </Mesh3d>
        <Mesh3d transform={Transform.fromXyz(0.4, -1.25, 0.8).withRotation(Quat.fromEuler("XYZ", 0.5, 0.7 + f * 0.02, 0))} name="box">
          <Cuboid half_size={[0.35, 0.35, 0.35]} />
          <ExtendedMaterial base={<StandardMaterial base_color="#7bd389" animate={BOX.base} />} extension={<ToonExtension steps={2} outline={{ width: 2 }} animate={BOX.extension} />} />
        </Mesh3d>
      </>
    </AbsoluteFill>
  );
}

registerRoot(() => <Composition id="toon" component={Shot} width={960} height={540} fps={FPS} durationInFrames={FRAMES} />);
