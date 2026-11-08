// bevy_feathers' two-state controls (checkbox.rs, radio.rs, toggle_switch.rs, disclosure_toggle.rs; docs/control.feature
// FC0a), as their style systems leave them on the first frame: coloured by Checked and InteractionDisabled from the
// UiTheme, a checkbox's or radio's mark hidden unless checked, a switch's slide at its left or right half. A render has
// no pointer, so hover and press colours need input.
import type { ReactNode } from "react";
import { Node, type NodeProps } from "../ui";
import { BorderRadius, UiRect, percent, px } from "../ui-val";
import { Rot2, UiTransform } from "../ui-transform";
import { fonts, size } from "./constants";
import { icon } from "./display";
import { icons } from "./icons";
import { InheritableText, useThemeColor } from "./theme";
import { tokens, type ThemeToken } from "./tokens";

export interface FeathersToggleProps extends NodeProps {
  /** Checked */
  checked?: boolean;
  /** InteractionDisabled */
  disabled?: boolean;
  /** the caption beside it (a caption()) */
  children?: ReactNode;
}

/** a token by state: [checked][disabled] */
const pick = (checked: boolean, disabled: boolean, t: { on: ThemeToken; onOff: ThemeToken; off: ThemeToken; offOff: ThemeToken }) => (checked ? (disabled ? t.onOff : t.on) : disabled ? t.offOff : t.off);
const CAPTION = { font: fonts.REGULAR, font_size: size.MEDIUM_FONT, weight: 400 } as const;

/** A checkbox: an 18 px box (2 px border, 4 px corners) holding a check mark when checked, beside its caption.
 *  @bevy bevy_feathers::controls::FeathersCheckbox */
export function FeathersCheckbox({ checked = false, disabled = false, children, ...node }: FeathersToggleProps) {
  const border = useThemeColor(pick(checked, disabled, { on: tokens.CHECKBOX_BORDER_CHECKED, onOff: tokens.CHECKBOX_BORDER_CHECKED_DISABLED, off: tokens.CHECKBOX_BORDER, offOff: tokens.CHECKBOX_BORDER_DISABLED }));
  const bg = useThemeColor(pick(checked, disabled, { on: tokens.CHECKBOX_BG_CHECKED, onOff: tokens.CHECKBOX_BG_CHECKED_DISABLED, off: tokens.CHECKBOX_BG, offOff: tokens.CHECKBOX_BG_DISABLED }));
  const mark = useThemeColor(disabled ? tokens.CHECKBOX_MARK_DISABLED : tokens.CHECKBOX_MARK);
  const text = useThemeColor(disabled ? tokens.CHECKBOX_TEXT_DISABLED : tokens.CHECKBOX_TEXT);
  return (
    <Node display="Flex" flex_direction="Row" justify_content="Start" align_items="Center" column_gap={px(4)} {...node}>
      <InheritableText {...CAPTION} color={text}>
        <Node width={size.CHECKBOX_SIZE} height={size.CHECKBOX_SIZE} border={UiRect.all(px(2))} border_radius={BorderRadius.all(px(4))} background_color={bg} border_color={border}>
          <Node position_type="Absolute" left={px(4)} top={px(0)} width={px(6)} height={px(11)} border={UiRect.new(px(0), px(2), px(0), px(2))} ui_transform={UiTransform.from_rotation(Rot2.radians(Math.PI / 4))} border_color={mark} visibility={checked ? "Inherited" : "Hidden"} />
        </Node>
        {children}
      </InheritableText>
    </Node>
  );
}

/** A radio button: an 18 px ring (2 px border) holding a 12 px dot when checked, beside its caption.
 *  @bevy bevy_feathers::controls::FeathersRadio */
