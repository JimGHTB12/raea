// bevy_feathers' FeathersSlider (slider.rs; docs/control.feature FC0a), as update_slider_pos and set_slider_styles
// leave it: a ROW_HEIGHT bar with 6 px corners, filled with SLIDER_BAR from its left to the value's place in its range
// and SLIDER_BG after (a hard-stop gradient), its value centred in Fira Mono 12 px. The value reads as Rust's {} on the
// f32, or at SliderPrecision's decimal places when it has no more decimals than that. Dragging needs input.
import { Node, type NodeProps } from "../ui";
import { LinearGradient, ColorStop } from "../ui-gradient";
import { UiRect, percent, px } from "../ui-val";
import { fonts, size } from "./constants";
import { to_border_radius } from "./controls";
import { display_f32 } from "./number-value";
import { InheritableText, ThemedText, useThemeColor } from "./theme";
import { tokens } from "./tokens";

export interface FeathersSliderProps extends NodeProps {
  /** SliderValue (min by default) */
  value?: number;
  /** SliderRange's ends (0 to 1 by default) */
  min?: number;
  max?: number;
  /** SliderPrecision: decimal places shown (0 by default) */
  precision?: number;
  /** SliderStep: a drag's or key's step */
  step?: number;
  /** InteractionDisabled */
  disabled?: boolean;
}

/** update_slider_pos's label: Rust's {} on the f32, or {:.precision$} when that has no more decimals than precision */
export function slider_label(value: number, precision: number): string {
  const label = display_f32(Math.fround(value));
  const dot = label.indexOf(".");
  const decimals = dot < 0 ? precision : label.length - dot - 1;
  return precision >= 0 && decimals <= precision ? Math.fround(value).toFixed(precision) : label;
}

/** A slider: a bar filled to its value, the value written on it. @bevy bevy_feathers::controls::FeathersSlider */
export function FeathersSlider({ value, min = 0, max = 1, precision = 0, step, disabled = false, ...node }: FeathersSliderProps) {
  const v = value ?? min;
  for (const [k, x] of [["value", v], ["min", min], ["max", max]] as const) if (!Number.isFinite(x)) throw new TypeError(`FeathersSlider: ${k} is a number, got ${x}`);
  if (!Number.isInteger(precision)) throw new TypeError(`FeathersSlider: precision is a whole number of decimal places, got ${precision}`);
  if (step !== undefined && !(step > 0)) throw new TypeError(`FeathersSlider: step is above 0, got ${step}`);
  // SliderRange::thumb_position: where the value sits (0.5 on an empty range), clamped to 0..100%
  const t = max > min ? (Math.min(Math.max(v, min), max) - min) / (max - min) : 0.5;
  const at = Math.min(Math.max(Math.fround(t) * 100, 0), 100);
  const bar = useThemeColor(disabled ? tokens.SLIDER_BAR_DISABLED : tokens.SLIDER_BAR);
  const bg = useThemeColor(disabled ? tokens.SLIDER_BG_DISABLED : tokens.SLIDER_BG);
  const text = useThemeColor(disabled ? tokens.SLIDER_TEXT_DISABLED : tokens.SLIDER_TEXT);
  return (
    <Node height={size.ROW_HEIGHT} justify_content="Center" align_items="Center" padding={UiRect.horizontal(px(8))} flex_grow={1} border_radius={to_border_radius("All", 6)} background_gradient={[LinearGradient.new(Math.PI * 0.5, [ColorStop.new(bar, percent(0)), ColorStop.new(bar, percent(at)), ColorStop.new(bg, percent(at)), ColorStop.new(bg, percent(100))], "Srgba")]} {...node}>
      <InheritableText color={text}>
        <Node display="Flex" position_type="Absolute" flex_direction="Row" align_items="Center" justify_content="Center">
          <InheritableText font={fonts.MONO} font_size={size.SMALL_FONT} weight={400}><ThemedText linebreak="NoWrap">{slider_label(v, precision)}</ThemedText></InheritableText>
        </Node>
      </InheritableText>
    </Node>
  );
}
