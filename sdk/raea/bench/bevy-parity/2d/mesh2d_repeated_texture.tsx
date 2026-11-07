// Bevy v0.20.0 examples/2d/mesh2d_repeated_texture.rs in raea's TSX (docs/control.feature FC0a): three 200 px
// Rectangles with a ColorMaterial texture, each labelled by a Text2d 125 px above its centre. "Control" at x 0 has
// panel-border-010.png on its default sampler. "Repeat On" at x -250 has panel-border-010-repeated.png loaded with
// ImageSamplerDescriptor { address_mode_u/v: Repeat, ..default() } (whose filters are nearest) and a uv_transform of
// Affine2::from_scale(2, 3). "Repeat Off" at x 250 has the first picture with the same uv_transform, so its edges
// clamp. On Bevy's default clear colour, in Bevy's default 1280 x 720 window.
import { AbsoluteFill, Composition, registerRoot, staticFile } from "raea";
import { Canvas3D } from "raea/3d";
import { Transform } from "raea/transform";
import { Affine2 } from "raea/math";
import { ColorMaterial, Mesh2d, Rectangle } from "raea/sprite";
import { Text2d } from "raea/ui";

const SIDE = 200, OFFSET = 250, LABEL = SIDE / 2 + 25;
const PLAIN = staticFile("textures/fantasy_ui_borders/panel-border-010.png");
// ImageSamplerDescriptor's own defaults are nearest filters; the example sets only the address modes
const REPEATED = { path: staticFile("textures/fantasy_ui_borders/panel-border-010-repeated.png"), sampler: { wrap: "repeat", filter: "nearest", mipmap: "nearest" } } as const;
const PANELS = [
  { x: 0, label: "Control", texture: PLAIN, uv: undefined },
  { x: -OFFSET, label: "Repeat On", texture: REPEATED, uv: Affine2.fromScale([2, 3]) },
  { x: OFFSET, label: "Repeat Off", texture: PLAIN, uv: Affine2.fromScale([2, 3]) },
];

function Shot() {
  return (
    <AbsoluteFill>
      <Canvas3D layered clearColor="rgb(43, 44, 47)">
        {PANELS.map((p) => (
          <Mesh2d key={p.label} transform={Transform.fromXyz(p.x, 0, 0)}>
            <Rectangle half_size={[SIDE / 2, SIDE / 2]} />
            <ColorMaterial texture={p.texture} uv_transform={p.uv} />
          </Mesh2d>
        ))}
      </Canvas3D>
      {PANELS.map((p) => <Text2d key={p.label} transform={Transform.fromXyz(p.x, LABEL, 0)}>{p.label}</Text2d>)}
    </AbsoluteFill>
  );
}

registerRoot(() => <Composition id="main" component={Shot} durationInFrames={11} fps={60} width={1280} height={720} />);
