// Bevy v0.20.0 examples/3d/3d_shapes.rs in raea's TSX (docs/control.feature FC0a): Bevy's 3D primitives (front row),
// extrusions of its 2D ones (middle) and extruded rings (rear), each meshed by Bevy's own mesher (a reflected Mesh3d
// handle: { primitive, ...its fields }), all in the uv_debug_texture StandardMaterial (an 8 x 8 palette shifted a texel
// each row, sampled nearest: ImagePlugin::default_nearest), turned -PI/4 about X and then about Y by half Bevy's
// elapsed time (rotate's delta_secs / 2 each update); a 50 m SILVER plane of 10 subdivisions, a shadowed PointLight of
// 10 000 000 to 100 m with depth bias 0.2 at (8, 16, 8), the camera at (0, 7, 14) looking at (0, 1, 0), and the help
// text in Bevy's default font at top 12, left 12. Bevy's default 1280 x 720 window.
import { AbsoluteFill, Composition, registerRoot, staticFile, useCurrentFrame } from "raea";
import { BevyComponent, Canvas3D, Entity } from "raea/3d";
import { Quat, Transform } from "raea/transform";
import { Camera3d } from "raea/camera";
import { PointLight } from "raea/light";

const FONT = `@font-face { font-family: "Fira Mono"; src: url("${staticFile("fonts/FiraMono-Medium.ttf")}"); }`;
const f = Math.fround;
type Prim = { primitive: string } & Record<string, unknown>;

// uv_debug_texture: the palette's 8 colours, rotated right by one colour each row
function debugTexture() {
  const palette = [255, 102, 159, 255, 255, 159, 102, 255, 236, 255, 102, 255, 121, 255, 102, 255, 102, 255, 198, 255, 102, 198, 255, 255, 121, 102, 255, 255, 236, 102, 255, 255];
  const pixels: number[] = [];
  for (let y = 0; y < 8; y++) {
    pixels.push(...palette);
    palette.unshift(...palette.splice(28, 4));
  }
  return { width: 8, height: 8, pixels, nearest: true };
}
const MATERIAL = { base_color_texture: debugTexture() };

const ring = (p: Prim): Prim => ({ ...p, ring: 0.1 });
const extrude = (base: Prim): Prim => ({ primitive: "Extrusion", base, half_depth: 0.5 });
const ROWS: [number, Prim[]][] = [
  [
    4,
    [
      { primitive: "Cuboid" },
      { primitive: "Tetrahedron" },
      { primitive: "Capsule3d" },
      { primitive: "Torus" },
      { primitive: "Cylinder" },
      { primitive: "Cone" },
      { primitive: "ConicalFrustum" },
      { primitive: "Sphere", ico: 5 },
      { primitive: "Sphere", uv: [32, 18] },
      { primitive: "Segment3d" },
      { primitive: "Polyline3d", vertices: [[-0.5, 0, 0], [0.5, 0, 0], [0, 0.5, 0]] },
    ],
  ],
  [
    0,
    [
      { primitive: "Rectangle" },
      { primitive: "Capsule2d" },
      { primitive: "Annulus" },
      { primitive: "Circle" },
      { primitive: "Ellipse" },
      { primitive: "RegularPolygon" },
      { primitive: "Triangle2d" },
      { primitive: "ConvexPolygon", vertices: [[0, 0.8], [-0.47, 0.25], [-0.47, -0.65], [0.47, -0.65], [0.47, 0.25]] },
    ].map(extrude),
  ],
  [
    -4,
    [
      ring({ primitive: "Rectangle" }),
      ring({ primitive: "Capsule2d" }),
      { primitive: "Circle", radius: 1, inner: { radius: 0.5 } },
      ring({ primitive: "Circle" }),
      { primitive: "Ellipse", inner: { half_size: [0.9, 0.4] } },
      ring({ primitive: "RegularPolygon" }),
      ring({ primitive: "Triangle2d" }),
    ].map(extrude),
  ],
];

// the turn about Y at update n: delta_secs / 2 a step summed in f32
function turnAt(n: number): number {
  let a = 0;
  for (let k = 1; k <= n; k++) a = f(a + f(f(16_666_667e-9) / 2));
  return a;
}

function Shot() {
  const tilt = Quat.mul(Quat.fromRotationY(turnAt(useCurrentFrame())), Quat.fromRotationX(-Math.PI / 4));
  return (
    <AbsoluteFill>
      <style>{FONT}</style>
      <Canvas3D layered clearColor="rgb(43, 44, 47)">
        {ROWS.flatMap(([z, shapes], r) =>
          shapes.map((shape, i) => (
            <Entity key={`${r}-${i}`} transform={{ translation: [f(-7 + f(f(i / (shapes.length - 1)) * 14)), 2, z], rotation: tilt }}>
              <BevyComponent type="bevy_mesh::components::Mesh3d" value={shape} />
              <BevyComponent type="bevy_pbr::mesh_material::MeshMaterial3d<bevy_pbr::pbr_material::StandardMaterial>" value={MATERIAL} />
            </Entity>
          )),
        )}
        <PointLight shadow_maps_enabled intensity={10_000_000} range={100} shadow_depth_bias={0.2} transform={Transform.fromXyz(8, 16, 8)} />
        <Entity>
          <BevyComponent type="bevy_mesh::components::Mesh3d" value={{ primitive: "Plane3d", half_size: [25, 25], subdivisions: 10 }} />
          <BevyComponent type="bevy_pbr::mesh_material::MeshMaterial3d<bevy_pbr::pbr_material::StandardMaterial>" value={{ base_color: { Srgba: { red: 0.7529412, green: 0.7529412, blue: 0.7529412, alpha: 1 } } }} />
        </Entity>
        <Camera3d transform={Transform.fromXyz(0, 7, 14).lookingAt([0, 1, 0])} />
      </Canvas3D>
      <div style={{ position: "absolute", top: 12, left: 12, fontFamily: "Fira Mono", fontSize: 20, lineHeight: 1.2, color: "#fff", whiteSpace: "pre" }}>
        {"Press 'R' to pause/resume rotation\nPress 'Tab' to cycle through rows\nPress 'Space' to toggle wireframes"}
      </div>
    </AbsoluteFill>
  );
}

registerRoot(() => <Composition id="main" component={Shot} durationInFrames={121} fps={60} width={1280} height={720} />);
