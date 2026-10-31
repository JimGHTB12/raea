// Environment fixture: Poly Haven CC0 panoramas (media/, 1k .hdr) as image-based light and sky. A mirror sphere, a
// matte one and a gold torus on a floor; the panorama turns and brightens over the shot, and at frame 30 it cuts from
// an exterior (venice_sunset) to an interior (museum_of_ethnography), so the light and the sky change on that frame. A
// small AmbientLight composes with it.
import { AbsoluteFill, Composition, interpolate, registerRoot, staticFile, useCurrentFrame } from "raea";
import { Environment, Sphere, Torus } from "raea/3d";
import { StandardMaterial } from "raea/pbr";
import { Cuboid, Mesh3d } from "raea/mesh";
import { AmbientLight } from "raea/light";
import { Quat, Transform } from "raea/transform";
import { Camera3d } from "raea/camera";

const FPS = 30;
const FRAMES = 60;

function Shot() {
  const f = useCurrentFrame();
  const outside = f < 30;
  const turn = interpolate(f, [0, FRAMES - 1], [0, 1.2]);
  return (
    <AbsoluteFill style={{ background: "#000000" }}>
      <>
        <Camera3d transform={Transform.fromXyz(0, 1.2, 5).lookingAt([0, 0.6, 0])} projection={{ Perspective: { fov: 0.872664625997165 } }} />
        <AmbientLight brightness={20} />
        <Environment
          src={staticFile(outside ? "venice_sunset_1k.hdr" : "museum_of_ethnography_1k.hdr")}
          intensity={interpolate(f, [0, FRAMES - 1], [0.8, 1.2])}
          rotation={turn}
          background
        />
        <Mesh3d transform={Transform.fromXyz(0, -0.05, 0)}>
          <Cuboid half_size={[4, 0.05, 4]} />
          <StandardMaterial base_color="#8a8580" perceptual_roughness={0.6} />
        </Mesh3d>
        <Mesh3d transform={Transform.fromXyz(-1.4, 0.6, 0)}>
          <Sphere radius={0.6} />
          <StandardMaterial base_color="#ffffff" perceptual_roughness={0.04} metallic={1} />
        </Mesh3d>
        <Mesh3d transform={Transform.fromXyz(0, 0.6, -0.4)}>
          <Sphere radius={0.6} />
          <StandardMaterial base_color="#e8e4dc" perceptual_roughness={0.9} />
        </Mesh3d>
        <Mesh3d transform={Transform.fromXyz(1.4, 0.6, 0).withRotation(Quat.fromEuler("XYZ", 0.4, f * 0.03, 0))}>
          <Torus major_radius={0.45} minor_radius={0.16} />
          <StandardMaterial base_color="#e0b050" perceptual_roughness={0.25} metallic={1} />
        </Mesh3d>
      </>
    </AbsoluteFill>
  );
}

registerRoot(() => <Composition id="environment" component={Shot} width={960} height={540} fps={FPS} durationInFrames={FRAMES} />);
