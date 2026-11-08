// Bevy v0.20.0 examples/ui/scroll_and_overflow/drag_to_scroll.rs in raea's TSX: at UiScale 0.5, a window-filling node
// scrolling both ways (at 0 until dragged) over a grid of 60 rows by 40 columns of 100 px tiles, every other one black
// and the rest hues across (270 degrees) and down (90) at full saturation, half lightness; a tile turns red only
// under the pointer. Bevy's Camera2d over the default clear colour is the layered <Canvas3D>. In Bevy's default
// 1280 x 720 window.
import { AbsoluteFill, Composition, registerRoot } from "raea";
import { Canvas3D } from "raea/3d";
import { GridPlacement, Node, Overflow, RepeatedGridTrack, UiScale, percent } from "raea/ui";
import { Hsla, Srgba } from "raea/color";

const [W, H] = [60, 40];
const tile = (x: number, y: number) => ((x + y) % 2 === 1 ? new Hsla(Math.fround(Math.fround((x / W) * 270) + Math.fround((y / H) * 90)), 1, 0.5).to_srgba() : new Srgba(0, 0, 0));

function Shot() {
  return (
    <AbsoluteFill>
      <Canvas3D layered clearColor="rgb(43, 44, 47)" />
      <UiScale value={0.5}>
        <Node width={percent(100)} height={percent(100)} overflow={Overflow.scroll()}>
          <Node display="Grid" grid_template_rows={[RepeatedGridTrack.px(W, 100)]} grid_template_columns={[RepeatedGridTrack.px(H, 100)]}>
            {Array.from({ length: H }, (_, y) => Array.from({ length: W }, (_, x) => (
              <Node key={`${x},${y}`} grid_row={GridPlacement.start(y + 1)} grid_column={GridPlacement.start(x + 1)} background_color={tile(x, y)} />
            )))}
          </Node>
        </Node>
      </UiScale>
    </AbsoluteFill>
  );
}

registerRoot(() => <Composition id="main" component={Shot} durationInFrames={11} fps={60} width={1280} height={720} />);
