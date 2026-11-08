// Bevy v0.20.0 examples/ui/navigation/directional_navigation_overrides.rs in raea's TSX (docs/control.feature FC0a): the
// three panels of directional_navigation (the instructions, "Focused: ..." and "Last Key: None"), then three pages,
// each a full-window root of its own: only page 1 is visible (the others are Visibility::Hidden). Page 1 is a grid of
// twelve absolute 140 x 100 buttons (tailwind BLUE_400, 4 px borders, 12 px corners), "Btn row-column" at columns 450,
// 650 and 850 and rows 80, 215, 350 and 485, and 140 px wide helper labels (12 px padding, 20 px text, justified
// left, centre or right) naming where each edge leads, over a 540 px footer. Btn 1-1 has AutoFocus: with
// InputFocusVisible its border is BLUE_50 and the focus panel reads "Focused: Btn 1-1". The arrows and Enter move and
// press only under input. Bevy's Camera2d over the default clear colour is the layered <Canvas3D>, in Bevy's default
// 1280 x 720 window.
import type { ReactNode } from "react";
import { AbsoluteFill, Composition, registerRoot } from "raea";
import { Canvas3D } from "raea/3d";
import { BorderRadius, Node, Text, UiRect, percent, px } from "raea/ui";
import { Srgba } from "raea/color";

// tailwind BLUE_400 and BLUE_50 (bevy_color's palettes::tailwind): page 1's
const NORMAL_BUTTON = new Srgba(0.3764706, 0.64705884, 0.98039216);
const FOCUSED_BORDER = new Srgba(0.9372549, 0.9647059, 1.0);
const COLUMNS = [450, 650, 850];
const ROWS = [80, 215, 350, 485];
const FOCUSED = "Btn 1-1";
const PAGE = 0;

function Panel({ top, bottom, color, children }: { top?: number; bottom?: number; color: Srgba; children: ReactNode }) {
  return (
    <Node position_type="Absolute" left={px(20)} top={top === undefined ? undefined : px(top)} bottom={bottom === undefined ? undefined : px(bottom)} width={px(280)} padding={UiRect.all(px(12))} border_radius={BorderRadius.all(px(8))} background_color={color}>
      {children}
    </Node>
  );
}

function Helper({ text, left, top, justify }: { text: string; left: number; top: number; justify: "Left" | "Center" | "Right" }) {
  return (
    <Node position_type="Absolute" left={px(left)} top={px(top)} width={px(140)} padding={UiRect.all(px(12))}>
      <Text font_size={{ Px: 20 }} justify={justify}>{text}</Text>
    </Node>
  );
}

function Button({ text, left, top }: { text: string; left: number; top: number }) {
  return (
    <Node position_type="Absolute" left={px(left)} top={px(top)} width={px(140)} height={px(100)} border={UiRect.all(px(4))} justify_content="Center" align_items="Center" border_radius={BorderRadius.all(px(12))} background_color={NORMAL_BUTTON} border_color={text === FOCUSED ? FOCUSED_BORDER : undefined}>
      <Text justify="Center">{text}</Text>
    </Node>
  );
}

function Shot() {
  const previous = PAGE === 0 ? 3 : PAGE;
  return (
    <AbsoluteFill>
      <Canvas3D layered clearColor="rgb(43, 44, 47)" />
      <Node width={percent(100)} height={percent(100)}>
        <Panel top={20} color={new Srgba(0.1, 0.1, 0.1, 0.8)}>
          <Text>{"Directional Navigation Overrides Demo\n\nUse arrow keys or D-pad to navigate.\nPress Enter or A button to interact.\n\nNavigation on each page is a combination of both automatic and manual navigation."}</Text>
        </Panel>
        <Panel bottom={80} color={new Srgba(0.1, 0.5, 0.1, 0.8)}>
          <Text font_size={{ Px: 20 }}>{`Focused: ${FOCUSED}`}</Text>
        </Panel>
        <Panel bottom={20} color={new Srgba(0.5, 0.1, 0.5, 0.8)}>
          <Text font_size={{ Px: 20 }}>Last Key: None</Text>
        </Panel>
      </Node>
      <Node width={percent(100)} height={percent(100)}>
        <Helper text={`Currently on Page ${PAGE + 1}`} left={650} top={10} justify="Center" />
        <Helper text={`Page ${previous} << `} left={310} top={120} justify="Right" />
        <Helper text={`>> Page ${((PAGE + 1) % 3) + 1}`} left={1000} top={525} justify="Left" />
        <Helper text="> Btn 2-1" left={1000} top={120} justify="Left" />
        <Helper text="> Btn 3-1" left={1000} top={255} justify="Left" />
        <Helper text="> Btn 4-1" left={1000} top={390} justify="Left" />
        <Helper text="Btn 1-3 < " left={310} top={255} justify="Right" />
        <Helper text="Btn 2-3 < " left={310} top={390} justify="Right" />
        <Helper text="Btn 3-3 < " left={310} top={525} justify="Right" />
        <Node position_type="Absolute" left={px(450)} top={px(600)} width={px(540)} padding={UiRect.all(px(12))}>
          <Text font_size={{ Px: 20 }}>Vertical movements disabled on each button, but you can still navigate between rows by going off the left or right sides.</Text>
        </Node>
        {ROWS.map((top, i) => COLUMNS.map((left, j) => <Button key={`${i}-${j}`} text={`Btn ${i + 1}-${j + 1}`} left={left} top={top} />))}
      </Node>
    </AbsoluteFill>
  );
}

registerRoot(() => <Composition id="main" component={Shot} durationInFrames={11} fps={60} width={1280} height={720} />);
