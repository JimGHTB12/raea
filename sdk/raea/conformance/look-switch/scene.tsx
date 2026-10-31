// One composition, four scenes in Sequences: none, paint, cartoon, none (../look-switch.test.tsx renders a frame of each
// in one process). Each scene its own Canvas3D, as a film's scenes are.
import { AbsoluteFill, Composition, Sequence, registerRoot } from "raea";
import { Look, Sphere } from "raea/3d";
import { StandardMaterial } from "raea/pbr";
import { Mesh3d } from "raea/mesh";
import { DirectionalLight } from "raea/light";
import { Transform } from "raea/transform";
import { Camera3d } from "raea/camera";

export const SCENE = 10;
export const PLAIN_BG = "#304050";
export const PAINT_SKY = "#e05030";
export const CARTOON_SKY = "#30c050";

function Ball({ look }: { look?: "paint" | "cartoon" }) {
  return (
    <AbsoluteFill style={{ background: PLAIN_BG }}>
      <>
        <Camera3d transform={Transform.fromXyz(0, 0, 6).lookingAt([0, 0, 0])} projection={{ Perspective: { fov: 0.698131700797732 } }} />
        {look === "paint" && <Look mode="paint" sky={PAINT_SKY} />}
        {look === "cartoon" && <Look mode="cartoon" sky={CARTOON_SKY} outline="#000000" outlineWidth={8} />}
        <DirectionalLight transform={Transform.fromXyz(3, 4, 5).lookingAt([0, 0, 0])} illuminance={12000} shadow_maps_enabled />
        <Mesh3d name="ball"><Sphere radius={1.2} /><StandardMaterial base_color="#f0f0f0" perceptual_roughness={1} /></Mesh3d>
      </>
    </AbsoluteFill>
  );
}

export function Scenes() {
  const looks = [undefined, "paint", "cartoon", undefined] as const;
  return (
    <AbsoluteFill>
      {looks.map((l, i) => (
        <Sequence key={i} from={i * SCENE} durationInFrames={SCENE}>
          <Ball look={l} />
        </Sequence>
      ))}
    </AbsoluteFill>
  );
}

registerRoot(() => <Composition id="looks" component={Scenes} width={320} height={180} fps={30} durationInFrames={4 * SCENE} />);
