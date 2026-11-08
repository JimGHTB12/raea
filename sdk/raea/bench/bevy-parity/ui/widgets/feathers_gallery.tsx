// Bevy v0.20.0 examples/ui/widgets/feathers_gallery.rs in raea's TSX (docs/control.feature FC0a): every bevy_feathers
// widget under the dark UiTheme, in three columns on WINDOW_BG, 8 px apart. The first (30%, at least 200 px) holds the
// buttons, a menu and a lazy menu, two selects, a split button row, the toggles, checkboxes and radios, a slider, and
// the colour controls of three demo colours (EMERALD_800 at 0.7 alpha, AMBER_800 as Hsla and as Okhsla): the hex field,
// swatches, the red-blue and okhsl hue-lightness planes, and a slider per channel, as update_colors sets them on the
// first frame. The second (30%) holds a pane of tool buttons over a sub-pane with a group of number inputs, and a list
// sub-pane; the third (15%) two colour inputs. Popups, hover, focus rings and drags need input. Bevy's Camera2d over the
// default clear colour is the layered <Canvas3D>, in Bevy's default 1280 x 720 window.
import { AbsoluteFill, Composition, registerRoot } from "raea";
import { Canvas3D } from "raea/3d";
import { Hsla, Okhsla, Srgba } from "raea/color";
import { Node, UiRect, percent, px } from "raea/ui";
import {
  FeathersButton, FeathersCheckbox, FeathersColorInput, FeathersColorPlane, FeathersColorSlider, FeathersColorSwatch, FeathersDisclosureToggle, FeathersLazyMenu, FeathersListRow, FeathersListView,
  FeathersMenu, FeathersMenuButton, FeathersMenuDivider, FeathersMenuItem, FeathersMenuPopup, FeathersMenuToolButton, FeathersNumberInput, FeathersRadio, FeathersSelect, FeathersSlider,
  FeathersTextInput, FeathersTextInputContainer, FeathersToggleSwitch, FeathersToolButton, NumberInputRange, NumberInputValue, ThemeBackgroundColor, UiTheme, caption, create_dark_theme, flex_spacer,
  group, group_body, group_header, icon, icons, label, label_dim, label_small, list_rows_from_strings, pane, pane_body, pane_header, pane_header_divider, subpane, subpane_body, subpane_header,
  tokens, type ColorChannel,
} from "raea/feathers";

// DemoWidgetStates
const rgb = Srgba.hex("#065F46").with_alpha(0.7);
const amber800 = Srgba.hex("#92400E");
const hsl = Hsla.from(amber800);
const okhsl = Okhsla.from(amber800);

const row = { display: "Flex", flex_direction: "Row", align_items: "Center", justify_content: "Start" } as const;
const column = { display: "Flex", flex_direction: "Column", align_items: "Stretch", justify_content: "Start", padding: UiRect.all(px(8)), row_gap: px(8) } as const;
const between = { display: "Flex", flex_direction: "Row", align_items: "Center", justify_content: "SpaceBetween" } as const;

/** update_colors: each slider's value its channel of the demo colour, its base that colour */
function slider(channel: ColorChannel) {
  const [base, value]: [Srgba | Hsla | Okhsla, number] =
    channel === "Red" ? [rgb, rgb.red] : channel === "Green" ? [rgb, rgb.green] : channel === "Blue" ? [rgb, rgb.blue] : channel === "Alpha" ? [rgb, rgb.alpha]
    : channel === "HslHue" ? [hsl, hsl.hue] : channel === "HslSaturation" ? [hsl, hsl.saturation] : channel === "HslLightness" ? [hsl, hsl.lightness]
    : channel === "OkhslHue" ? [okhsl, okhsl.hue] : channel === "OkhslSaturation" ? [okhsl, okhsl.saturation] : [okhsl, okhsl.lightness];
  return <FeathersColorSlider key={channel} channel={channel} value={value} base_color={base} />;
}

const MONTHS = ["January", "February", "March", "April", "May", "June", "July", "August", "September", "October", "November", "December"];

