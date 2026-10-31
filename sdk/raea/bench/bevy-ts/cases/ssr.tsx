// gap: G14
// Screen-space reflections from Bevy's own components and resource through the generic layer: on the camera Msaa Off,
// DepthPrepass, NormalPrepass, DeferredPrepass and ScreenSpaceReflections; DefaultOpaqueRendererMethod Deferred (SSR
// reflects deferred surfaces only); a metal floor at perceptual roughness 0.2 (SSR fades in from 0.08 and is full from
// 0.12: the first try at 0.02 showed nothing). The red cube shows in the floor below it; ssr-control.tsx, without
// them, keeps that floor grey.
// probe: 32 44 r>150
import { AbsoluteFill, Composition, registerRoot } from "raea";
import { BevyComponent, BevyResource, Canvas3D } from "raea/3d";
import { LinearRgba, Srgba } from "raea/color";
import { StandardMaterial } from "raea/pbr";
import { Cuboid, Mesh3d } from "raea/mesh";
import { Transform } from "raea/transform";
import { Camera3d } from "raea/camera";

function Shot() {
  return (
    <AbsoluteFill style={{ background: "#000" }}>
      <Canvas3D layered>
        <Camera3d transform={Transform.fromXyz(0, 1.2, 4).lookingAt([0, 0.3, 0])} projection={{ Perspective: { fov: 0.698131700797732 } }}>
          <BevyComponent type="bevy_render::view::Msaa" value="Off" />
          <BevyComponent type="bevy_core_pipeline::prepass::DepthPrepass" value={{}} />
          <BevyComponent type="bevy_core_pipeline::prepass::NormalPrepass" value={{}} />
          <BevyComponent type="bevy_core_pipeline::prepass::DeferredPrepass" value={{}} />
          <BevyComponent type="bevy_pbr::ssr::ScreenSpaceReflections" value={{}} />
        </Camera3d>
        <BevyResource type="bevy_pbr::material::DefaultOpaqueRendererMethod" value="Deferred" />
        <Mesh3d transform={Transform.fromXyz(0, 0.6, 0)} name="cube"><Cuboid half_size={[0.5, 0.5, 0.5]} /><StandardMaterial base_color="#ff0000" emissive={LinearRgba.from(Srgba.hex("#ff0000")).mul(4)} /></Mesh3d>
        <Mesh3d name="floor"><Cuboid half_size={[4, 0.05, 4]} /><StandardMaterial base_color="#ffffff" perceptual_roughness={0.2} metallic={1} /></Mesh3d>
      </Canvas3D>
    </AbsoluteFill>
  );
}
registerRoot(() => <Composition id="main" component={Shot} durationInFrames={2} fps={30} width={64} height={64} />);
