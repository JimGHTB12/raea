// Bevy v0.20.0 examples/shader/gpu_component_array_buffer.rs in raea's TSX (docs/control.feature FC0a, FC2a): unit
// cubes at a tenth of their size, each with the example's own Material, assets/shaders/gpu_component_array_buffer.wesl
// as written: its colour read from a storage buffer of every cube's CustomMaterialData ({ color, pad }, in the struct
// CustomMaterialDataArray's data_array) by the cube's MeshTag, times its texture (branding/icon.png or
// bevy_bird_dark.png). The example's update systems, run as Bevy's clock steps them (16 666 667 ns an update, the first
// update's delta 0): add_cube every 300 ms (on_timer) puts a cube at (x, 0.5, z) with x and z in [-1, 1), a colour in
// [0, 1)^3 and a coin flip for its texture, all from its ChaCha8Rng (seed 12345, lib/rand.ts); remove_cube every second
// despawns one of the cubes there before the update, chosen from them in the World's order (spawned in order, a
// despawn moving the last into its place, a cube spawned on that update joining after), after add_cube's draws on an
// update both run. The camera at (-2, 1.25, 2.5) looking at the origin, in Bevy's default 1280 x 720 window.
import { Composition, registerRoot, staticFile, useCurrentFrame } from "raea";
import { Transform } from "raea/transform";
import { Camera3d } from "raea/camera";
import { Cuboid, Mesh3d } from "raea/mesh";
import { BevyComponent } from "raea/3d";
import { Material } from "@raea/render/bevy3d"; // raea/pbr once scene-loop lands (its pbr.ts commit)
import { ChaCha8Rng } from "../../../../../bench/bevy-parity/lib/rand";

const SHADER = staticFile("shaders/gpu_component_array_buffer.wesl");
const LIGHT = staticFile("branding/icon.png");
const DARK = staticFile("branding/bevy_bird_dark.png");
const DT_NS = 16_666_667;

// `n`: the cube's spawn number, its own as an entity is (its name, which keeps a part one row across frames: a row's
// texture is its first frame's), its MeshTag and its place in the buffer
// (where Bevy's GpuComponentArray tags each by its own index)
type Cube = { n: number; x: number; z: number; color: [number, number, number]; light: boolean };

/** on_timer(period): the updates whose clock crosses a whole period (Timer in Repeating mode, ticked by each delta) */
const fires = (f: number, periodMs: number) => f > 0 && Math.floor((f * DT_NS) / (periodMs * 1e6)) > Math.floor(((f - 1) * DT_NS) / (periodMs * 1e6));

// the World's cubes after each update, worked out once in order (each frame is the fold of the updates up to it)
const worlds: Cube[][] = [];
const rng = ChaCha8Rng.seed_from_u64(12345);
let spawned = 0;
function cubesAt(f: number): Cube[] {
  while (worlds.length <= f) {
    const k = worlds.length;
    const cubes = (worlds[k - 1] ?? []).slice();
    let born: Cube | null = null;
    if (fires(k, 300)) {
      const x = rng.random_range(-1, 1);
      const z = rng.random_range(-1, 1);
      const color: [number, number, number] = [rng.random_range(0, 1), rng.random_range(0, 1), rng.random_range(0, 1)];
      born = { n: spawned++, x, z, color, light: rng.random_bool(0.5) };
    }
    if (fires(k, 1000) && cubes.length > 0) {
      // despawned once the update's commands apply, swapping the last of the cubes already there into its place: a
      // cube spawned this update is in another archetype until PostUpdate adds its render components, so it comes after
      const i = rng.random_index(cubes.length);
      const last = cubes.pop()!;
      if (i < cubes.length) cubes[i] = last;
    }
    if (born) cubes.push(born);
    worlds.push(cubes);
  }
  return worlds[f]!;
}

function Shot() {
  const cubes = cubesAt(useCurrentFrame());
  const data_array = Array.from({ length: Math.max(0, ...cubes.map((c) => c.n + 1)) }, () => ({ color: [0, 0, 0], pad: 0 }));
  for (const c of cubes) data_array[c.n] = { color: c.color, pad: 0 };
  return (
    <>
      {cubes.map((c) => (
        <Mesh3d key={c.n} name={`cube${c.n}`} transform={{ translation: [c.x, 0.5, c.z], rotation: [0, 0, 0, 1], scale: [0.1, 0.1, 0.1] }}>
          <Cuboid half_size={[0.5, 0.5, 0.5]} />
          <Material fragment={SHADER} buffers={{ material_data: { data_array } }} textures={{ color_texture: c.light ? LIGHT : DARK }} />
          <BevyComponent type="bevy_mesh::components::MeshTag" value={{ value: c.n }} />
        </Mesh3d>
      ))}
      <Camera3d transform={Transform.fromXyz(-2, 1.25, 2.5).lookingAt([0, 0, 0])} />
    </>
  );
}

registerRoot(() => <Composition id="main" component={Shot} durationInFrames={401} fps={60} width={1280} height={720} />);
