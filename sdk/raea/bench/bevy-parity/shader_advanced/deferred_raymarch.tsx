// Bevy v0.20.0 examples/shader_advanced/deferred_raymarch.rs in raea's TSX (docs/control.feature FC2d): a gyroid
// raymarched in a fullscreen pass that writes Bevy's deferred G-buffer and depth, so deferred lighting lights it and
// the scene's meshes meet it in depth, and its `fragment_shadow` writes the directional light's shadow maps, so it
// casts shadows: a <GBufferPass> of assets/shaders/deferred_raymarch.wesl as written on the camera, which has Msaa off,
// DepthPrepass and DeferredPrepass, and every opaque material deferred (DefaultOpaqueRendererMethod::deferred()). The
// camera at (4, 3, 6) looking at (0, 0.2, 0) with an AmbientLight of 200; a 20 m plane of srgb(0.3, 0.5, 0.3) at
// y -1.5, a 1.2 m cube of srgb(0.2, 0.4, 0.9) and roughness 0.4 at (2.2, -0.9, 0.5), and a shadowed DirectionalLight of
// 8000 lux from (4, 8, 4) to the origin, in Bevy's default 1280 x 720 window. The gyroid moves with Globals' time.
// The cascades and the shadow map's size go in a <Shadows> at Bevy's defaults (2048 texels, four cascades to 150 m, the
// first to 10 m, overlapping by 0.2), as raea's sun keeps a film's. (Its FreeCamera only moves the camera under input,
// which a render has none of.)
import { Composition, registerRoot, staticFile } from "raea";
import { BevyComponent, BevyResource, GBufferPass, Shadows } from "raea/3d";
import { Camera3d } from "raea/camera";
import { Srgba } from "raea/color";
import { AmbientLight, DirectionalLight } from "raea/light";
import { Cuboid, Mesh3d, Plane3d } from "raea/mesh";
import { StandardMaterial } from "raea/pbr";
import { Transform } from "raea/transform";

function Shot() {
  return (
    <>
      <BevyResource type="bevy_pbr::material::DefaultOpaqueRendererMethod" value="Deferred" />
      <Camera3d transform={Transform.fromXyz(4, 3, 6).lookingAt([0, 0.2, 0])}>
        <BevyComponent type="bevy_render::view::Msaa" value="Off" />
        <BevyComponent type="bevy_core_pipeline::prepass::DepthPrepass" value={{}} />
        <BevyComponent type="bevy_core_pipeline::prepass::DeferredPrepass" value={{}} />
        <GBufferPass shader={staticFile("shaders/deferred_raymarch.wesl")} shadow="fragment_shadow" />
      </Camera3d>
      <Shadows mapSize={2048} distance={150} firstCascade={10} cascades={4} overlap={0.2} />
      <AmbientLight brightness={200} />
      <Mesh3d transform={Transform.fromXyz(0, -1.5, 0)}>
        <Plane3d half_size={[10, 10]} />
        <StandardMaterial base_color={new Srgba(0.3, 0.5, 0.3)} />
      </Mesh3d>
      <Mesh3d transform={Transform.fromXyz(2.2, -0.9, 0.5)}>
        <Cuboid half_size={[0.6, 0.6, 0.6]} />
        <StandardMaterial base_color={new Srgba(0.2, 0.4, 0.9)} perceptual_roughness={0.4} />
      </Mesh3d>
      <DirectionalLight illuminance={8000} shadow_maps_enabled transform={Transform.fromXyz(4, 8, 4).lookingAt([0, 0, 0])} />
    </>
  );
}

registerRoot(() => <Composition id="main" component={Shot} durationInFrames={31} fps={60} width={1280} height={720} />);
