// Bevy v0.20.0 examples/ui/widgets/headless_tabs.rs in raea's TSX (docs/control.feature FC0a): on srgb(0.06, 0.07,
// 0.09), a full-window column (24 px padding, 18 px rows, its items at the start, scrolling on y) of four tab lists,
// each under a 17 px srgb(0.72, 0.76, 0.84) label: a strip (srgb(0.10, 0.11, 0.14), a row or for the vertical one a
// column) of tab headers at least 112 x 36 (12 x 8 px padding, a 1 px srgb(0.24, 0.25, 0.30) border) holding a 16 px
// srgb(0.88, 0.89, 0.92) label. update_tab_styles colours each header: the list's SelectedTab srgb(0.18, 0.34, 0.52),
// an InteractionDisabled one srgb(0.10, 0.10, 0.11), the rest srgb(0.13, 0.14, 0.18); nothing has focus, so every
// border keeps its colour. Hover, the keys and clicks change them only under input. Bevy's Camera2d over the default
// clear colour is the layered <Canvas3D>, in Bevy's default 1280 x 720 window.
import { AbsoluteFill, Composition, registerRoot } from "raea";
import { Canvas3D } from "raea/3d";
import { Node, Overflow, Text, UiRect, percent, px } from "raea/ui";
import { Srgba } from "raea/color";

const SELECTED = new Srgba(0.18, 0.34, 0.52);
const DISABLED = new Srgba(0.1, 0.1, 0.11);
const NORMAL = new Srgba(0.13, 0.14, 0.18);

interface Header { label: string; selected?: boolean; disabled?: boolean }

function Label({ children }: { children: string }) {
  return <Text font_size={{ Px: 17 }} color={new Srgba(0.72, 0.76, 0.84)}>{children}</Text>;
}

function Strip({ vertical, headers }: { vertical?: boolean; headers: Header[] }) {
  return (
    <Node display="Flex" flex_direction={vertical ? "Column" : "Row"} align_items="Stretch" background_color={new Srgba(0.1, 0.11, 0.14)}>
      {headers.map((h) => (
        <Node key={h.label} min_width={px(112)} min_height={px(36)} padding={UiRect.axes(px(12), px(8))} border={UiRect.all(px(1))} align_items="Center" justify_content="Center" background_color={h.disabled ? DISABLED : h.selected ? SELECTED : NORMAL} border_color={new Srgba(0.24, 0.25, 0.3)}>
          <Text font_size={{ Px: 16 }} color={new Srgba(0.88, 0.89, 0.92)}>{h.label}</Text>
        </Node>
      ))}
    </Node>
  );
}

function Shot() {
  return (
    <AbsoluteFill>
      <Canvas3D layered clearColor="rgb(43, 44, 47)" />
      <Node width={percent(100)} height={percent(100)} padding={UiRect.all(px(24))} display="Flex" flex_direction="Column" align_items="Start" row_gap={px(18)} overflow={Overflow.scroll_y()} background_color={new Srgba(0.06, 0.07, 0.09)}>
        <Label>Horizontal automatic - self-updating</Label>
        <Strip headers={[{ label: "General", selected: true }, { label: "Rendering" }, { label: "Disabled", disabled: true }]} />
        <Label>Horizontal manual - focus and selection are separate</Label>
        <Strip headers={[{ label: "Scene", selected: true }, { label: "Assets" }, { label: "Inspector" }]} />
        <Label>Vertical manual</Label>
        <Strip vertical headers={[{ label: "Transform", selected: true }, { label: "Visibility" }, { label: "Metadata" }]} />
        <Label>Controlled - observer updates external state</Label>
        <Strip headers={[{ label: "External A", selected: true }, { label: "External B" }]} />
      </Node>
    </AbsoluteFill>
  );
}

registerRoot(() => <Composition id="main" component={Shot} durationInFrames={11} fps={60} width={1280} height={720} />);
