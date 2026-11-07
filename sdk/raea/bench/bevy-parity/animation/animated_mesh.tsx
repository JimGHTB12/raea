// Bevy v0.20.0 examples/animation/animated_mesh.rs in raea's TSX (docs/control.feature FC0a): the fox
// (models/animated/Fox.glb) playing its Run clip, Animation(2), on a 500 km green plane under a shadow-casting sun,
// with a GlobalAmbientLight of 2000, in Bevy's default 1280 x 720 window.
import { Composition, registerRoot, staticFile, useCurrentFrame } from "raea";
import { Model } from "raea/3d";
import { Quat, Transform } from "raea/transform";
import { Camera3d } from "raea/camera";
import { Mesh3d, Plane3d } from "raea/mesh";
import { StandardMaterial } from "raea/pbr";
import { DirectionalLight, GlobalAmbientLight, Shadows } from "raea/light";

function Shot() {
  // the clip advances by each update's delta_secs from the first (as many_foxes)
  const t = useCurrentFrame() / 60;
  return (
    <>
      <GlobalAmbientLight color="#ffffff" brightness={2000} />
      <Camera3d transform={Transform.fromXyz(100, 100, 150).lookingAt([0, 20, 0])} />
      <Model src={staticFile("models/animated/Fox.glb")} animation={2} time={t} />
      <Mesh3d transform={Transform.IDENTITY}>
        <Plane3d half_size={[250000, 250000]} />
        <StandardMaterial base_color="rgb(76.5, 127.5, 76.5)" />
      </Mesh3d>
      {/* Bevy's default 2048-texel maps; its CascadeShadowConfigBuilder's 4 cascades, 0.2 overlap */}
      <Shadows mapSize={2048} firstCascade={200} distance={400} cascades={4} overlap={0.2} />
      <DirectionalLight shadow_maps_enabled transform={{ translation: [0, 0, 0], rotation: Quat.fromEuler("ZYX", 0, 1, -Math.PI / 4), scale: [1, 1, 1] }} />
    </>
  );
}

registerRoot(() => <Composition id="main" component={Shot} durationInFrames={61} fps={60} width={1280} height={720} />);
