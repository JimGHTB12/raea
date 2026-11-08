// Bevy v0.20.0 examples/ui/navigation/directional_navigation.rs in raea's TSX (docs/control.feature FC0a): a full-window
// root holding three absolute panels at left 20, 280 px wide with 12 px padding and 8 px corners (the instructions at
// top 20 on srgba(0.1, 0.1, 0.1, 0.8); "Focused: ..." 80 px from the bottom on srgba(0.1, 0.5, 0.1, 0.8) and "Last Key:
// None" 20 px from it on srgba(0.5, 0.1, 0.5, 0.8), both 20 px), and ten absolute 140 x 80 buttons (tailwind
// BLUE_400, 4 px borders, 12 px corners) scattered at fixed places, each its centred "Button N", the fifth scaled 1.2
// and turned FRAC_PI_2 by its UiTransform. init_focus gives Button 1 AutoFocus, so with InputFocusVisible its border is
// BLUE_50 and the focus panel reads "Focused: Button 1"; every other border is BorderColor::DEFAULT (none). The arrows,
// D-pad and Enter move and press only under input. Bevy's Camera2d over the default clear colour is the layered
// <Canvas3D>, in Bevy's default 1280 x 720 window.
import type { ReactNode } from "react";
import { AbsoluteFill, Composition, registerRoot } from "raea";
import { Canvas3D } from "raea/3d";
import { BorderRadius, Node, Rot2, Text, UiRect, UiTransform, percent, px } from "raea/ui";
import { Srgba } from "raea/color";

// tailwind BLUE_400 and BLUE_50 (bevy_color's palettes::tailwind)
const NORMAL_BUTTON = new Srgba(0.3764706, 0.64705884, 0.98039216);
const FOCUSED_BORDER = new Srgba(0.9372549, 0.9647059, 1.0);
const POSITIONS: [number, number][] = [[350, 100], [520, 120], [700, 90], [380, 220], [600, 240], [450, 340], [620, 360], [360, 480], [540, 460], [720, 490]];
const FOCUSED = 0;

function Panel({ top, bottom, color, children }: { top?: number; bottom?: number; color: Srgba; children: ReactNode }) {
  return (
    <Node position_type="Absolute" left={px(20)} top={top === undefined ? undefined : px(top)} bottom={bottom === undefined ? undefined : px(bottom)} width={px(280)} padding={UiRect.all(px(12))} border_radius={BorderRadius.all(px(8))} background_color={color}>
      {children}
    </Node>
  );
}

function Shot() {
  return (
    <AbsoluteFill>
      <Canvas3D layered clearColor="rgb(43, 44, 47)" />
      <Node width={percent(100)} height={percent(100)}>
        <Panel top={20} color={new Srgba(0.1, 0.1, 0.1, 0.8)}>
          <Text>{"Directional Navigation Demo\n\nUse arrow keys or D-pad to navigate.\nPress Enter or A button to interact.\n\nButtons are scattered irregularly,\nbut navigation is automatic!"}</Text>
        </Panel>
        <Panel bottom={80} color={new Srgba(0.1, 0.5, 0.1, 0.8)}>
          <Text font_size={{ Px: 20 }}>{`Focused: Button ${FOCUSED + 1}`}</Text>
        </Panel>
        <Panel bottom={20} color={new Srgba(0.5, 0.1, 0.5, 0.8)}>
          <Text font_size={{ Px: 20 }}>Last Key: None</Text>
        </Panel>
        {POSITIONS.map(([x, y], i) => (
          <Node key={i} position_type="Absolute" left={px(x)} top={px(y)} width={px(140)} height={px(80)} border={UiRect.all(px(4))} justify_content="Center" align_items="Center" border_radius={BorderRadius.all(px(12))} background_color={NORMAL_BUTTON} border_color={i === FOCUSED ? FOCUSED_BORDER : undefined} ui_transform={i === 4 ? { scale: [1.2, 1.2], rotation: Rot2.radians(Math.PI / 2) } : undefined}>
            <Text justify="Center">{`Button ${i + 1}`}</Text>
          </Node>
        ))}
      </Node>
    </AbsoluteFill>
  );
}

registerRoot(() => <Composition id="main" component={Shot} durationInFrames={11} fps={60} width={1280} height={720} />);
