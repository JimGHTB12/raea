// bevy_feathers' FeathersNumberInput (number_input.rs on text_input.rs's FeathersTextInputContainer and
// FeathersTextInput; docs/control.feature FC0a), as Bevy spawns it and number_input_init leaves it. A ROW_HEIGHT row
// on TEXT_INPUT_BG with 4 px corners, its left border 4 px in the sigil's colour when it has a label (an "X" in Fira
// Sans 13 px on TEXT_INPUT_LABEL_BG, 6 px each side); then the editable text, growing to fill the row, its value
// centred in Fira Sans 13 px on a 24 px line, over a hard-stop gradient: the slider bar (SLIDER_BAR) from the left to
// where the value sits in its SoftLimit (no bar without one), SLIDER_BG after. The text is the value held within its
// HardLimit, through its units (Dimensionless by default). InteractionDisabled dims the bar, the field and the text.
// NumberInputPrecision, NumberInputStep and NumberInputWrap act only on drags and edits, which need input, as do the
// hover and press colours, the chevrons (shown on hover) and the caret.
import { Node, Text, type NodeProps } from "../ui";
import { EditableText } from "../ui-edit";
import { LinearGradient, ColorStop } from "../ui-gradient";
import { UiRect, percent, px } from "../ui-val";
import { fonts, size } from "./constants";
import { to_border_radius } from "./controls";
import { useThemeColor } from "./theme";
import { tokens, type ThemeToken } from "./tokens";
import { Dimensionless, clamp, thumb_position, unpack, type NumberInputRange, type NumberInputValue, type UnitsFormat } from "./number-value";

/** whether a drag or step past a HardLimit wraps round. @bevy bevy_feathers::controls::NumberInputWrap */
export type NumberInputWrap = "NoWrap" | "Wrap";

export interface FeathersNumberInputProps extends NodeProps {
  /** NumberInputValue: the value shown */
  value: NumberInputValue;
  /** SoftLimit: the slider bar's range (no bar without one) */
  soft_limit?: NumberInputRange;
  /** HardLimit: the value is held within it */
  hard_limit?: NumberInputRange;
  /** NumberInputPrecision: decimal places a drag rounds to */
  precision?: number;
  /** NumberInputStep: a drag's step */
  step?: number;
  /** NumberInputWrap (NoWrap by default) */
  wrap?: NumberInputWrap;
  /** NumberInputUnits: LengthMeters, TimeSeconds, AngleDegrees (Dimensionless by default) */
  units?: UnitsFormat;
  /** InteractionDisabled */
  disabled?: boolean;
  /** the sigil's colour, the left border shown with a label (TEXT_INPUT_LABEL_BG by default) */
  sigil_color?: ThemeToken;
  /** the caption at its left, usually "X", "Y" or "Z" */
  label_text?: string;
}

const WHO = "FeathersNumberInput";

/** A feathers number input: its value centred over a slider bar, in its units. Its own Node fields (flex_grow, a
 *  max_width) patch the input's, as a bsn! patch does. @bevy bevy_feathers::controls::FeathersNumberInput */
export function FeathersNumberInput({ value, soft_limit, hard_limit, precision, step, wrap = "NoWrap", units = Dimensionless, disabled = false, sigil_color = tokens.TEXT_INPUT_LABEL_BG, label_text, ...node }: FeathersNumberInputProps) {
  const [kind, v] = unpack(WHO, value);
  if (precision !== undefined && !Number.isInteger(precision)) throw new TypeError(`${WHO}: precision is a whole number of decimal places, got ${JSON.stringify(precision)}`);
  if (step !== undefined && !(typeof step === "number" && step > 0)) throw new TypeError(`${WHO}: step is above 0, got ${JSON.stringify(step)}`);
  if (wrap !== "NoWrap" && wrap !== "Wrap") throw new TypeError(`${WHO}: wrap is NoWrap or Wrap, got ${JSON.stringify(wrap)}`);
  if (!units || typeof units.format !== "function") throw new TypeError(`${WHO}: units is a UnitsFormat (LengthMeters, TimeSeconds, AngleDegrees), got ${JSON.stringify(units)}`);
  if (label_text !== undefined && typeof label_text !== "string") throw new TypeError(`${WHO}: label_text is a string, got ${JSON.stringify(label_text)}`);
  // number_input_init shows the value as given; number_input_on_insert_value, which follows, holds it in the HardLimit
  const held = hard_limit ? clamp(WHO, hard_limit, kind, v) : v;
  const digits = units.format({ [kind]: held } as NumberInputValue, false);
  const at = soft_limit ? Math.min(Math.max(thumb_position(WHO, soft_limit, kind, held) * 100, 0), 100) : 0;
  const bar = useThemeColor(disabled ? tokens.SLIDER_BAR_DISABLED : tokens.SLIDER_BAR);
  const field = useThemeColor(disabled ? tokens.SLIDER_BG_DISABLED : tokens.SLIDER_BG);
  const text = useThemeColor(disabled ? tokens.TEXT_INPUT_TEXT_DISABLED : tokens.TEXT_INPUT_TEXT);
  const sigil = useThemeColor(sigil_color);
  const background = useThemeColor(tokens.TEXT_INPUT_BG);
  const labelBg = useThemeColor(tokens.TEXT_INPUT_LABEL_BG);
  const labelText = useThemeColor(tokens.TEXT_INPUT_TEXT);
  const labelled = label_text !== undefined;
  return (
    <Node height={size.ROW_HEIGHT} display="Flex" justify_content="Center" align_items="Center" flex_grow={1} border_radius={to_border_radius("All", 4)} column_gap={px(0)} border={UiRect.left(px(labelled ? 4 : 0))} border_color={sigil} background_color={background} {...node}>
      {labelled ? (
        <Node display="Flex" align_items="Center" align_self="Stretch" justify_content="Center" padding={UiRect.horizontal(px(6))} background_color={labelBg}>
          <Text font={fonts.REGULAR} font_size={size.COMPACT_FONT} weight={400} color={labelText}>{label_text}</Text>
        </Node>
      ) : null}
      <EditableText
        flex_grow={1} align_items="Center" align_self="Stretch" border_radius={to_border_radius(labelled ? "Right" : "All", 4)}
        background_gradient={[LinearGradient.new(Math.PI * 0.5, [ColorStop.new(bar, percent(0)), ColorStop.new(bar, percent(at)), ColorStop.new(field, percent(at)), ColorStop.new(field, percent(100))], "Srgba")]}
        cursor_width={0.3} justify="Center" linebreak="NoWrap" font={fonts.REGULAR} font_size={size.COMPACT_FONT} weight={400} line_height={{ Px: 24 }} color={text}
      >
        {digits}
      </EditableText>
    </Node>
  );
}
