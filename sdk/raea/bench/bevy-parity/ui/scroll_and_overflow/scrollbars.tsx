// Bevy v0.20.0 examples/ui/scroll_and_overflow/scrollbars.rs in raea's TSX: at UiScale 1.25, a 0.1 grey window with
// "Scrolling" over a 200 x 150 grid: a scroll area (grey, 4 px padding, scrolling both ways, ScrollPosition 10 px
// down) of twelve 14 px text rows, an 8 px vertical scrollbar to its right and an 8 px horizontal one below it, 2 px
// apart. Each thumb is grey with a lighter 1 px border and 4 px corners, and turns white only while hovered or
// dragged, so it stays grey. Bevy's Camera2d over the default clear colour is the layered <Canvas3D>. In Bevy's
// default 1280 x 720 window.
import { AbsoluteFill, Composition, registerRoot } from "raea";
import { Canvas3D } from "raea/3d";
import { BorderRadius, GridPlacement, Node, Overflow, RepeatedGridTrack, Scrollbar, ScrollbarThumb, Text, UiRect, UiScale, px } from "raea/ui";
import { Srgba } from "raea/color";

const GRAY1 = new Srgba(0.224, 0.224, 0.243);
const GRAY2 = new Srgba(0.486, 0.486, 0.529);
const GRAY3 = new Srgba(0.71, 0.71, 0.772);
const ROWS = ["Alpha Wolf", "Beta Blocker", "Delta Sleep", "Gamma Ray", "Epsilon Eridani", "Zeta Function", "Lambda Calculus", "Nu Metal", "Pi Day", "Chi Pants", "Psi Powers", "Omega Fatty Acid"];

const thumb = <ScrollbarThumb background_color={GRAY2} border_color={GRAY3} border_radius={BorderRadius.all(px(4))} border={UiRect.all(px(1))} />;

function Shot() {
  return (
    <AbsoluteFill>
      <Canvas3D layered clearColor="rgb(43, 44, 47)" />
      <UiScale value={1.25}>
        <Node display="Flex" flex_direction="Column" position_type="Absolute" left={px(0)} top={px(0)} right={px(0)} bottom={px(0)} padding={UiRect.all(px(3))} row_gap={px(6)} background_color={new Srgba(0.1, 0.1, 0.1)}>
          <Text>Scrolling</Text>
          <Node display="Grid" width={px(200)} height={px(150)} grid_template_columns={[RepeatedGridTrack.flex(1, 1), RepeatedGridTrack.auto(1)]} grid_template_rows={[RepeatedGridTrack.flex(1, 1), RepeatedGridTrack.auto(1)]} row_gap={px(2)} column_gap={px(2)}>
            <Node name="scroll-area" display="Flex" flex_direction="Column" padding={UiRect.all(px(4))} overflow={Overflow.scroll()} background_color={GRAY1} scroll_position={[0, 10]}>
              {ROWS.map((r) => <Text key={r} font_size={{ Px: 14 }}>{r}</Text>)}
            </Node>
            <Scrollbar target="scroll-area" orientation="Vertical" min_thumb_length={8} min_width={px(8)} grid_row={GridPlacement.start(1)} grid_column={GridPlacement.start(2)}>{thumb}</Scrollbar>
            <Scrollbar target="scroll-area" orientation="Horizontal" min_thumb_length={8} min_height={px(8)} grid_row={GridPlacement.start(2)} grid_column={GridPlacement.start(1)}>{thumb}</Scrollbar>
          </Node>
        </Node>
      </UiScale>
    </AbsoluteFill>
  );
}

registerRoot(() => <Composition id="main" component={Shot} durationInFrames={11} fps={60} width={1280} height={720} />);
