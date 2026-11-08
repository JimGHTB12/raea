// gap: SH3
// A shader over a Character's skin (<Character> with a <MeshStandardMaterial shader> child, passed to its body Model):
// card-body is a one-joint test body, a 1 m x 2 m white card skinned to pelvis (made in code; its joint renamed from
// CMU's Hips for the game_engine names, raea #38's corpus). The `lit` hook keeps only the lit green, so the body
// shows green.
// probe: 32 32 g>100 r<40 b<40
// probe: 4 32 g<20
import { AbsoluteFill, Composition, registerRoot } from "raea";
import { ToneMapping } from "raea/3d";
import { Character, type Skeleton } from "raea/character";
import { AmbientLight, DirectionalLight } from "raea/light";
import { Transform } from "raea/transform";
import { Camera3d } from "raea/camera";
import { ExtendedMaterial, ShaderExtension, StandardMaterial } from "raea/pbr";
// card-body/skeleton.json, inline
const skeleton: Skeleton = { name: "card", height: 2, joints: [{ name: "pelvis", parent: -1, head: [0, 1, 0], tail: [0, 2, 0] }], landmarks: { eyes: [0, 1.7, 0] } };

const wesl = `
fn lit(in: VertexOutput, pbr: PbrInput, color: vec4<f32>) -> vec4<f32> {
  return vec4<f32>(0.0, color.g, 0.0, 1.0);
}`;

function Shot() {
  return (
    <AbsoluteFill style={{ background: "#000" }}>
      <>
        <Camera3d transform={Transform.fromXyz(0, 1, 4).lookingAt([0, 1, 0])} projection={{ Perspective: { fov: 0.698131700797732 } }} />
        <ToneMapping mode="none" />
        <AmbientLight brightness={120} />
        <DirectionalLight transform={Transform.fromXyz(0, 1, 5).lookingAt([0, 0, 0])} illuminance={8000} />
        <Character skeleton={skeleton} src="card-body" outline={false}>
          <ExtendedMaterial base={<StandardMaterial base_color="#cccccc" />} extension={<ShaderExtension {...({ wesl })} />} />
        </Character>
      </>
    </AbsoluteFill>
  );
}
registerRoot(() => <Composition id="main" component={Shot} durationInFrames={2} fps={30} width={64} height={64} />);
