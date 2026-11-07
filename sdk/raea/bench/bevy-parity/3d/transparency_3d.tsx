// Bevy v0.20.0 examples/3d/transparency_3d.rs in raea's TSX (docs/control.feature FC0a): a 6 m green plane; three ico-3
// spheres of radius 0.5 at z -1.5 (masked at 0.5, masked at 0.1 and unlit, and an opaque red one) and two 1 m cubes at
// z 0 (blended, as Bevy's From<Color> makes a colour with alpha under 1, and alpha-to-coverage), every material's alpha
// set to sin(elapsed) / 2 + 0.5 each update (fade_transparency), a shadow-casting PointLight at (4, 8, 4) and the camera at
// (-2, 3, 5) looking at the origin, in Bevy's default 1280 x 720 window.
// Open (frame 10 fails, worst tile 25): the AlphaToCoverage cube's samples it leaves uncovered come out near the clear
// colour in raea (grey ~45, with RAEA_READBACK=rgba too) where Bevy's show the floor (~51), so the cube reads darker.
import { Composition, registerRoot, useCurrentFrame } from "raea";
import { Transform } from "raea/transform";
import { Camera3d } from "raea/camera";
import { Srgba } from "raea/color";
import { Cuboid, Mesh3d, Plane3d, Sphere } from "raea/mesh";
import { StandardMaterial } from "raea/pbr";
import { PointLight } from "raea/light";

function Shot() {
  // elapsed: f updates of 16_666_667 ns (none on the first), as f32
  const t = Math.fround(useCurrentFrame() * 16_666_667e-9);
  const a = Math.fround(Math.fround(Math.sin(t)) / 2 + 0.5);
  const c = (r: number, g: number, b: number) => new Srgba(r, g, b, a);
  return (
    <>
      <Mesh3d>
        <Plane3d half_size={[3, 3]} />
        <StandardMaterial base_color={c(0.3, 0.5, 0.3)} />
      </Mesh3d>
      <Mesh3d transform={Transform.fromXyz(1, 0.5, -1.5)}>
        <Sphere radius={0.5} ico={3} />
        <StandardMaterial base_color={c(0.2, 0.7, 0.1)} alpha_mode={{ Mask: 0.5 }} />
      </Mesh3d>
      <Mesh3d transform={Transform.fromXyz(-1, 0.5, -1.5)}>
        <Sphere radius={0.5} ico={3} />
        <StandardMaterial base_color={c(0.2, 0.7, 0.1)} alpha_mode={{ Mask: 0.1 }} unlit />
      </Mesh3d>
      <Mesh3d transform={Transform.fromXyz(0, 0.5, 0)}>
        <Cuboid half_size={[0.5, 0.5, 0.5]} />
        <StandardMaterial base_color={c(0.5, 0.5, 1)} alpha_mode="Blend" />
      </Mesh3d>
      <Mesh3d transform={Transform.fromXyz(-1.5, 0.5, 0)}>
        <Cuboid half_size={[0.5, 0.5, 0.5]} />
        <StandardMaterial base_color={c(0.5, 1, 0.5)} alpha_mode="AlphaToCoverage" />
      </Mesh3d>
      <Mesh3d transform={Transform.fromXyz(0, 0.5, -1.5)}>
        <Sphere radius={0.5} ico={3} />
        <StandardMaterial base_color={c(0.7, 0.2, 0.1)} />
      </Mesh3d>
      <PointLight shadow_maps_enabled transform={Transform.fromXyz(4, 8, 4)} />
      <Camera3d transform={Transform.fromXyz(-2, 3, 5).lookingAt([0, 0, 0])} />
    </>
  );
}

registerRoot(() => <Composition id="main" component={Shot} durationInFrames={121} fps={60} width={1280} height={720} />);
