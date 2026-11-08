// Bevy v0.20.0 examples/3d/fog.rs in raea's TSX (docs/control.feature FC0a): a stone (#28221B, rough) pyramid of 50
// slabs under four 1 x 3 x 1 pillars, a #126212CC metallic 0.5 glassy sphere scaled 1.75 at (0, 4, 0) blended and out
// of the shadows, all inside an unlit #888888 2 x 1 x 1 box scaled a million drawn both sides, lit by a shadowed
// default PointLight at (0, 1, 0) with no ambient light; DistanceFog of srgb(0.25, 0.25, 0.25), Linear from 5 to 20,
// on a camera orbiting the origin each update (radius 8 + 7 sin(t / 10) at angle t / 5, height 12 - radius / 2) at
// Bevy's elapsed time; the fog's settings in Bevy's default font at top 12, left 12. Bevy's default 1280 x 720 window.
import { AbsoluteFill, Composition, registerRoot, staticFile, useCurrentFrame } from "raea";
import { Canvas3D, Fog } from "raea/3d";
import { Transform } from "raea/transform";
import { Camera3d } from "raea/camera";
import { Cuboid, Mesh3d, Sphere } from "raea/mesh";
import { StandardMaterial } from "raea/pbr";
import { GlobalAmbientLight, PointLight } from "raea/light";
import { Srgba } from "raea/color";

const FONT = `@font-face { font-family: "Fira Mono"; src: url("${staticFile("fonts/FiraMono-Medium.ttf")}"); }`;
const TEXT =
  "Fog Falloff: Linear { start: 5.0, end: 20.0 }\nFog Color: Srgba(Srgba { red: 0.25, green: 0.25, blue: 0.25, alpha: 1.0 })" +
  "\n\n1 / 2 / 3 - Fog Falloff Mode\nA / S - Move Start Distance\nZ / X - Move End Distance" +
  "\n\n- / = - Red\n[ / ] - Green\n; / ' - Blue\n. / ? - Alpha";
const f = Math.fround;

// Time::elapsed_secs at frame f: f steps of 1/60 s, summed in f32
function elapsedAt(n: number): number {
  let t = 0;
  for (let k = 1; k <= n; k++) t = f(t + f(16_666_667e-9));
  return t;
}

function Stone({ at, half }: { at: [number, number, number]; half: [number, number, number] }) {
  return (
    <Mesh3d transform={Transform.fromXyz(...at)}>
      <Cuboid half_size={half} />
      <StandardMaterial base_color="#28221B" perceptual_roughness={1} />
    </Mesh3d>
  );
}

function Shot() {
  const now = elapsedAt(useCurrentFrame());
  const r = f(8 + f(f(Math.sin(f(now / 10))) * 7));
  const a = f(now / 5);
  const eye: [number, number, number] = [f(f(Math.cos(a)) * r), f(12 - f(r / 2)), f(f(Math.sin(a)) * r)];
  return (
    <AbsoluteFill>
      <style>{FONT}</style>
      <Canvas3D layered clearColor="rgb(43, 44, 47)">
        <GlobalAmbientLight brightness={0} />
        <Camera3d transform={Transform.fromXyz(...eye).lookingAt([0, 0, 0])}>
          <Fog color={new Srgba(0.25, 0.25, 0.25, 1)} falloff={{ Linear: { start: 5, end: 20 } }} />
        </Camera3d>
        {[[-1.5, -1.5], [1.5, -1.5], [1.5, 1.5], [-1.5, 1.5]].map(([x, z], i) => (
          <Stone key={`p${i}`} at={[x, 1.5, z]} half={[0.5, 1.5, 0.5]} />
        ))}
        <Mesh3d transform={Transform.fromScale([1.75, 1.75, 1.75]).withTranslation([0, 4, 0])} castShadow={false} receiveShadow={false}>
          <Sphere radius={0.5} ico={5} />
          <StandardMaterial base_color={Srgba.hex("126212CC")} reflectance={1} perceptual_roughness={0} metallic={0.5} alpha_mode="Blend" />
        </Mesh3d>
        {Array.from({ length: 50 }, (_, i) => {
          const h = i / 2 + 3;
          return <Stone key={`s${i}`} at={[0, -i / 2 + 0.25, 0]} half={[h, 0.25, h]} />;
        })}
        <Mesh3d transform={Transform.fromScale([1_000_000, 1_000_000, 1_000_000])}>
          <Cuboid half_size={[1, 0.5, 0.5]} />
          <StandardMaterial base_color="#888888" unlit cull_mode={null} />
        </Mesh3d>
        <PointLight shadow_maps_enabled transform={Transform.fromXyz(0, 1, 0)} />
      </Canvas3D>
      <div style={{ position: "absolute", top: 12, left: 12, fontFamily: "Fira Mono", fontSize: 20, lineHeight: 1.2, color: "#fff", whiteSpace: "pre" }}>{TEXT}</div>
    </AbsoluteFill>
  );
}

registerRoot(() => <Composition id="main" component={Shot} durationInFrames={121} fps={60} width={1280} height={720} />);
