// Bevy v0.20.0 examples/shader/array_texture.rs in raea's TSX (docs/control.feature FC0a, FC2a): eleven 1 m cubes at
// (x + 0.5, 0, 0) for x in -5..=5 with the example's own Material, assets/shaders/array_texture.wesl as written: it
// lights a StandardMaterial-like surface whose base colour is a layer of a texture_2d_array (textures/array_texture.png,
// four rows stacked, ImageArrayLayout::RowCount) picked by the cube's MeshTag, x as u32 % 4. The example's
// update_mesh_tags steps every tag by one each time its 1.5 s repeating Timer finishes. A default DirectionalLight from
// (3, 2, 1), the camera at (5, 5, 5) looking at (1.5, 0, 0), in Bevy's default 1280 x 720 window.
import { Composition, registerRoot, staticFile, useCurrentFrame } from "raea";
import { Transform } from "raea/transform";
import { Camera3d } from "raea/camera";
import { Cuboid, Mesh3d } from "raea/mesh";
import { DirectionalLight } from "raea/light";
import { BevyComponent } from "raea/3d";
import { Material } from "@raea/render/bevy3d"; // raea/pbr once scene-loop lands (its pbr.ts commit)

const ARRAY = { src: staticFile("textures/array_texture.png"), layers: 4 };

function Shot() {
  // the Timer finishes once per 1.5 s of elapsed time (1/60 s a frame, none on the first update)
  const steps = Math.floor(useCurrentFrame() / 90);
  return (
    <>
      <DirectionalLight transform={Transform.fromXyz(3, 2, 1).lookingAt([0, 0, 0])} />
      <Camera3d transform={Transform.fromXyz(5, 5, 5).lookingAt([1.5, 0, 0])} />
      {Array.from({ length: 11 }, (_, i) => i - 5).map((x) => (
        <Mesh3d key={x} transform={Transform.fromXyz(x + 0.5, 0, 0)}>
          <Cuboid half_size={[0.5, 0.5, 0.5]} />
          <Material fragment={staticFile("shaders/array_texture.wesl")} textures={{ my_array_texture: ARRAY }} />
          {/* x as u32 % 4 wraps a negative x to 2^32 + x, whose remainder is x's non-negative one */}
          <BevyComponent type="bevy_mesh::components::MeshTag" value={{ value: ((((x % 4) + 4) % 4) + steps) % 4 }} />
        </Mesh3d>
      ))}
    </>
  );
}

registerRoot(() => <Composition id="main" component={Shot} durationInFrames={101} fps={60} width={1280} height={720} />);
