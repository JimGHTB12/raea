// Bevy v0.20.0 examples/ui/images/ui_texture_slice_flip_and_tile.rs in raea's TSX (docs/control.feature FC0a): at
// UiScale 2, a full-window wrapping row (centred, 10 px gaps) of twenty ImageNodes of numbered_slices.png, loaded
// with a nearest sampler and sliced by a TextureSlicer (16 px borders, sides and centre tiled at stretch_value 1):
// for each of 3 x 3, 4 x 4, 5 x 4, 4 x 5 and 5 x 5 cells of 16 px, unflipped, flip_y, flip_x and both. Bevy's
// Camera2d over the default clear colour is the layered <Canvas3D>. In Bevy's default 1280 x 720 window.
import { AbsoluteFill, Composition, registerRoot, staticFile } from "raea";
import { Canvas3D } from "raea/3d";
import { ImageNode, Node, UiScale, percent, px, type TextureSlicer } from "raea/ui";

const IMAGE = { path: staticFile("textures/fantasy_ui_borders/numbered_slices.png"), sampler: { filter: "nearest" as const } };
const SLICER: TextureSlicer = { border: { left: 16, right: 16, top: 16, bottom: 16 }, center_scale_mode: { Tile: { stretch_value: 1 } }, sides_scale_mode: { Tile: { stretch_value: 1 } } };
const GRIDS = [[3, 3], [4, 4], [5, 4], [4, 5], [5, 5]];
const FLIPS = [[false, false], [false, true], [true, false], [true, true]];

function Shot() {
  return (
    <AbsoluteFill>
      <Canvas3D layered clearColor="rgb(43, 44, 47)" />
      <UiScale value={2}>
      <Node width={percent(100)} height={percent(100)} justify_content="Center" align_content="Center" flex_wrap="Wrap" column_gap={px(10)} row_gap={px(10)}>
        {GRIDS.flatMap(([columns, rows]) => FLIPS.map(([flip_x, flip_y]) => (
          <ImageNode key={`${columns}x${rows}${flip_x}${flip_y}`} image={IMAGE} flip_x={flip_x} flip_y={flip_y} image_mode={{ Sliced: SLICER }} width={px(16 * columns)} height={px(16 * rows)} />
        )))}
      </Node>
      </UiScale>
    </AbsoluteFill>
  );
}

registerRoot(() => <Composition id="main" component={Shot} durationInFrames={11} fps={60} width={1280} height={720} />);
