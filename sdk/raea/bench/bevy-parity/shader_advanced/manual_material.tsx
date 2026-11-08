// Bevy v0.20.0 examples/shader_advanced/manual_material.rs in raea's TSX (docs/control.feature FC2d): an ImageMaterial
// prepared by hand (ErasedRenderAsset) rather than by the Material trait, drawn in Opaque3d with
// assets/shaders/manual_material.wesl as written, its two bindings the Bevy logo and a sampler from
// SamplerDescriptor::default() (wgpu's: nearest filtering, nearest mipmaps and clamp to edge, over the image's own
// sampler). A 2 m cube at (0, 0.5, 0), a shadow-casting PointLight at (4, 8, 4) at Bevy's default intensity (the
// shader is unlit, so it doesn't show), and the camera at (-2.5, 4.5, 9) looking at the origin, in Bevy's default
// 1280 x 720 window.
import { Composition, registerRoot, staticFile } from "raea";
import { Transform } from "raea/transform";
import { Camera3d } from "raea/camera";
import { PointLight } from "raea/light";
import { Cuboid, Mesh3d } from "raea/mesh";
import { Material } from "@raea/render/bevy3d";

const DEFAULT_SAMPLER = { filter: "nearest", mipmap: "nearest", wrap: "clamp" } as const;

function Shot() {
  return (
    <>
      <Mesh3d transform={Transform.fromXyz(0, 0.5, 0)}>
        <Cuboid half_size={[1, 1, 1]} />
        <Material
          fragment={staticFile("shaders/manual_material.wesl")}
          textures={{ material_color_texture: { src: staticFile("branding/icon.png"), sampler: DEFAULT_SAMPLER } }}
        />
      </Mesh3d>
      <PointLight shadow_maps_enabled transform={Transform.fromXyz(4, 8, 4)} />
      <Camera3d transform={Transform.fromXyz(-2.5, 4.5, 9).lookingAt([0, 0, 0])} />
    </>
  );
}

registerRoot(() => <Composition id="main" component={Shot} durationInFrames={31} fps={60} width={1280} height={720} />);
