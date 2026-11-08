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
// the widgets under input: the Button reads "Hover" on srgb(0.25, 0.25, 0.25) with a white border while the pointer is over
// it and "Press" on srgb(0.35, 0.75, 0.35) with a red one while pressed there; activating the menu button toggles its
// popup (spawn_menu: a GRAY column, 1 px GREEN border, its shadow, GlobalZIndex 100, placed by its Popover below the
// anchor, else above, 2 px off, 10 px from the window's edge) of four "Menu Item"s, each lit while hovered and green
// while pressed; activating one closes it. The slider, checkbox, radios and 'D' respond to input in Bevy only. Bevy's Camera2d over the default clear colour is the layered <Canvas3D>, in Bevy's
// default 1280 x 720 window.
import { useState } from "react";
import { AbsoluteFill, Composition, registerRoot, staticFile } from "raea";
import { Canvas3D } from "raea/3d";
import { BorderRadius, BoxShadow, Node, Text, TextShadow, UiRect, percent, px } from "raea/ui";
import { Srgba } from "raea/color";

const BOLD = "fonts/FiraSans-Bold.ttf";
const OUTLINE = new Srgba(0.45, 0.45, 0.45);
const FILL = new Srgba(0.35, 0.75, 0.35);
const NONE = new Srgba(0, 0, 0, 0);
const GRAY = new Srgba(0.5019608, 0.5019608, 0.5019608);
// basic::GREEN, and the example's button colours
const GREEN = new Srgba(0, 0.5019608, 0);
const NORMAL_BUTTON = new Srgba(0.15, 0.15, 0.15);
const HOVERED_BUTTON = new Srgba(0.25, 0.25, 0.25);
const PRESSED_BUTTON = new Srgba(0.35, 0.75, 0.35);
const LABEL = { font: staticFile(BOLD), font_size: { Px: 33 }, color: new Srgba(0.9, 0.9, 0.9), shadow: TextShadow.default() } as const;

/** Hovered (the pointer over it or what's in it) and Pressed (pressed on it, until the release), as bevy_picking and
 *  bevy_ui_widgets' Button keep them, and their handlers */
function useHoverPress() {
  const [hovered, setHovered] = useState(false);
  const [pressed, setPressed] = useState(false);
  const on = { onPointerEnter: () => setHovered(true), onPointerLeave: () => setHovered(false), onPointerDown: () => setPressed(true), onPointerUp: () => setPressed(false) };
  return { hovered, pressed, on };
}

/** DemoButton, styled by set_button_style */
function DemoButton() {
  const { hovered, pressed, on } = useHoverPress();
  const [text, bg, border] = hovered && pressed ? ["Press", PRESSED_BUTTON, new Srgba(1, 0, 0)] : hovered ? ["Hover", HOVERED_BUTTON, new Srgba(1, 1, 1)] : ["Button", NORMAL_BUTTON, new Srgba(0, 0, 0)];
  return (
    <Node width={px(150)} height={px(65)} border={UiRect.all(px(5))} border_radius={BorderRadius.MAX} justify_content="Center" align_items="Center" border_color={border} background_color={bg} {...on}>
      <Text {...LABEL}>{text}</Text>
    </Node>
  );
}

/** A DemoMenuItem, styled by set_menu_item_style; activating it closes the menu */
function MenuItem({ close }: { close: () => void }) {
  const { hovered, pressed, on } = useHoverPress();
  return (
    <Node padding={UiRect.axes(px(8), px(2))} justify_content="Center" align_items="Start" background_color={hovered ? (pressed ? PRESSED_BUTTON : HOVERED_BUTTON) : NORMAL_BUTTON} {...on} onClick={close}>
      <Text {...LABEL}>Menu Item</Text>
    </Node>
  );
}

/** The DemoMenuAnchor: its MenuButton toggles the popup spawn_menu makes */
function Menu() {
  const [open, setOpen] = useState(false);
  return (
    <Node>
      <Node width={px(200)} height={px(65)} border={UiRect.all(px(5))} box_sizing="BorderBox" justify_content="SpaceBetween" align_items="Center" padding={UiRect.axes(px(16), px(0))} border_radius={BorderRadius.all(px(5))} border_color={new Srgba(0, 0, 0)} background_color={NORMAL_BUTTON} onClick={() => setOpen((o) => !o)}>
        <Text {...LABEL}>Menu</Text>
        <Node width={px(12)} height={px(12)} background_color={GRAY} />
      </Node>
      {open && (
        <Node display="Flex" flex_direction="Column" min_height={px(10)} min_width={percent(100)} border={UiRect.all(px(1))} position_type="Absolute" border_color={GREEN} background_color={GRAY}
          box_shadow={BoxShadow.new(new Srgba(0, 0, 0, 0.9), px(0), px(0), px(1), px(4))} global_z_index={100}
          popover={{ positions: [{ side: "Bottom", align: "Start", gap: 2 }, { side: "Top", align: "Start", gap: 2 }], window_margin: 10 }}>
          {[0, 1, 2, 3].map((i) => <MenuItem key={i} close={() => setOpen(false)} />)}
        </Node>
      )}
    </Node>
  );
}

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
        <DemoButton />
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
        <Menu />
        <Text>Press 'D' to toggle widget disabled states</Text>
      </Node>
    </AbsoluteFill>
  );
}

registerRoot(() => <Composition id="main" component={Shot} durationInFrames={11} fps={60} width={1280} height={720} />);
