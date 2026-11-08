// Bevy v0.20.0 examples/ui/ui_drag_and_drop.rs in raea's TSX (docs/control.feature FC0a): a grey (0.4) grid centred in
// the window, ten columns by ten rows of 40 px tiles with 4 px borders, alternating srgb(0.2, 0.2, 0.8) and
// srgb(0.8, 0.2, 0.2) (by row parity plus column), each border its tile's colour darker(0.025), each tile showing its
// index (column + row x 10) in Bevy's default font, a 2 px outline of Color::NONE until it's dragged. Hovering lightens a
// tile and dragging lifts it with a white outline and swaps it on a drop: only under a pointer, which no render has.
// Bevy's Camera2d over the default clear colour is the layered <Canvas3D>, in Bevy's default 1280 x 720 window.
import { AbsoluteFill, Composition, registerRoot } from "raea";
import { Canvas3D } from "raea/3d";
import { GridPlacement, Node, Text, UiRect, px } from "raea/ui";
import { Srgba } from "raea/color";

const COLUMNS = 10;
const ROWS = 10;
const TILE_COLORS = [new Srgba(0.2, 0.2, 0.8), new Srgba(0.8, 0.2, 0.2)];

function Tile({ column, row }: { column: number; row: number }) {
  const color = TILE_COLORS[((row % 2) + column) % TILE_COLORS.length];
  return (
    <Node width={px(40)} height={px(40)} border={UiRect.all(px(4))} grid_row={GridPlacement.start(row + 1)} grid_column={GridPlacement.start(column + 1)} align_items="Center" justify_content="Center" border_color={color.darker(0.025)} background_color={color} outline={{ width: px(2), offset: px(0), color: new Srgba(0, 0, 0, 0) }}>
      <Text>{`${column + row * COLUMNS}`}</Text>
    </Node>
  );
}

function Shot() {
  const tiles = [];
  for (let column = 0; column < COLUMNS; column++) for (let row = 0; row < ROWS; row++) tiles.push(<Tile key={`${column},${row}`} column={column} row={row} />);
  return (
    <AbsoluteFill>
      <Canvas3D layered clearColor="rgb(43, 44, 47)" />
      <Node display="Grid" align_self="Center" justify_self="Center" background_color={new Srgba(0.4, 0.4, 0.4)}>{tiles}</Node>
    </AbsoluteFill>
  );
}

registerRoot(() => <Composition id="main" component={Shot} durationInFrames={11} fps={60} width={1280} height={720} />);
