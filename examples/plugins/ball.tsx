// PL1's proof (docs/plugins.feature): a project outside the plugins bench imports the catalog by name, the same
// way any author's project does. A ball dropped on a floor with avian's components from raea/plugins/avian and Bevy's
// own Bloom from raea/plugins/bevy, typed by bench/tsc. The plugin build (tools/bevy-plugins.sh) renders it.
import { AbsoluteFill, Composition, registerRoot } from "raea";
import { Sphere } from "raea/3d";
import { ColliderConstructor, Restitution, RigidBody } from "raea/plugins/avian";
import { Bloom } from "raea/plugins/bevy";
import { avian, bevy } from "@raea/plugins";
import { StandardMaterial } from "raea/pbr";
import { Cuboid, Mesh3d } from "raea/mesh";
import { AmbientLight, DirectionalLight } from "raea/light";
import { Transform } from "raea/transform";
import { Camera3d } from "raea/camera";

function Drop() {
  return (
    <AbsoluteFill style={{ background: "#0f1720" }}>
      <>
        <Camera3d transform={Transform.fromXyz(0, 2, 7).lookingAt([0, 1, 0])} projection={{ Perspective: { fov: 0.785398163397448 } }} />
        <Bloom value={{ intensity: 0.2 }} />
        <AmbientLight brightness={200} />
        <DirectionalLight transform={Transform.fromXyz(3, 6, 4).lookingAt([0, 0, 0])} illuminance={10400} shadow_maps_enabled />
        <Mesh3d name="floor">
          <Cuboid half_size={[3, 0.15, 2]} />
          <StandardMaterial base_color="#334155" perceptual_roughness={0.8} />
          <RigidBody value="Static" />
          <ColliderConstructor value={{ Cuboid: { x_length: 6, y_length: 0.3, z_length: 4 } }} />
        </Mesh3d>
        <Mesh3d transform={Transform.fromXyz(0, 3, 0)} name="ball" driven={true}>
          <Sphere radius={0.4} />
          <StandardMaterial base_color="#e11d48" perceptual_roughness={0.4} />
          <RigidBody value="Dynamic" />
          <ColliderConstructor value={{ Sphere: { radius: 0.4 } }} />
          <Restitution value={{ coefficient: 0.7, combine_rule: "Max" }} />
        </Mesh3d>
      </>
    </AbsoluteFill>
  );
}

// the handles name the plugins a headless BevyApp opens (raea/bevy), from the package's index
export const plugins = [avian.plugin, bevy.plugin] as const;

registerRoot(() => <Composition id="plugins-ball" component={Drop} width={640} height={360} fps={30} durationInFrames={90} />);
