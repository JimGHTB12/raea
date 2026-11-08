// Bevy v0.20.0 examples/ui/widgets/vertical_slider.rs in raea's TSX (docs/control.feature FC0a): a full-window row,
// centred both ways with 50 px gaps, of two columns (10 px rows, centred), each a 20 px "Vertical" or "Horizontal"
// label and a 24 px "50" in FiraSans-Bold srgb(0.9, 0.9, 0.9) over its slider at SliderValue 50 of 0..100: the vertical
// one 12 x 200, a 6 px wide srgb(0.05, 0.05, 0.05) track (3 px corners) centred in it and, in an absolute box 12 px
// below its top, a round 12 x 12 srgb(0.35, 0.75, 0.35) thumb at bottom 50%; the horizontal one 200 x 12, a 6 px high
// track and the thumb at left 50% of a box 12 px short of its right. update_slider_visuals places each thumb by
// thumb_position(50) = 0.5, and nothing is hovered or dragged, so neither is lighter(0.3). Dragging, clicks and keys
// change them only under input. Bevy's Camera2d over the default clear colour is the layered <Canvas3D>, in Bevy's
// default 1280 x 720 window.
import { AbsoluteFill, Composition, registerRoot, staticFile } from "raea";
import { Canvas3D } from "raea/3d";
import { BorderRadius, Node, Text, percent, px } from "raea/ui";
import { Srgba } from "raea/color";

const TRACK = new Srgba(0.05, 0.05, 0.05);
const THUMB = new Srgba(0.35, 0.75, 0.35);
const FONT = "fonts/FiraSans-Bold.ttf";

function Label({ size, children }: { size: number; children: string }) {
  return <Text font={staticFile(FONT)} font_size={{ Px: size }} color={new Srgba(0.9, 0.9, 0.9)}>{children}</Text>;
}

function Thumb({ vertical }: { vertical: boolean }) {
  return <Node display="Flex" width={px(12)} height={px(12)} position_type="Absolute" {...(vertical ? { bottom: percent(50) } : { left: percent(50) })} border_radius={BorderRadius.MAX} background_color={THUMB} />;
}

function Slider({ vertical }: { vertical: boolean }) {
  return vertical ? (
    <Node display="Flex" flex_direction="Row" justify_content="Center" align_items="Stretch" column_gap={px(4)} width={px(12)} height={px(200)}>
      <Node width={px(6)} border_radius={BorderRadius.all(px(3))} background_color={TRACK} />
      <Node display="Flex" position_type="Absolute" top={px(12)} bottom={px(0)} left={px(0)} right={px(0)}>
        <Thumb vertical />
      </Node>
    </Node>
  ) : (
    <Node display="Flex" flex_direction="Column" justify_content="Center" align_items="Stretch" column_gap={px(4)} height={px(12)} width={px(200)}>
      <Node height={px(6)} border_radius={BorderRadius.all(px(3))} background_color={TRACK} />
      <Node display="Flex" position_type="Absolute" left={px(0)} right={px(12)} top={px(0)} bottom={px(0)}>
        <Thumb vertical={false} />
      </Node>
    </Node>
  );
}

function Shot() {
  return (
    <AbsoluteFill>
      <Canvas3D layered clearColor="rgb(43, 44, 47)" />
      <Node width={percent(100)} height={percent(100)} align_items="Center" justify_content="Center" display="Flex" flex_direction="Row" column_gap={px(50)}>
        {[true, false].map((vertical) => (
          <Node key={String(vertical)} display="Flex" flex_direction="Column" align_items="Center" row_gap={px(10)}>
            <Label size={20}>{vertical ? "Vertical" : "Horizontal"}</Label>
            <Label size={24}>50</Label>
            <Slider vertical={vertical} />
          </Node>
        ))}
      </Node>
    </AbsoluteFill>
  );
}

registerRoot(() => <Composition id="main" component={Shot} durationInFrames={11} fps={60} width={1280} height={720} />);
