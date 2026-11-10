// DR4's proof's 3D scene, shared by ride.tsx and ride-cheat.tsx: a seat (its base and back) turned by `yaw`, and a
// rider on it at `rider` in the seat's own space.
import { AbsoluteFill } from "raea";
import { Camera3d } from "raea/camera";
import { DirectionalLight } from "raea/light";
import { Cuboid, Mesh3d } from "raea/mesh";
import { StandardMaterial } from "raea/pbr";
import { Quat, Transform } from "raea/transform";

/** A point `[x, y, z]` of the seat's own space, turned by `yaw` about its base at (0, 0.5, 0). */
const on = (yaw: number, [x, y, z]: number[]) => Transform.fromXyz(x * Math.cos(yaw) + z * Math.sin(yaw), 0.5 + y, -x * Math.sin(yaw) + z * Math.cos(yaw)).withRotation(Quat.fromEuler("XYZ", 0, yaw, 0));

export function Seat({ yaw, rider }: { yaw: number; rider: number[] }) {
  return (
    <AbsoluteFill style={{ background: "#1d2430" }}>
      <Camera3d transform={Transform.fromXyz(2.5, 2, 4).lookingAt([0, 0.8, 0])} />
      <DirectionalLight illuminance={9000} transform={Transform.fromXyz(2, 4, 6).lookingAt([0, 0, 0])} />
      <Mesh3d name="seat" transform={on(yaw, [0, 0, 0])}>
        <Cuboid half_size={[0.3, 0.05, 0.3]} />
        <StandardMaterial base_color="#8a5a3c" />
      </Mesh3d>
      <Mesh3d name="seat-back" transform={on(yaw, [0, 0.45, -0.25])}>
        <Cuboid half_size={[0.3, 0.4, 0.05]} />
        <StandardMaterial base_color="#8a5a3c" />
      </Mesh3d>
      <Mesh3d name="rider" transform={on(yaw, rider)}>
        <Cuboid half_size={[0.2, 0.3, 0.15]} />
        <StandardMaterial base_color="#3fa7ff" />
      </Mesh3d>
    </AbsoluteFill>
  );
}
