// Bevy v0.20.0 examples/shader/extended_material_bindless.rs in raea's TSX (docs/control.feature FC0a, FC2b): a UV
// sphere (radius 1, 20 sectors and stacks) at (0, 0.5, 0), its StandardMaterial GRAY_600 extended by the example's own
// shader, assets/shaders/extended_material_bindless.wesl as written: its uniform's modulate_color (RED) times
// uv_checker_bw.png, under the standard lighting. raea's extension isn't bindless, so the shader's @if(BINDLESS) set
// compiles out and its @else bindings (50, 51, 52) bind, as Bevy's do on a GPU without bindless. rotate_sphere turns it
// to Quat::from_euler(YXZ, -elapsed_secs, 3 pi / 2, 0) each update; a DirectionalLight at (1, 1, 1) looking at the
// origin, the camera at (-2, 2.5, 5).
import { Composition, registerRoot, staticFile, useCurrentFrame } from "raea";
import { Quat, Transform } from "raea/transform";
import { Camera3d } from "raea/camera";
import { Mesh3d, Sphere } from "raea/mesh";
import { ExtendedMaterial, ShaderExtension, StandardMaterial } from "raea/pbr";
import { DirectionalLight } from "raea/light";

// Time::elapsed_secs at frame f: f steps of 1/60 s, summed in f32
function elapsedAt(f: number): number {
  let t = 0;
  for (let k = 1; k <= f; k++) t = Math.fround(t + Math.fround(16_666_667e-9));
  return t;
}

function Shot() {
  const t = elapsedAt(useCurrentFrame());
  return (
    <>
      <Mesh3d transform={{ translation: [0, 0.5, 0], rotation: Quat.fromEuler("YXZ", -t, Math.PI * 1.5, 0), scale: [1, 1, 1] }}>
        <Sphere radius={1} uv={[20, 20]} />
        <ExtendedMaterial
          base={<StandardMaterial base_color="#4b5563" />}
          extension={<ShaderExtension fragment={staticFile("shaders/extended_material_bindless.wesl")} uniforms={{ modulate_color: [1, 0, 0, 1] }} textures={{ modulate_texture: staticFile("textures/uv_checker_bw.png") }} />}
        />
      </Mesh3d>
      <DirectionalLight transform={Transform.fromXyz(1, 1, 1).lookingAt([0, 0, 0])} />
      <Camera3d transform={Transform.fromXyz(-2, 2.5, 5).lookingAt([0, 0, 0])} />
    </>
  );
}

registerRoot(() => <Composition id="main" component={Shot} durationInFrames={61} fps={60} width={1280} height={720} />);
