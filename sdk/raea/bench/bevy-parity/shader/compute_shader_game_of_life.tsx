// Bevy v0.20.0 examples/shader/compute_shader_game_of_life.rs in raea's TSX (docs/control.feature FC0a, FC2d): its
// assets/shaders/game_of_life.wesl as written, a <Compute file> over two 320 x 180 rgba32float images, `a` and `b`,
// dispatched 40 x 22 workgroups of 8 x 8 before the camera draws, its uniform `config` { alive_color: RED }. The
// example's render-graph node runs `init` once its pipeline is ready, binding (a, b), then `update` every frame,
// alternating (b, a) and (a, b): in its parity run the init is frame 2's, so frame f's update writes generation f - 2,
// into b when that is even and a when odd. Its switch_textures flips the Sprite between the two every update, starting
// from a, so the sprite shows b on even frames and a on odd: the image just written. The Sprite is 320 x 180 at scale 4,
// drawn nearest (ImagePlugin::default_nearest), over a black clear colour, in the 1280 x 720 window.
import { Composition, registerRoot, staticFile, useCurrentFrame } from "raea";
import { Canvas3D, Compute, Sprite } from "raea/3d";

const INIT_FRAME = 2;
// ImagePlugin::default_nearest: the two images are drawn nearest
const NEAREST = { filter: "nearest" } as const;

function Shot() {
  const f = useCurrentFrame();
  const entry = f < INIT_FRAME ? null : f === INIT_FRAME ? "init" : "update";
  // generation f - 2 goes to b when even: input a, output b; when odd the other way
  const bind = (f - INIT_FRAME) % 2 === 0 ? { input: "a", output: "b" } : { input: "b", output: "a" };
  return (
    <Canvas3D layered clearColor="#000000">
      <Compute
        file={staticFile("shaders/game_of_life.wesl")}
        workgroups={[40, 22, 1]}
        images={{ a: { src: "render:life_a:320x180", sampler: NEAREST }, b: { src: "render:life_b:320x180", sampler: NEAREST } }}
        uniforms={{ config: { alive_color: [1, 0, 0, 1] } }}
        entry={entry}
        bind={entry === null ? {} : bind}
      />
      <Sprite src={f % 2 === 0 ? "render:life_b:320x180" : "render:life_a:320x180"} x={640} y={360} width={320} height={180} scale={4} />
    </Canvas3D>
  );
}

registerRoot(() => <Composition id="main" component={Shot} durationInFrames={61} fps={60} width={1280} height={720} />);
