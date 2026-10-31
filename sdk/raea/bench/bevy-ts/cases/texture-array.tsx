// gap: SH9
// Texture arrays (docs/shaders.feature SH9): a texture given as { layers: [picture, ...] } or { src, layers: N } is read
// as texture_2d_array<f32> tex_NAME, by layer, in each kind of shader.
//   top: two <ShaderMesh> boxes over one array of two 1 x 1 pictures (red, blue): the left reads layer 0, the right 1;
//   bottom: two surface shaders over checker2.png (2 x 2, white top left) as a stack of two 2 x 1 layers, read at the
//     left texel: layer 0 is white and shows red, layer 1 black and shows green (the file loads before the array is made);
//   a <FramePass> whose array's layer 1 (white) is a band over the frame's top rows.
// probe: 17 18 r>200 g<40 b<40
// probe: 47 18 r<40 g<40 b>200
// probe: 17 46 r>200 g<40 b<40
// probe: 47 46 r<40 g>200 b<40
// probe: 32 1 r>200 g>200 b>200
// probe: 32 32 r<40 g<40 b<40
import { AbsoluteFill, Composition, registerRoot } from "raea";
import { Canvas3D, FramePass, ShaderMesh, ToneMapping } from "raea/3d";
import { Cuboid, Mesh3d } from "raea/mesh";
import { Transform } from "raea/transform";
import { Camera3d } from "raea/camera";
import { ExtendedMaterial, ShaderExtension, StandardMaterial } from "raea/pbr";

const one = (c: string) => ({ width: 1, height: 1, pixels: [c] });
const frames = { layers: [one("#ff0000"), one("#0000ff")] };

const pick = `
fn shade(in: VertexOutput) -> vec4<f32> {
  return textureSampleLevel(tex_frames, samp_frames, vec2<f32>(0.5, 0.5), i32(u_layer()), 0.0);
}`;

const strip = (layer: number) => `
fn lit(in: VertexOutput, pbr: PbrInput, color: vec4<f32>) -> vec4<f32> {
  let s = textureSampleLevel(tex_strip, samp_strip, vec2<f32>(0.25, 0.5), ${layer}, 0.0);
  return vec4<f32>(s.r, 1.0 - s.r, 0.0, 1.0);
}`;
const checker = { src: "checker2.png", layers: 2, sampler: { filter: "nearest" } };

const band = `
fn shade(in: FullscreenVertexOutput) -> vec4<f32> {
  if in.uv.y < 0.06 { return textureSampleLevel(tex_ramp, samp_ramp, vec2<f32>(0.5, 0.5), 1, 0.0); }
  return frame(in.uv);
}`;

function Shot() {
  return (
    <AbsoluteFill style={{ background: "#000" }}>
      <Canvas3D>
        <Camera3d transform={Transform.fromXyz(0, 0, 6).lookingAt([0, 0, 0])} projection={{ Perspective: { fov: 0.872664625997165 } }} />
        <ToneMapping mode="none" />
        <ShaderMesh shape="box(1, 1, 1)" position={[-1.4, 1.2, 0]} wesl={pick} uniforms={{ layer: 0 }} textures={{ frames }} />
        <ShaderMesh shape="box(1, 1, 1)" position={[1.4, 1.2, 0]} wesl={pick} uniforms={{ layer: 1 }} textures={{ frames }} />
        <Mesh3d transform={Transform.fromXyz(-1.4, -1.2, 0)}><Cuboid half_size={[0.5, 0.5, 0.5]} /><ExtendedMaterial base={<StandardMaterial base_color="#ffffff" />} extension={<ShaderExtension wesl={strip(0)} textures={{ strip: checker }} />} /></Mesh3d>
        <Mesh3d transform={Transform.fromXyz(1.4, -1.2, 0)}><Cuboid half_size={[0.5, 0.5, 0.5]} /><ExtendedMaterial base={<StandardMaterial base_color="#ffffff" />} extension={<ShaderExtension wesl={strip(1)} textures={{ strip: checker }} />} /></Mesh3d>
        <FramePass wesl={band} textures={{ ramp: { layers: [one("#000000"), one("#ffffff")] } }} />
      </Canvas3D>
    </AbsoluteFill>
  );
}
registerRoot(() => <Composition id="main" component={Shot} durationInFrames={2} fps={30} width={64} height={64} />);