export function FeathersRadio({ checked = false, disabled = false, children, ...node }: FeathersToggleProps) {
  const bg = useThemeColor(pick(checked, disabled, { on: tokens.RADIO_BG_CHECKED, onOff: tokens.RADIO_BG_CHECKED_DISABLED, off: tokens.RADIO_BG, offOff: tokens.RADIO_BG_DISABLED }));
  const border = useThemeColor(pick(checked, disabled, { on: tokens.RADIO_BORDER_CHECKED, onOff: tokens.RADIO_BORDER_CHECKED_DISABLED, off: tokens.RADIO_BORDER, offOff: tokens.RADIO_BORDER_DISABLED }));
  const mark = useThemeColor(disabled ? tokens.RADIO_MARK_DISABLED : tokens.RADIO_MARK);
  const text = useThemeColor(disabled ? tokens.RADIO_TEXT_DISABLED : tokens.RADIO_TEXT);
  return (
    <Node display="Flex" flex_direction="Row" justify_content="Start" align_items="Center" column_gap={px(4)} {...node}>
      <InheritableText {...CAPTION} color={text}>
        <Node display="Flex" align_items="Center" justify_content="Center" width={size.RADIO_SIZE} height={size.RADIO_SIZE} border={UiRect.all(px(2))} border_radius={BorderRadius.MAX} border_color={border} background_color={bg}>
          <Node width={px(12)} height={px(12)} border={UiRect.all(px(2))} border_radius={BorderRadius.MAX} background_color={mark} visibility={checked ? "Inherited" : "Hidden"} />
        </Node>
        {children}
      </InheritableText>
    </Node>
  );
}

/** A toggle switch: a 32 x 18 track (2 px border, 5 px corners) and a slide filling its left half, or its right half
 *  when checked. @bevy bevy_feathers::controls::FeathersToggleSwitch */
export function FeathersToggleSwitch({ checked = false, disabled = false, ...node }: Omit<FeathersToggleProps, "children">) {
  const bg = useThemeColor(pick(checked, disabled, { on: tokens.SWITCH_BG_CHECKED, onOff: tokens.SWITCH_BG_CHECKED_DISABLED, off: tokens.SWITCH_BG, offOff: tokens.SWITCH_BG_DISABLED }));
  const border = useThemeColor(pick(checked, disabled, { on: tokens.SWITCH_BORDER_CHECKED, onOff: tokens.SWITCH_BORDER_CHECKED_DISABLED, off: tokens.SWITCH_BORDER, offOff: tokens.SWITCH_BORDER_DISABLED }));
  const slideBg = useThemeColor(pick(checked, disabled, { on: tokens.SWITCH_SLIDE_BG_CHECKED, onOff: tokens.SWITCH_SLIDE_BG_CHECKED_DISABLED, off: tokens.SWITCH_SLIDE_BG, offOff: tokens.SWITCH_SLIDE_BG_DISABLED }));
  const slideBorder = useThemeColor(pick(checked, disabled, { on: tokens.SWITCH_SLIDE_BORDER_CHECKED, onOff: tokens.SWITCH_SLIDE_BORDER_CHECKED_DISABLED, off: tokens.SWITCH_SLIDE_BORDER, offOff: tokens.SWITCH_SLIDE_BORDER_DISABLED }));
  return (
    <Node width={size.TOGGLE_WIDTH} height={size.TOGGLE_HEIGHT} border={UiRect.all(px(2))} border_radius={BorderRadius.all(px(5))} background_color={bg} border_color={border} {...node}>
      <Node position_type="Absolute" left={percent(checked ? 50 : 0)} top={px(0)} bottom={px(0)} width={percent(50)} border={UiRect.all(px(2))} border_radius={BorderRadius.all(px(3))} background_color={slideBg} border_color={slideBorder} />
    </Node>
  );
}

/** A disclosure toggle: a 12 px box holding a right chevron in BUTTON_TEXT, turned a quarter (pointing down) when
 *  checked. @bevy bevy_feathers::controls::FeathersDisclosureToggle */
export function FeathersDisclosureToggle({ checked = false, disabled = false, ...node }: Omit<FeathersToggleProps, "children">) {
  const color = useThemeColor(disabled ? tokens.BUTTON_TEXT_DISABLED : tokens.BUTTON_TEXT);
  return (
    <Node width={px(12)} height={px(12)} display="Flex" align_items="Center" justify_content="Center" ui_transform={UiTransform.from_rotation(Rot2.turn_fraction(checked ? 0.25 : 0))} {...node}>
      <InheritableText color={color}>{icon(icons.CHEVRON_RIGHT)}</InheritableText>
    </Node>
  );
}
