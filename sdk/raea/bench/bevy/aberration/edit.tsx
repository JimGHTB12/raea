// Aberration fixture: a white paint-look sky behind a few shapes, depth of field on the camera, and a <Grade>'s
// aberration keyed 0 -> 0.03 -> 0 (a glitch's spike, as the film's FilmLook does). Its colour fringes are a function
// of the frame alone: a frame reached after the spike equals the same frame rendered by itself, and the sky stays
// white throughout (check.sh beside this compares them).
import { AbsoluteFill, Composition, interpolate, registerRoot, useCurrentFrame } from "raea";
import { DepthOfField, Grade, Look, Sphere } from "raea/3d";
import { StandardMaterial } from "raea/pbr";
import { Cuboid, Mesh3d } from "raea/mesh";
import { DirectionalLight } from "raea/light";
import { Transform } from "raea/transform";
import { Camera3d } from "raea/camera";

const FPS = 30;
const FRAMES = 60;

function Shot() {
  const f = useCurrentFrame();
  const ab = interpolate(f, [0, 20, 30, 40, FRAMES - 1], [0, 0, 0.03, 0, 0], { extrapolateRight: "clamp" });
  return (
    <AbsoluteFill style={{ background: "#ffffff" }}>
      <>
        <Camera3d transform={Transform.fromXyz(0, 2, 8).lookingAt([0, 1, 0])} projection={{ Perspective: { fov: 0.698131700797732 } }}><DepthOfField focal_distance={8} aperture_f_stops={8} /></Camera3d>
        <Look mode="paint" bands={3} shadow="#4a5590" mid="#c98a74" sky={["#f4f4f4", "#f4f4f4"]} />
        <Grade temperature={0.03} vignette={0.2} aberration={Math.max(ab, 0.0001)} />
        <DirectionalLight transform={Transform.fromXyz(-4, 3, 3).lookingAt([0, 0, 0])} illuminance={9600} shadow_maps_enabled />
        <Mesh3d transform={Transform.fromXyz(0, 1, 0)} name="tower"><Cuboid half_size={[0.5, 1, 0.5]} /><StandardMaterial base_color="#d8463a" /></Mesh3d>
        <Mesh3d transform={Transform.fromXyz(-1.8, 0.6, 0)} name="ball"><Sphere radius={0.6} /><StandardMaterial base_color="#f2b632" /></Mesh3d>
      </>
    </AbsoluteFill>
  );
}

registerRoot(() => <Composition id="aberration" component={Shot} width={640} height={360} fps={FPS} durationInFrames={FRAMES} />);
