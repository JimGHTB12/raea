// Synthetic b-roll, from the legacy app's ad eval, kept as a P26 fixture (bench/p26-baseline/list.txt): a fictional cold brew
// can ("Plumquiver", invented) in three 5 s shots, 1080x1920. No real brand, no person, no creator's footage: it's drawn here.
// Rendered once per shot with --composition.
import { AbsoluteFill, Composition, interpolate, registerRoot, useCurrentFrame } from "raea";
import { Cylinder, Sphere } from "raea/3d";
import { AmbientLight, DirectionalLight, PointLight } from "raea/light";
import { Camera3d } from "raea/camera";
import { StandardMaterial } from "raea/pbr";
import { Quat, Transform } from "raea/transform";
import { Cuboid, Mesh3d, ConicalFrustum } from "raea/mesh";
import { LinearRgba, Srgba } from "raea/color";

const ORANGE = "#ff7a1a";
const CREAM = "#f6ead8";
const BROWN = "#3b2416";

function Can({ x = 0, z = 0, spin = 0, name }: { x?: number; z?: number; spin?: number; name: string }) {
  return (
    <>
      <Mesh3d transform={Transform.fromXyz(x, 0.62, z).withRotation(Quat.fromEuler("XYZ", 0, spin, 0))} name={`${name}-body`}>
        <Cylinder radius={0.33} half_height={0.62} />
        <StandardMaterial base_color={ORANGE} perceptual_roughness={0.35} metallic={0.35} />
      </Mesh3d>
      <Mesh3d transform={Transform.fromXyz(x, 0.7, z).withRotation(Quat.fromEuler("XYZ", 0, spin, 0))} name={`${name}-band`}>
        <Cylinder radius={0.335} half_height={0.18} />
        <StandardMaterial base_color={CREAM} perceptual_roughness={0.5} />
      </Mesh3d>
      <Mesh3d transform={Transform.fromXyz(x, 1.26, z)} name={`${name}-lid`}>
        <ConicalFrustum radius_top={0.3} radius_bottom={0.33} height={0.05} />
        <StandardMaterial base_color="#c9ccd1" perceptual_roughness={0.25} metallic={0.6} />
      </Mesh3d>
    </>
  );
}

/** Shot 1: the can turns on a warm table, the camera eases in. */
function Turn() {
  const f = useCurrentFrame();
  const z = interpolate(f, [0, 149], [4.4, 3.4]);
  return (
    <AbsoluteFill style={{ background: "#f3c89a" }}>
      <>
        <Camera3d transform={Transform.fromXyz(0, 1.2, z).lookingAt([0, 0.65, 0])} projection={{ Perspective: { fov: 0.663225115757845 } }} />
        <AmbientLight brightness={200} />
        <DirectionalLight transform={Transform.fromXyz(2.5, 4, 3).lookingAt([0, 0, 0])} illuminance={11200} shadow_maps_enabled />
        <Mesh3d transform={Transform.fromXyz(0, -0.05, 0)}>
          <Cuboid half_size={[4, 0.05, 4]} />
          <StandardMaterial base_color="#c98b5a" perceptual_roughness={0.8} />
        </Mesh3d>
        <Can name="hero" spin={f * 0.03} />
      </>
    </AbsoluteFill>
  );
}

/** Shot 2: ice cubes fall around the can, low and close. */
function Ice() {
  const f = useCurrentFrame();
  const cubes = Array.from({ length: 9 }, (_, i) => {
    const r = ((i * 9301 + 49297) % 233280) / 233280;
    const start = i * 9;
    const y = Math.max(0.12, 3 - Math.max(0, f - start) * 0.06);
    return { i, x: (r - 0.5) * 1.6 + (r > 0.5 ? 0.4 : -0.4), z: ((i * 7) % 5) * 0.18 - 0.3, y, rot: f * 0.02 + i };
  });
  return (
    <AbsoluteFill style={{ background: "#16202b" }}>
      <>
        <Camera3d transform={Transform.fromXyz(0, 0.9, 3.3).lookingAt([0, 0.6, 0])} projection={{ Perspective: { fov: 0.733038285837618 } }} />
        <AmbientLight brightness={140} />
        <DirectionalLight transform={Transform.fromXyz(-2, 4, 2).lookingAt([0, 0, 0])} illuminance={8800} shadow_maps_enabled />
        <PointLight color="#ffd2a8" transform={Transform.fromXyz(1.2, 1.4, 1.2)} intensity={2250000} />
        <Mesh3d transform={Transform.fromXyz(0, -0.05, 0)}>
          <Cuboid half_size={[4, 0.05, 4]} />
          <StandardMaterial base_color="#22313f" perceptual_roughness={0.6} />
        </Mesh3d>
        <Can name="hero" spin={0.4} />
        {cubes.map((c) => (
          <Mesh3d key={c.i} transform={Transform.fromXyz(c.x, c.y, c.z).withRotation(Quat.fromEuler("XYZ", c.rot, c.rot * 0.7, 0))} name={`ice-${c.i}`}>
            <Cuboid half_size={[0.1, 0.1, 0.1]} />
            <StandardMaterial base_color={Srgba.hex("#dff3ff").with_alpha(0.75)} perceptual_roughness={0.1} alpha_mode="Blend" />
          </Mesh3d>
        ))}
      </>
    </AbsoluteFill>
  );
}

/** Shot 3: three cans in a row, a slow dolly along them, a warm sun disc behind. */
function Row() {
  const f = useCurrentFrame();
  const x = interpolate(f, [0, 149], [-1.2, 1.2]);
  return (
    <AbsoluteFill style={{ background: "#ffb36b" }}>
      <>
        <Camera3d transform={Transform.fromXyz(x, 1.0, 4.2).lookingAt([x * 0.5, 0.7, 0])} projection={{ Perspective: { fov: 0.698131700797732 } }} />
        <AmbientLight brightness={220} />
        <DirectionalLight transform={Transform.fromXyz(3, 5, 4).lookingAt([0, 0, 0])} illuminance={10400} shadow_maps_enabled />
        <Mesh3d transform={Transform.fromXyz(0, 1.6, -3)}>
          <Sphere radius={1.1} />
          <StandardMaterial base_color="#fff1c2" emissive={LinearRgba.from(Srgba.hex("#ffd27a")).mul(1.2)} />
        </Mesh3d>
        <Mesh3d transform={Transform.fromXyz(0, -0.05, 0)}>
          <Cuboid half_size={[5, 0.05, 4]} />
          <StandardMaterial base_color={BROWN} perceptual_roughness={0.7} />
        </Mesh3d>
        <Can name="a" x={-0.9} spin={0.2} />
        <Can name="b" x={0} spin={0.6} />
        <Can name="c" x={0.9} spin={1.0} />
      </>
    </AbsoluteFill>
  );
}

registerRoot(() => (
  <>
    <Composition id="turn" component={Turn} width={1080} height={1920} fps={30} durationInFrames={150} />
    <Composition id="ice" component={Ice} width={1080} height={1920} fps={30} durationInFrames={150} />
    <Composition id="row" component={Row} width={1080} height={1920} fps={30} durationInFrames={150} />
  </>
));
