// Bevy v0.20.0 examples/ui/widgets/standard_widgets.rs in raea's TSX (docs/control.feature FC0a): a full-window
// column, centred both ways with 10 px rows, of the headless widgets at rest, as the style systems leave them on the
// first frame. A 150 x 65 round Button (5 px black border on srgb(0.15, 0.15, 0.15)) reading "Button" in
// FiraSans-Bold 33 px srgb(0.9, 0.9, 0.9) with TextShadow's default; a Slider 30% wide and 12 px high, its 6 px
// srgb(0.05, 0.05, 0.05) track (3 px corners) and a round 12 px srgb(0.35, 0.75, 0.35) thumb at left 50% of a box
// 12 px short of its right; a Checkbox (a 16 px box, 2 px srgb(0.45, 0.45, 0.45) border, 3 px corners, its mark
// cleared as it isn't Checked) beside "Checkbox" in FiraSans-Bold 20 px; a RadioGroup of three round radios captioned
// "Slider Drag", "Slider Step" and "Slider Snap", only Snap's 8 px mark shown, as update_widget_values checks the
// radio matching DemoWidgetStates' Snap; a 200 x 65 menu Button (5 px black border, 5 px corners, 16 px padding each
// side, its items spaced between) holding "Menu" as the Button's text and a 12 px gray square; and "Press 'D' to
// toggle widget disabled states" in Bevy's default font. The menu's popup opens, and hover, presses and 'D' change
// the widgets, only under input. Bevy's Camera2d over the default clear colour is the layered <Canvas3D>, in Bevy's
// default 1280 x 720 window.
import { AbsoluteFill, Composition, registerRoot, staticFile } from "raea";
import { Canvas3D } from "raea/3d";
import { BorderRadius, Node, Text, TextShadow, UiRect, percent, px } from "raea/ui";
import { Srgba } from "raea/color";

const BOLD = "fonts/FiraSans-Bold.ttf";
const OUTLINE = new Srgba(0.45, 0.45, 0.45);
const FILL = new Srgba(0.35, 0.75, 0.35);
const NONE = new Srgba(0, 0, 0, 0);
const GRAY = new Srgba(0.5019608, 0.5019608, 0.5019608);

/** a checkbox or radio: its 16 px box with an 8 px mark (cleared unless checked) beside its caption */
function Toggle({ caption, round, checked }: { caption: string; round: boolean; checked: boolean }) {
  return (
    <Node display="Flex" flex_direction="Row" justify_content="FlexStart" align_items="Center" align_content="Center" column_gap={px(4)}>
      <Node display="Flex" width={px(16)} height={px(16)} border={UiRect.all(px(2))} border_radius={round ? BorderRadius.MAX : BorderRadius.all(px(3))} border_color={OUTLINE}>
        <Node display="Flex" width={px(8)} height={px(8)} position_type="Absolute" left={px(2)} top={px(2)} border_radius={round ? BorderRadius.MAX : undefined} background_color={checked ? FILL : NONE} />
      </Node>
      <Text font={staticFile(BOLD)} font_size={{ Px: 20 }}>{caption}</Text>
    </Node>
  );
}

function Shot() {
  return (
    <AbsoluteFill>
      <Canvas3D layered clearColor="rgb(43, 44, 47)" />
      <Node width={percent(100)} height={percent(100)} align_items="Center" justify_content="Center" display="Flex" flex_direction="Column" row_gap={px(10)}>
        <Node width={px(150)} height={px(65)} border={UiRect.all(px(5))} border_radius={BorderRadius.MAX} justify_content="Center" align_items="Center" border_color={new Srgba(0, 0, 0)} background_color={new Srgba(0.15, 0.15, 0.15)}>
          <Text font={staticFile(BOLD)} font_size={{ Px: 33 }} color={new Srgba(0.9, 0.9, 0.9)} shadow={TextShadow.default()}>Button</Text>
        </Node>
        <Node display="Flex" flex_direction="Column" justify_content="Center" align_items="Stretch" justify_items="Center" column_gap={px(4)} height={px(12)} width={percent(30)}>
          <Node height={px(6)} border_radius={BorderRadius.all(px(3))} background_color={new Srgba(0.05, 0.05, 0.05)} />
          <Node display="Flex" position_type="Absolute" left={px(0)} right={px(12)} top={px(0)} bottom={px(0)}>
            <Node display="Flex" width={px(12)} height={px(12)} position_type="Absolute" left={percent(50)} border_radius={BorderRadius.MAX} background_color={FILL} />
          </Node>
        </Node>
        <Toggle caption="Checkbox" round={false} checked={false} />
        <Node display="Flex" flex_direction="Column" align_items="Start" column_gap={px(4)}>
          <Toggle caption="Slider Drag" round checked={false} />
          <Toggle caption="Slider Step" round checked={false} />
          <Toggle caption="Slider Snap" round checked />
        </Node>
        <Node>
          <Node width={px(200)} height={px(65)} border={UiRect.all(px(5))} box_sizing="BorderBox" justify_content="SpaceBetween" align_items="Center" padding={UiRect.axes(px(16), px(0))} border_radius={BorderRadius.all(px(5))} border_color={new Srgba(0, 0, 0)} background_color={new Srgba(0.15, 0.15, 0.15)}>
            <Text font={staticFile(BOLD)} font_size={{ Px: 33 }} color={new Srgba(0.9, 0.9, 0.9)} shadow={TextShadow.default()}>Menu</Text>
            <Node width={px(12)} height={px(12)} background_color={GRAY} />
          </Node>
        </Node>
        <Text>Press 'D' to toggle widget disabled states</Text>
      </Node>
    </AbsoluteFill>
  );
}

registerRoot(() => <Composition id="main" component={Shot} durationInFrames={11} fps={60} width={1280} height={720} />);