function Column1() {
  return (
    <Node {...column} width={percent(30)} min_width={px(200)}>
      <Node {...row} column_gap={px(8)}>
        <FeathersButton flex_grow={1}>{caption("Normal")}</FeathersButton>
        <FeathersButton flex_grow={1} disabled>{caption("Disabled")}</FeathersButton>
        <FeathersButton variant="Primary" flex_grow={1}>{caption("Primary")}</FeathersButton>
        <FeathersMenu>
          <FeathersMenuButton flex_grow={1}>{caption("Menu")}</FeathersMenuButton>
          <FeathersMenuPopup>
            <FeathersMenuItem>{caption("MenuItem 1")}</FeathersMenuItem>
            <FeathersMenuItem>{caption("MenuItem 2")}</FeathersMenuItem>
            <FeathersMenuDivider />
            <FeathersMenuItem>{caption("MenuItem 3")}</FeathersMenuItem>
          </FeathersMenuPopup>
        </FeathersMenu>
        <FeathersLazyMenu>
          <FeathersMenuToolButton flex_grow={1}>{caption("Θ")}</FeathersMenuToolButton>
        </FeathersLazyMenu>
      </Node>
      <FeathersSelect options={list_rows_from_strings(["One", "Two", "Three"], 0)} flex_grow={1} />
      <FeathersSelect options={list_rows_from_strings(MONTHS, 0)} max_visible={6} flex_grow={1} />
      <Node {...row} column_gap={px(1)}>
        <FeathersButton corners="Left" flex_grow={1}>{caption("Left")}</FeathersButton>
        <FeathersButton corners="None" flex_grow={1}>{caption("Center")}</FeathersButton>
        <FeathersButton variant="Primary" corners="Right" flex_grow={1}>{caption("Right")}</FeathersButton>
      </Node>
      <Node {...row} column_gap={px(8)}>
        <FeathersButton flex_grow={1}>{caption("Toggle override")}</FeathersButton>
        <FeathersButton flex_grow={1}>{caption("Quit…")}</FeathersButton>
      </Node>
      <Node {...row} column_gap={px(8)}>
        {label("Dialog:")}
        <FeathersToggleSwitch />
      </Node>
      <FeathersCheckbox checked>{caption("Checkbox")}</FeathersCheckbox>
      <FeathersCheckbox>{caption("Fast Click Checkbox")}</FeathersCheckbox>
      <FeathersCheckbox disabled>{caption("Disabled")}</FeathersCheckbox>
      <FeathersCheckbox disabled checked>{caption("Checked+Disabled")}</FeathersCheckbox>
      <Node {...row} column_gap={px(8)}>
        <Node display="Flex" flex_direction="Column" row_gap={px(4)}>
          <FeathersRadio checked>{caption("One")}</FeathersRadio>
          <FeathersRadio>{caption("Two")}</FeathersRadio>
          <FeathersRadio>{caption("Fast Click")}</FeathersRadio>
          <FeathersRadio disabled>{caption("Disabled")}</FeathersRadio>
        </Node>
      </Node>
      <Node {...row} column_gap={px(8)}>
        <FeathersToggleSwitch />
        <FeathersToggleSwitch />
        <FeathersToggleSwitch disabled />
        <FeathersToggleSwitch disabled checked />
        <FeathersDisclosureToggle />
      </Node>
      <FeathersSlider max={100} value={20} step={10} precision={2} />
      <Node {...between} column_gap={px(4)}>
        {label("Srgba")}
        {flex_spacer()}
        {/* the example's InheritableFont { font: MONO } on the input reaches no text: its PropagateOver<TextFont> keeps the
            propagated font off the input's own text, so Bevy draws it in feathers' REGULAR face, as here */}
        <FeathersTextInputContainer flex_grow={0} padding={UiRect.left(px(4))}>
          <FeathersTextInput visible_width={10} max_characters={9}>{rgb.to_hex()}</FeathersTextInput>
        </FeathersTextInputContainer>
        <FeathersColorSwatch color={rgb} opaque_color_percentage={30} />
      </Node>
      <FeathersColorPlane plane="RedBlue" value={[rgb.red, rgb.blue, rgb.green]} />
      {(["Red", "Green", "Blue", "Alpha"] as const).map(slider)}
      <Node {...between}>
        {label("Hsl")}
        <FeathersColorSwatch color={hsl} />
      </Node>
      {(["HslHue", "HslSaturation", "HslLightness"] as const).map(slider)}
      <Node {...between}>
        {label("Okhsl")}
        <FeathersColorSwatch color={okhsl} />
      </Node>
      <FeathersColorPlane plane="OkhslHueLightness" value={[okhsl.hue / 360, okhsl.lightness, okhsl.saturation]} />
      {(["OkhslHue", "OkhslSaturation", "OkhslLightness"] as const).map(slider)}
    </Node>
  );
}

