// gap: SH8
// Shader defs (docs/shaders.feature SH8): one source, branched by its defs, compiles a pipeline per set.
//   top: two <ShaderMesh> boxes with one source, `@if(RED)` picking its colour: RED true is red, false blue;
//   bottom: two boxes with one surface shader whose `lit` hook reads constants::G: 255 is green, 0 red;
//   a <FramePass> whose defs turn on `@if(BAND)`, a white band over the frame's top rows.
// probe: 17 18 r>200 g<40 b<40
// probe: 47 18 r<40 g<40 b>200
// probe: 17 46 r<40 g>200 b<40
// probe: 47 46 r>200 g<40 b<40
// probe: 32 1 r>200 g>200 b>200
// probe: 32 32 r<40 g<40 b<40
import { AbsoluteFill, Composition, registerRoot } from "raea";
import { FramePass, ShaderMesh, ToneMapping } from "raea/3d";
import { Cuboid, Mesh3d } from "raea/mesh";
import { Transform } from "raea/transform";
import { Camera3d } from "raea/camera";
import { ExtendedMaterial, ShaderExtension, StandardMaterial } from "raea/pbr";

const pick = `
@if(RED) fn colour() -> vec4<f32> { return vec4<f32>(1.0, 0.0, 0.0, 1.0); }
@if(!RED) fn colour() -> vec4<f32> { return vec4<f32>(0.0, 0.0, 1.0, 1.0); }
fn shade(in: VertexOutput) -> vec4<f32> { return colour(); }`;

const level = `
fn lit(in: VertexOutput, pbr: PbrInput, color: vec4<f32>) -> vec4<f32> {
  let g = f32(constants::G) / 255.0;
  return vec4<f32>(1.0 - g, g, 0.0, 1.0);
}`;

const band = `
@if(BAND) fn banded(uv: vec2<f32>) -> bool { return uv.y < 0.06; }
@if(!BAND) fn banded(uv: vec2<f32>) -> bool { return false; }
fn shade(in: FullscreenVertexOutput) -> vec4<f32> {
  if banded(in.uv) { return vec4<f32>(1.0); }
  return frame(in.uv);
}`;

function Shot() {
  return (
    <AbsoluteFill style={{ background: "#000" }}>
      <>
        <Camera3d transform={Transform.fromXyz(0, 0, 6).lookingAt([0, 0, 0])} projection={{ Perspective: { fov: 0.872664625997165 } }} />
        <ToneMapping mode="none" />
        <ShaderMesh shape="box(1, 1, 1)" position={[-1.4, 1.2, 0]} wesl={pick} defs={{ RED: true }} />
        <ShaderMesh shape="box(1, 1, 1)" position={[1.4, 1.2, 0]} wesl={pick} defs={{ RED: false }} />
        <Mesh3d transform={Transform.fromXyz(-1.4, -1.2, 0)}><Cuboid half_size={[0.5, 0.5, 0.5]} /><ExtendedMaterial base={<StandardMaterial base_color="#ffffff" />} extension={<ShaderExtension wesl={level} defs={{ G: 255 }} />} /></Mesh3d>
        <Mesh3d transform={Transform.fromXyz(1.4, -1.2, 0)}><Cuboid half_size={[0.5, 0.5, 0.5]} /><ExtendedMaterial base={<StandardMaterial base_color="#ffffff" />} extension={<ShaderExtension wesl={level} defs={{ G: 0 }} />} /></Mesh3d>
        <FramePass wesl={band} defs={{ BAND: true }} />
      </>
    </AbsoluteFill>
  );
}
registerRoot(() => <Composition id="main" component={Shot} durationInFrames={2} fps={30} width={64} height={64} />);
