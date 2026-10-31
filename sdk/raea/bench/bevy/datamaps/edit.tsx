// Textures given inline as data: URLs (sdk/raea/bench/bevy/check.sh): an SVG label (data:image/svg+xml, utf8) on a box
// and on an ImagePlane, and a PNG checker (data:image/png;base64) on a second box. raea-render decodes them in memory; before
// 2026-10-18 a data: URL was read as a file path and the render refused.
import { AbsoluteFill, Composition, registerRoot, useCurrentFrame } from "raea";
import { ImagePlane } from "raea/3d";
import { StandardMaterial } from "raea/pbr";
import { Cuboid, Mesh3d } from "raea/mesh";
import { AmbientLight, DirectionalLight } from "raea/light";
import { Quat, Transform } from "raea/transform";
import { Camera3d } from "raea/camera";

const FPS = 30;
const FRAMES = 60;
const LABEL = 'data:image/svg+xml;utf8,<svg xmlns="http://www.w3.org/2000/svg" width="512" height="256"><rect width="512" height="256" fill="%23121724"/><circle cx="96" cy="128" r="64" fill="%23d9b877"/><rect x="190" y="96" width="280" height="64" rx="16" fill="%23efe9db"/></svg>';
const CHECKER = "data:image/png;base64,iVBORw0KGgoAAAANSUhEUgAAAAgAAAAICAYAAADED76LAAAAIUlEQVR4nGOQUAz5D8L/q6TAGJ3PQFABLgkYn7CCQeAGAMt1g8GFTe7BAAAAAElFTkSuQmCC";

function Shot() {
  const f = useCurrentFrame();
  return (
    <AbsoluteFill style={{ background: "#f4efe6" }}>
      <>
        <Camera3d transform={Transform.fromXyz(0, 0.6, 6).lookingAt([0, 0.2, 0])} projection={{ Perspective: { fov: 0.733038285837618 } }} />
        <AmbientLight brightness={240} />
        <DirectionalLight transform={Transform.fromXyz(2, 4, 5).lookingAt([0, 0, 0])} illuminance={8000} shadow_maps_enabled />
        <ImagePlane src={LABEL} width={2.2} position={[-1.5, 0.9, 0]} rotation={[0, 0.25, 0]} />
        <Mesh3d transform={Transform.fromXyz(1.4, 0.7, 0).withRotation(Quat.fromEuler("XYZ", 0.3, f * 0.04, 0))} name="label-box">
          <Cuboid half_size={[0.6, 0.3, 0.3]} />
          <StandardMaterial base_color="#cccccc" perceptual_roughness={0.6} base_color_texture={LABEL} />
        </Mesh3d>
        <Mesh3d transform={Transform.fromXyz(0, -0.9, 0).withRotation(Quat.fromEuler("XYZ", 0.4, 0.6 + f * 0.03, 0))} name="checker-box">
          <Cuboid half_size={[0.5, 0.5, 0.5]} />
          <StandardMaterial base_color="#cccccc" perceptual_roughness={0.5} base_color_texture={CHECKER} />
        </Mesh3d>
      </>
    </AbsoluteFill>
  );
}

registerRoot(() => <Composition id="datamaps" component={Shot} width={1280} height={720} fps={FPS} durationInFrames={FRAMES} />);
