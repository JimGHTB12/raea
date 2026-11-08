// Bevy v0.20.0 examples/ui/widgets/tab_navigation.rs in raea's TSX (docs/control.feature FC0a): a full-window column,
// centred both ways with 6 px rows, of four labelled TabGroups ("TabGroup 0", "TabGroup 2", "TabGroup 1", "Modal
// TabGroup", in Bevy's default font), each a row (6 px gaps, 10 px below) of four 200 x 65 buttons with 5 px black
// borders on srgb(0.15, 0.15, 0.15), each "TabIndex i" in 20 px srgb(0.9, 0.9, 0.9) by its group's order (0 0 0 0,
// 3 2 1 0, 0 1 2 3, 0 3 1 2). Nothing has focus, so no button has focus_system's white outline; hover, presses and Tab
// change them only under input. Bevy's Camera2d over the default clear colour is the layered <Canvas3D>, in Bevy's
// default 1280 x 720 window.
import { AbsoluteFill, Composition, registerRoot } from "raea";
import { Canvas3D } from "raea/3d";
import { Node, Text, UiRect, percent, px } from "raea/ui";
import { Srgba } from "raea/color";

const GROUPS: [string, number[]][] = [["TabGroup 0", [0, 0, 0, 0]], ["TabGroup 2", [3, 2, 1, 0]], ["TabGroup 1", [0, 1, 2, 3]], ["Modal TabGroup", [0, 3, 1, 2]]];

function Shot() {
  return (
    <AbsoluteFill>
      <Canvas3D layered clearColor="rgb(43, 44, 47)" />
      <Node width={percent(100)} height={percent(100)} display="Flex" flex_direction="Column" align_items="Center" justify_content="Center" row_gap={px(6)}>
        {GROUPS.flatMap(([label, indices]) => [
          <Text key={`${label}-label`}>{label}</Text>,
          <Node key={label} display="Flex" flex_direction="Row" column_gap={px(6)} margin={UiRect.bottom(px(10))}>
            {indices.map((i, k) => (
              <Node key={k} width={px(200)} height={px(65)} border={UiRect.all(px(5))} justify_content="Center" align_items="Center" border_color={new Srgba(0, 0, 0)} background_color={new Srgba(0.15, 0.15, 0.15)}>
                <Text font_size={{ Px: 20 }} color={new Srgba(0.9, 0.9, 0.9)}>{`TabIndex ${i}`}</Text>
              </Node>
            ))}
          </Node>,
        ])}
      </Node>
    </AbsoluteFill>
  );
}

registerRoot(() => <Composition id="main" component={Shot} durationInFrames={11} fps={60} width={1280} height={720} />);
