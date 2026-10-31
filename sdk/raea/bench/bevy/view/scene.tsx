import { Mesh3d, Cuboid, Sphere } from "raea/mesh";
import { Camera3d } from "raea/camera";
import { AmbientLight, DirectionalLight } from "raea/light";
import { Transform } from "raea/transform";
import { StandardMaterial } from "raea/pbr";

/** The scene both fixtures draw: its camera where the fixture puts it. */
export function Scene({ t }: { t: number }) {
  return (
    <>
      <Camera3d transform={Transform.fromXyz(0, 1.2, 5).lookingAt([0, 0.3, 0])} />
      <AmbientLight brightness={300} />
      <DirectionalLight transform={Transform.fromXyz(3, 4, 5).lookingAt([0, 0, 0])} illuminance={4000} />
      <Mesh3d transform={Transform.fromXyz(-1, 0, 0)}>
        <Cuboid half_size={[0.6, 0.6, 0.6]} />
        <StandardMaterial base_color="#c84" />
      </Mesh3d>
      <Mesh3d transform={Transform.fromXyz(1.1 - t, 0.2, -0.5)}>
        <Sphere radius={0.7} />
        <StandardMaterial base_color="#3a6cff" perceptual_roughness={0.4} />
      </Mesh3d>
    </>
  );
}
