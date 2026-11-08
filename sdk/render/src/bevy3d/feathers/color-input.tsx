// bevy_feathers' colour input (color_input.rs; docs/control.feature FC0a): a menu tool button whose caption is an 18 px
// swatch of its ColorInputValue, a chevron after it. Its popup (a colour wheel or planes, sliders, a hex field and the
// recent colours) is built lazily on a click, so a render without input shows the button alone.
import { px } from "../ui-val";
import type { NodeProps } from "../ui";
import { FeathersColorSwatch, type FeathersColor } from "./color";
import { FeathersLazyMenu, FeathersMenuToolButton } from "./menu";

export interface FeathersColorInputProps extends NodeProps {
  /** ColorInputValue: the colour it holds */
  value?: FeathersColor;
  /** how much of the swatch's right side shows the colour opaque, in percent */
  opaque_color_percentage?: number;
}

/** A colour input: a button showing its colour, opening a colour picker when clicked.
 *  @bevy bevy_feathers::controls::FeathersColorInput */
export function FeathersColorInput({ value, opaque_color_percentage = 0, ...node }: FeathersColorInputProps) {
  return (
    <FeathersLazyMenu {...node}>
      <FeathersMenuToolButton>
        <FeathersColorSwatch color={value} opaque_color_percentage={opaque_color_percentage} corners="None" width={px(18)} min_width={px(18)} height={px(18)} flex_grow={0} align_self="Center" justify_self="Start" />
      </FeathersMenuToolButton>
    </FeathersLazyMenu>
  );
}
