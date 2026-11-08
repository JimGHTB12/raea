// Bevy v0.20.0 examples/ui/layout/grid.rs in raea's TSX: an 800 x 600 window holding a white grid (columns
// min-content and flex 1; rows auto, flex 1 and 20 px): a header across both columns with "Bevy CSS Grid Layout
// Example" in black FiraSans-Bold; a square 0.25 grey grid as tall as its row (24 px padding, 4 x 4 flex tracks, 12 px
// gaps) of sixteen black-framed CSS colours; a black sidebar (rows auto, auto, fr 1; items at the start, centred) with
// "Sidebar" and a 13 px paragraph; a white footer across both columns; and a hidden absolute modal. Bevy's Camera2d
// over the default clear colour is the layered <Canvas3D>.
import { AbsoluteFill, Composition, registerRoot, staticFile } from "raea";
import { Canvas3D } from "raea/3d";
import { GridPlacement, GridTrack, Node, RepeatedGridTrack, Text, UiRect, percent, px } from "raea/ui";
import { Srgba } from "raea/color";

const FONT = staticFile("fonts/FiraSans-Bold.ttf");
const css = (r: number, g: number, b: number) => new Srgba(r / 255, g / 255, b / 255);
const [WHITE, BLACK] = [css(255, 255, 255), css(0, 0, 0)];
const ITEMS = [
  css(255, 165, 0), css(255, 228, 196), css(0, 0, 255), css(220, 20, 60), css(0, 255, 255), css(255, 69, 0), css(0, 100, 0), css(255, 0, 255),
  css(0, 128, 128), css(240, 248, 255), css(220, 20, 60), css(250, 235, 215), css(255, 255, 0), css(255, 20, 147), css(154, 205, 50), css(250, 128, 114),
];
const PARAGRAPH = Array(7).fill("A paragraph of text which ought to wrap nicely.").join(" ");

function Shot() {
  return (
    <AbsoluteFill>
      <Canvas3D layered clearColor="rgb(43, 44, 47)" />
      <Node display="Grid" width={percent(100)} height={percent(100)} grid_template_columns={[GridTrack.min_content(), GridTrack.flex(1)]} grid_template_rows={[GridTrack.auto(), GridTrack.flex(1), GridTrack.px(20)]} background_color={WHITE}>
        <Node display="Grid" grid_column={GridPlacement.span(2)} padding={UiRect.all(px(6))}>
          <Text font={FONT} color={BLACK}>Bevy CSS Grid Layout Example</Text>
        </Node>
        <Node height={percent(100)} aspect_ratio={1} display="Grid" padding={UiRect.all(px(24))} grid_template_columns={[RepeatedGridTrack.flex(4, 1)]} grid_template_rows={[RepeatedGridTrack.flex(4, 1)]} row_gap={px(12)} column_gap={px(12)} background_color={new Srgba(0.25, 0.25, 0.25)}>
          {ITEMS.map((c, i) => (
            <Node key={i} display="Grid" padding={UiRect.all(px(3))} background_color={BLACK}>
              <Node background_color={c} />
            </Node>
          ))}
        </Node>
        <Node display="Grid" align_items="Start" justify_items="Center" padding={UiRect.all(px(10))} grid_template_rows={[GridTrack.auto(), GridTrack.auto(), GridTrack.fr(1)]} row_gap={px(10)} background_color={BLACK}>
          <Text font={FONT}>Sidebar</Text>
          <Text font={FONT} font_size={{ Px: 13 }}>{PARAGRAPH}</Text>
          <Node />
        </Node>
        <Node grid_column={GridPlacement.span(2)} background_color={WHITE} />
        <Node position_type="Absolute" margin={UiRect.new("auto", "auto", px(100), "auto")} width={percent(60)} height={px(300)} max_width={px(600)} visibility="Hidden" background_color={new Srgba(1, 1, 1, 0.8)} />
      </Node>
    </AbsoluteFill>
  );
}

registerRoot(() => <Composition id="main" component={Shot} durationInFrames={11} fps={60} width={800} height={600} />);
