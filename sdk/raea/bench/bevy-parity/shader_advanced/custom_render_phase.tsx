// Bevy v0.20.0 examples/shader_advanced/custom_render_phase.rs in raea's TSX (docs/control.feature FC2d): a phase of
// its own after the main opaque pass, drawing each mesh marked DrawStencil again by assets/shaders/custom_stencil.wesl
// as written (red, at the mesh's positions) onto the view's colour target with no depth attachment, back faces
// culled: a <MeshPass> inside the mesh. A white Circle of radius 4 lying flat, a 1 m cube of srgb_u8(124, 144, 255) at
// (0, 0.5, 0) and a sphere of radius 0.5 (Bevy's default ico mesher, 5 subdivisions) of srgb_u8(124, 255, 144) at
// (2, 0.5, 0), its mesh compressed (MeshCompressionArgs::regular(), so the shader reads compressed positions), both
// marked; a PointLight with shadows at (4, 8, 4), and the camera at (-2, 4.5, 9) looking at the origin with Msaa off,
// in Bevy's default 1280 x 720 window.
import { Composition, registerRoot, staticFile } from "raea";
import { BevyComponent, Entity, MeshPass } from "raea/3d";
import { Camera3d } from "raea/camera";
import { PointLight } from "raea/light";
import { Cuboid, Mesh3d, Sphere } from "raea/mesh";
import { StandardMaterial } from "raea/pbr";
import { Quat, Transform, eulerOf } from "raea/transform";

const STENCIL = staticFile("shaders/custom_stencil.wesl");

function Shot() {
  return (
    <>
      {/* a Circle in 3D: raea's typed Circle only goes in a Mesh2d (conventions C15), so through reflection */}
      <Entity rotation={eulerOf(Quat.fromRotationX(-Math.PI / 2))}>
        <BevyComponent type="bevy_mesh::components::Mesh3d" value={{ primitive: "Circle", radius: 4 }} />
        <BevyComponent type="bevy_pbr::mesh_material::MeshMaterial3d<bevy_pbr::pbr_material::StandardMaterial>" value={{ base_color: { Srgba: { red: 1, green: 1, blue: 1, alpha: 1 } } }} />
      </Entity>
      <Mesh3d transform={Transform.fromXyz(0, 0.5, 0)}>
        <Cuboid half_size={[0.5, 0.5, 0.5]} />
        <StandardMaterial base_color="rgb(124, 144, 255)" />
        <MeshPass shader={STENCIL} />
      </Mesh3d>
      <Mesh3d transform={Transform.fromXyz(2, 0.5, 0)} compressed>
        <Sphere radius={0.5} ico={5} />
        <StandardMaterial base_color="rgb(124, 255, 144)" />
        <MeshPass shader={STENCIL} />
      </Mesh3d>
      <PointLight shadow_maps_enabled transform={Transform.fromXyz(4, 8, 4)} />
      <Camera3d transform={Transform.fromXyz(-2, 4.5, 9).lookingAt([0, 0, 0])}>
        <BevyComponent type="bevy_render::view::Msaa" value="Off" />
      </Camera3d>
    </>
  );
}

registerRoot(() => <Composition id="main" component={Shot} durationInFrames={11} fps={60} width={1280} height={720} />);