const axis = (sigil: string, token: (typeof tokens)[keyof typeof tokens], v: number, disabled = false) => (
  <FeathersNumberInput key={sigil} value={NumberInputValue.F32(v)} precision={2} sigil_color={token} label_text={sigil} flex_grow={1} disabled={disabled} hard_limit={disabled ? NumberInputRange.F32(0, 1) : undefined} />
);

function Column2() {
  const tool = (variant: "Primary" | "Plain", c: ReturnType<typeof caption>) => <FeathersToolButton variant={variant}>{c}</FeathersToolButton>;
  return (
    <Node {...column} width={percent(30)} min_width={px(200)}>
      {pane([
        pane_header([
          tool("Primary", caption("Θ")), pane_header_divider(), tool("Plain", caption("¼")), tool("Plain", caption("½")), tool("Plain", caption("¾")), pane_header_divider(),
          tool("Plain", icon(icons.CHEVRON_DOWN)), flex_spacer(), tool("Plain", icon(icons.X)),
        ]),
        pane_body([
          label_dim("A standard editor pane"),
          subpane([
            subpane_header([caption("Left"), caption("Center"), caption("Right")]),
            subpane_body([
              label_dim("A standard sub-pane"),
              group([
                group_header([caption("Group")]),
                group_body([
                  label("A standard group"),
                  label_small("Scalar property"),
                  <FeathersNumberInput value={NumberInputValue.F32(7)} precision={2} hard_limit={NumberInputRange.F32(0, 100)} flex_grow={1} max_width={px(100)} />,
                  label_small("Scalar property (copy)"),
                  <FeathersNumberInput value={NumberInputValue.F32(7)} precision={4} flex_grow={1} max_width={px(100)} />,
                  label_small("Vec3 property"),
                  <Node {...between} column_gap={px(6)}>{[axis("X", tokens.TEXT_INPUT_X_AXIS, 10.1), axis("Y", tokens.TEXT_INPUT_Y_AXIS, 7.124), axis("Z", tokens.TEXT_INPUT_Z_AXIS, 100)]}</Node>,
                  label_small("Color property"),
                  <Node {...between} column_gap={px(6)}>{[axis("R", tokens.TEXT_INPUT_X_AXIS, 0, true), axis("G", tokens.TEXT_INPUT_Y_AXIS, 0, true), axis("B", tokens.TEXT_INPUT_Z_AXIS, 0, true), axis("A", tokens.TEXT_INPUT_W_AXIS, 0, true)]}</Node>,
                ]),
              ]),
            ]),
          ]),
        ]),
      ])}
      {subpane([
        subpane_header([caption("List")]),
        subpane_body([
          <FeathersListView max_height={px(130)}>
            {["First World", "Second Nature", "Third Degree", "Fourth Wall", "Fifth Column", "Sixth Sense", "Seventh Heaven", "Eighth Wonder", "Ninth Inning", "Tenth Amendment", "Eleventh Hour", "Twelfth Night"].map((t, i) => (
              <FeathersListRow key={t} selected={i === 1} disabled={i === 3}>{caption(t)}</FeathersListRow>
            ))}
          </FeathersListView>,
        ]),
      ])}
    </Node>
  );
}

function Column3() {
  return (
    <Node {...column} width={percent(15)} min_width={px(100)}>
      <Node {...between}>
        {label("Armor Tint")}
        <FeathersColorInput value={Srgba.hex("#D97706")} />
      </Node>
      <Node {...between}>
        {label("Skin Color")}
        <FeathersColorInput value={Srgba.hex("#1E40AF")} />
      </Node>
    </Node>
  );
}

function Shot() {
  return (
    <AbsoluteFill>
      <Canvas3D layered clearColor="rgb(43, 44, 47)" />
      <UiTheme theme={create_dark_theme()}>
        <ThemeBackgroundColor token={tokens.WINDOW_BG}>
          <Node width={percent(100)} height={percent(100)} align_items="Start" justify_content="Start" display="Flex" flex_direction="Row" column_gap={px(8)}>
            <Column1 />
            <Column2 />
            <Column3 />
          </Node>
        </ThemeBackgroundColor>
      </UiTheme>
    </AbsoluteFill>
  );
}

registerRoot(() => <Composition id="main" component={Shot} durationInFrames={11} fps={60} width={1280} height={720} />);
