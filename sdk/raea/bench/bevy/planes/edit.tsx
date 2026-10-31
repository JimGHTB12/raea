// Pictures and shapes in space: a poster on an ImagePlane swinging in, a picture-mapped box (map), a Cone, and a
// Text3D headline, under a point light.
import { AbsoluteFill, Composition, interpolate, registerRoot, spring, staticFile, useCurrentFrame } from "raea";
import { Cone, ImagePlane, Text3D } from "raea/3d";
import { StandardMaterial } from "raea/pbr";
import { Cuboid, Mesh3d } from "raea/mesh";
import { AmbientLight, DirectionalLight, PointLight } from "raea/light";
import { Quat, Transform } from "raea/transform";
import { Camera3d } from "raea/camera";

const FPS = 30;
const FRAMES = 60;

function Shot() {
  const f = useCurrentFrame();
  const s = spring({ frame: f, fps: FPS, config: { damping: 14 } });
  const swing = interpolate(s, [0, 1], [-1.2, -0.25]);
  return (
    <AbsoluteFill style={{ background: "#f4efe6" }}>
      <>
        <Camera3d transform={Transform.fromXyz(0, 0.8, 6).lookingAt([0, 0.3, 0])} projection={{ Perspective: { fov: 0.733038285837618 } }} />
        <AmbientLight brightness={240} />
        <DirectionalLight transform={Transform.fromXyz(2, 4, 5).lookingAt([0, 0, 0])} illuminance={8000} shadow_maps_enabled />
        <PointLight color="#ffe0b0" transform={Transform.fromXyz(2, 1.5, 2)} intensity={720000} />
        <ImagePlane src={staticFile("poster.png")} width={2.2} position={[-1.4, 0.6, 0]} rotation={[0, swing * -1, 0]} />
        <Mesh3d transform={Transform.fromXyz(1.4, 0.6, 0).withRotation(Quat.fromEuler("XYZ", 0.3, f * 0.04, 0))}>
          <Cuboid half_size={[0.5, 0.5, 0.5]} />
          <StandardMaterial base_color="#cccccc" perceptual_roughness={0.6} base_color_texture={staticFile("poster.png")} />
        </Mesh3d>
        <Mesh3d transform={Transform.fromXyz(2.6, -0.4, -1)}>
          <Cone radius={0.4} height={1} />
          <StandardMaterial base_color="#2a9d8f" perceptual_roughness={0.5} />
        </Mesh3d>
        <Mesh3d transform={Transform.fromXyz(-1.5, -1.0, 0.5).withRotation(Quat.fromEuler("XYZ", 0, 0.1, 0))}>
          <Text3D size={0.45} depth={0.15}>RAEA</Text3D>
          <StandardMaterial base_color="#e76f51" perceptual_roughness={0.4} />
        </Mesh3d>
      </>
    </AbsoluteFill>
  );
}

registerRoot(() => <Composition id="planes" component={Shot} width={960} height={540} fps={FPS} durationInFrames={FRAMES} />);
