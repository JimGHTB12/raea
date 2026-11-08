// Bevy v0.20.0 examples/2d/sprite_slice.rs in raea's TSX (docs/control.feature FC0a): two rows (y 150 with
// slice_square.png, border 200; y -150 with slice_square_2.png, border 80) of six sprites left to right from x -600,
// 40 px apart: the picture at 100 x 100, stretched to 100 x 200, then nine-sliced (Bevy's SpriteImageMode::Sliced:
// TextureSlicer) stretched, tiled, tiled wider and with its corners constrained to 0.2, each labelled by a FiraSans-Bold
// Text2d centred under it (Anchor::TOP_CENTER, 10 px below). Camera2d, Bevy's default clear colour and 1280 x 720 window.
import { AbsoluteFill, Composition, registerRoot, staticFile } from "raea";
import { Canvas3D, Sprite } from "raea/3d";
import type { SpriteImageMode } from "raea/sprite";
import { Transform } from "raea/transform";
import { Text2d } from "raea/ui";

const tile = (stretch_value: number) => ({ Tile: { stretch_value } });
function cases(border: number): [string, number, number, SpriteImageMode][] {
  return [
    ["Original", 100, 100, "Auto"],
    ["Stretched", 100, 200, "Auto"],
    ["With Slicing", 100, 200, { Sliced: { border, center_scale_mode: "Stretch" } }],
    ["With Tiling", 100, 200, { Sliced: { border, center_scale_mode: tile(0.5), sides_scale_mode: tile(0.2) } }],
    ["With Tiling", 300, 200, { Sliced: { border, center_scale_mode: tile(0.2), sides_scale_mode: tile(0.3) } }],
    ["With Corners Constrained", 300, 200, { Sliced: { border, center_scale_mode: tile(0.1), sides_scale_mode: tile(0.2), max_corner_scale: 0.2 } }],
  ];
}

// spawn_sprites: each sprite's centre, moving right by half its width, then half again and the gap
function row(src: string, y: number, border: number) {
  let x = -600;
  return cases(border).map(([label, w, h, mode]) => {
    x += 0.5 * w;
    const at = x;
    x += 0.5 * w + 40;
    return { src, label, w, h, mode, x: at, y };
  });
}
const SPRITES = [...row("textures/slice_square.png", 150, 200), ...row("textures/slice_square_2.png", -150, 80)];

function Shot() {
  return (
    <AbsoluteFill>
      <Canvas3D layered clearColor="rgb(43, 44, 47)">
        {SPRITES.map((s, i) => (
          <Sprite key={i} src={staticFile(s.src)} width={s.w} height={s.h} image_mode={s.mode} x={640 + s.x} y={360 - s.y} />
        ))}
      </Canvas3D>
      {SPRITES.map((s, i) => (
        <Text2d key={i} transform={Transform.fromXyz(s.x, s.y - 0.5 * s.h - 10, 0)} anchor={[0, 0.5]} justify="Center" font={staticFile("fonts/FiraSans-Bold.ttf")}>
          {s.label}
        </Text2d>
      ))}
    </AbsoluteFill>
  );
}

registerRoot(() => <Composition id="main" component={Shot} durationInFrames={11} fps={60} width={1280} height={720} />);
