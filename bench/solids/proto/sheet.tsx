// One still of the B objects side by side, each part coloured by its material label (labels reaching the renderer).
import { AbsoluteFill, Composition, registerRoot } from "raea";
import { Entity } from "raea/3d";
import { objects } from "./objects.ts";
import { Mesh3d } from "raea/mesh";
import { StandardMaterial } from "raea/pbr";
import { AmbientLight, DirectionalLight } from "raea/light";
import { Transform } from "raea/transform";
import { Camera3d } from "raea/camera";

const COLOR: Record<string, string> = {
  oak: "#b5834a", walnut: "#6b4428", "glazed ceramic": "#e9eef2", bark: "#5a4030", "oak leaves": "#4f7d3a",
};
const OBJS = objects().filter((o) => o.asm && !o.redTeam);
// each object scaled to 1.6 m tall and set in a row
const ROW = OBJS.map((o, i) => {
  const b = o.solid.bounds(), h = b.max[1] - b.min[1], s = 1.6 / h;
  return { o, s, x: (i - (OBJS.length - 1) / 2) * 2.2, cx: (b.min[0] + b.max[0]) / 2, cz: (b.min[2] + b.max[2]) / 2 };
});

function Sheet() {
  return (
    <AbsoluteFill style={{ background: "#dfe3e8" }}>
      <>
        <Camera3d transform={Transform.fromXyz(0, 2.6, 7.5).lookingAt([0, 0.8, 0])} projection={{ Perspective: { fov: 0.785398163397448 } }} />
        <AmbientLight brightness={180} />
        <DirectionalLight transform={Transform.fromXyz(4, 6, 5).lookingAt([0, 0, 0])} illuminance={12800} shadow_maps_enabled />
        {/* a floor was meant here, an r3f <planeGeometry> the old <Mesh> ignored: it never drew (removed in the M5 move) */}
        {ROW.map(({ o, s, x, cx, cz }) => (
          <Entity key={o.name} transform={Transform.fromXyz(x, 0, 0).withScale(s)}>
            {o.asm!.parts.map((p) => (
              <Mesh3d key={p.name} transform={Transform.fromXyz(-cx, 0, -cz)} name={`${o.name}/${p.name}`} shape={p.solid}>
                <StandardMaterial base_color={COLOR[String(p.labels.material)] ?? "#999999"} perceptual_roughness={p.labels.material === "glazed ceramic" ? 0.25 : 0.7} />
              </Mesh3d>
            ))}
          </Entity>
        ))}
      </>
    </AbsoluteFill>
  );
}

registerRoot(() => <Composition id="sheet" component={Sheet} width={1600} height={700} fps={30} durationInFrames={1} />);
