// Bevy v0.20.0 examples/ui/widgets/feathers_number_input.rs in raea's TSX (docs/control.feature FC0a): under feathers'
// dark UiTheme, a full-window column on WINDOW_BG, 8 px padding and 8 px rows, wrapping into a second column, of 18
// demo fields. Each is a 200 px column of a label (Fira Sans 14 px, TEXT_MAIN) over a row, spaced between, of a
// FeathersNumberInput growing to at most 120 px and a "-" label; the f32 fields put 4 px between label and row, the
// sigil, f64 and i32 fields none. The inputs show their value (Rust's Display: "1", not "1.0") over a slider bar to
// where it sits in a SoftLimit; precision, step and wrap act only on drags; "soft limit + disabled" is dimmed; the
// sigil field has a 4 px TEXT_INPUT_X_AXIS strip and an "X"; units append m, s or ° (PI radians shown as 180°).
// Drags, typing and hover need input. Bevy's Camera2d over the default clear colour is the layered <Canvas3D>, in
// Bevy's default 1280 x 720 window.
import { AbsoluteFill, Composition, registerRoot } from "raea";
import { Canvas3D } from "raea/3d";
import { Node, percent, px } from "raea/ui";
import {
  AngleDegrees, FeathersNumberInput, LengthMeters, NumberInputRange, NumberInputValue, ThemeBackgroundColor, TimeSeconds, UiTheme, create_dark_theme, label, tokens,
  type FeathersNumberInputProps,
} from "raea/feathers";

type Options = Omit<FeathersNumberInputProps, "value">;

function Field({ caption, value, gap, options = {} }: { caption: string; value: NumberInputValue; gap: boolean; options?: Options }) {
  return (
    <Node display="Flex" flex_direction="Column" align_items="Start" width={px(200)} row_gap={gap ? px(4) : undefined}>
      {label(caption)}
      <Node display="Flex" flex_direction="Row" align_items="Center" align_self="Stretch" justify_content="SpaceBetween">
        <FeathersNumberInput value={value} {...options} flex_grow={1} max_width={px(120)} />
        {label("-")}
      </Node>
    </Node>
  );
}

const f32 = (caption: string, v: number, options?: Options) => <Field key={caption} caption={caption} value={NumberInputValue.F32(v)} gap options={options} />;
const f64 = (caption: string, v: number, options?: Options) => <Field key={caption} caption={caption} value={NumberInputValue.F64(v)} gap={false} options={options} />;
const i32 = (caption: string, v: number, options?: Options) => <Field key={caption} caption={caption} value={NumberInputValue.I32(v)} gap={false} options={options} />;

function Shot() {
  return (
    <AbsoluteFill>
      <Canvas3D layered clearColor="rgb(43, 44, 47)" />
      <UiTheme theme={create_dark_theme()}>
        <ThemeBackgroundColor token={tokens.WINDOW_BG}>
          <Node width={percent(100)} height={percent(100)} align_items="Start" justify_content="Start" display="Flex" flex_direction="Column" flex_wrap="Wrap" padding={{ left: px(8), right: px(8), top: px(8), bottom: px(8) }} row_gap={px(8)}>
            {f32("none (bare)", 1)}
            {f32("soft limit", 2, { soft_limit: NumberInputRange.F32(0, 10) })}
            {f32("hard limit", 3, { hard_limit: NumberInputRange.F32(-100, 100) })}
            {f32("soft + hard", 4, { soft_limit: NumberInputRange.F32(0, 10), hard_limit: NumberInputRange.F32(-100, 100) })}
            {f32("precision(0)", 5, { precision: 0 })}
            {f32("precision(2)", 6, { precision: 2 })}
            {f32("precision(4)", 7, { precision: 4 })}
            {f32("step(1.0)", 8, { step: 1 })}
            {f64("f64: soft limit", 1, { soft_limit: NumberInputRange.F64(0, 10) })}
            {f64("f64: soft limit + precision(2)", 1, { soft_limit: NumberInputRange.F64(0, 10), precision: 2 })}
            {i32("i32: bare", 1)}
            {i32("i32: soft limit", 1, { soft_limit: NumberInputRange.I32(0, 10) })}
            <Field key="sigil" caption="precision(2) + sigil" value={NumberInputValue.F32(6)} gap={false} options={{ precision: 2, sigil_color: tokens.TEXT_INPUT_X_AXIS, label_text: "X" }} />
            {f32("soft limit + disabled", 2, { disabled: true, soft_limit: NumberInputRange.F32(0, 10) })}
            {f32("hard limit + wrap", 0, { hard_limit: NumberInputRange.F32(-180, 180), wrap: "Wrap" })}
            {f32("in meters", 2, { units: LengthMeters })}
            {f32("in seconds", 2, { units: TimeSeconds })}
            {f32("in degrees", Math.PI, { units: AngleDegrees })}
          </Node>
        </ThemeBackgroundColor>
      </UiTheme>
    </AbsoluteFill>
  );
}

registerRoot(() => <Composition id="main" component={Shot} durationInFrames={11} fps={60} width={1280} height={720} />);
