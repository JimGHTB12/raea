// A half-transparent unlit white square filling the middle of a layered Canvas3D (layered-clear's scenes).
import { Canvas3D } from "raea/3d";
import { Camera3d } from "raea/camera";
import { Srgba } from "raea/color";
import { Cuboid, Mesh3d } from "raea/mesh";
import { StandardMaterial } from "raea/pbr";
import { Transform } from "raea/transform";

export const Square = ({ clearColor }: { clearColor?: string }) => (
  <Canvas3D layered clearColor={clearColor}>
    <Camera3d transform={Transform.IDENTITY} />
    <Mesh3d transform={Transform.fromXyz(0, 0, -2)}>
      <Cuboid half_size={[0.5, 0.5, 0.01]} />
      <StandardMaterial base_color={new Srgba(1, 1, 1, 0.5)} alpha_mode="Blend" unlit />
    </Mesh3d>
  </Canvas3D>
);
