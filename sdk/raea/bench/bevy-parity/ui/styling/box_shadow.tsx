// Bevy v0.20.0 examples/ui/styling/box_shadow.rs in raea's TSX (docs/control.feature FC0a): on a CSS GRAY window, a
// 164 px dark square (srgb 0.21, a 1 px white border) centred, its BoxShadow black at 0.8 alpha, 20 px right and down,
// spread 15 and blurred 10, under a camera's BoxShadowSamples(6). At the bottom left, 10 px in, a feathers pane over
// the dark theme with its pane body at 0.9 alpha (the example's get_example_theme): the Shape radios (Square checked),
// six number inputs (X and Y offset 20, blur 10, spread 15, count 1, samples 6; each a 150 px label beside a field
// growing from 50 px) and a Reset button, as examples/helpers' radio.rs and number_input_*.rs build them. Editing the
// settings needs input. Bevy's Camera2d over the default clear colour is the layered <Canvas3D>, in Bevy's default
// 1280 x 720 window.
import { AbsoluteFill, Composition, registerRoot } from "raea";
import { Canvas3D } from "raea/3d";
import { Srgba } from "raea/color";
import { BoxShadow, BoxShadowSamples, Node, UiRect, percent, px } from "raea/ui";
import {
  FeathersButton, FeathersNumberInput, FeathersRadio, NumberInputRange, NumberInputValue, UiTheme, caption, create_dark_theme, label, pane, pane_body,
} from "raea/feathers";

const SHAPES = ["Square", "Rounded Square", "Circle", "Long Rectangle", "Tall Rectangle"];

/** get_example_theme: the dark theme, its pane body 0.9 opaque */
function exampleTheme() {
  const props = create_dark_theme();
  props.semantic_base.SURFACE_PANE_BODY = props.semantic_base.SURFACE_PANE_BODY.with_alpha(0.9);
  return props;
}

/** examples/helpers number_input_f32 and number_input_i32: a 150 px label beside a field */
function field(name: string, value: NumberInputValue, limit: NumberInputRange) {
  return (
    <Node key={name} align_items="Center" column_gap={px(5)}>
      <Node align_items="Center" width={px(150)}>{label(name)}</Node>
      <FeathersNumberInput value={value} precision={0} hard_limit={limit} align_items="Center" flex_grow={1} min_width={px(50)} />
    </Node>
  );
}

function Shot() {
  return (
    <AbsoluteFill>
      <Canvas3D layered clearColor="rgb(43, 44, 47)" />
      <BoxShadowSamples samples={6}>
        <UiTheme theme={exampleTheme()}>
          <Node width={percent(100)} height={percent(100)} align_items="Center" justify_content="Center" background_color={new Srgba(0.5, 0.5, 0.5)}>
            <Node width={px(164)} height={px(164)} border={UiRect.all(px(1))} align_items="Center" justify_content="Center" border_color={new Srgba(1, 1, 1)} background_color={new Srgba(0.21, 0.21, 0.21)}
              box_shadow={BoxShadow.new(new Srgba(0, 0, 0, 0.8), px(20), px(20), px(15), px(10))} />
          </Node>
          {pane(
            pane_body([
              <Node key="shape" align_items="Center" column_gap={px(5)}>
                {label("Shape")}
                {SHAPES.map((s, i) => <FeathersRadio key={s} checked={i === 0}>{caption(s)}</FeathersRadio>)}
              </Node>,
              field("X Offset", NumberInputValue.F32(20), NumberInputRange.F32(-200, 200)),
              field("Y Offset", NumberInputValue.F32(20), NumberInputRange.F32(-200, 200)),
              field("Blur", NumberInputValue.F32(10), NumberInputRange.F32(0, 100)),
              field("Spread", NumberInputValue.F32(15), NumberInputRange.F32(-200, 200)),
              field("Count (1 - 3)", NumberInputValue.I32(1), NumberInputRange.I32(1, 3)),
              field("Samples (0 - 15)", NumberInputValue.I32(6), NumberInputRange.I32(0, 15)),
              <FeathersButton key="reset">{caption("Reset")}</FeathersButton>,
            ]),
            // main_ui_node_scene's patch, and the panel's ZIndex(10)
            { flex_direction: "Column", position_type: "Absolute", row_gap: px(6), left: px(10), bottom: px(10), z_index: 10 },
          )}
        </UiTheme>
      </BoxShadowSamples>
    </AbsoluteFill>
  );
}

registerRoot(() => <Composition id="main" component={Shot} durationInFrames={11} fps={60} width={1280} height={720} />);
