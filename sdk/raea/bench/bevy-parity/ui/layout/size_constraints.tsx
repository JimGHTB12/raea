// Bevy v0.20.0 examples/ui/layout/size_constraints.rs in raea's TSX: on black, centred, a column of "Size Constraints
// Example" (33 px FiraSans-Bold, 0.9 grey), a yellow bar (padding 10) round a black 100 px track (padding 4) holding the
// white Bar node, and a yellow panel (padding 10, 50 px down) of four black radio groups (min_size, flex_basis, size,
// max_size): a 200 px label, then buttons Auto (checked: antique-white border, white inside, black text) and 0% to
// 125% (black border, navy inside, half-grey text), each 100 px wide in a 2 px border. The radio buttons are headless
// (bevy_ui_widgets' RadioButton draws nothing), so they are their Nodes. Bevy's Camera2d over the default clear colour
// is the layered <Canvas3D>. In Bevy's default 1280 x 720 window.
import { AbsoluteFill, Composition, registerRoot, staticFile } from "raea";
import { Canvas3D } from "raea/3d";
import { Node, Text, UiRect, percent, px } from "raea/ui";
import { Srgba } from "raea/color";

const FONT = staticFile("fonts/FiraSans-Bold.ttf");
const SIZE = { Px: 33 };
const [BLACK, WHITE, YELLOW] = [new Srgba(0, 0, 0), new Srgba(1, 1, 1), new Srgba(1, 1, 0)];
const ANTIQUE_WHITE = new Srgba(250 / 255, 235 / 255, 215 / 255);
const NAVY = new Srgba(0, 0, 128 / 255);
const GREY = new Srgba(0.5, 0.5, 0.5);
const VALUES = ["Auto", "0%", "25%", "50%", "75%", "100%", "125%"];

function Radio({ label, active }: { label: string; active: boolean }) {
  return (
    <Node align_items="Center" justify_content="Center" border={UiRect.all(px(2))} margin={UiRect.horizontal(px(2))} border_color={active ? ANTIQUE_WHITE : BLACK}>
      <Node width={px(100)} justify_content="Center" background_color={active ? WHITE : NAVY}>
        <Text font={FONT} font_size={SIZE} color={active ? BLACK : GREY} justify="Center">{label}</Text>
      </Node>
    </Node>
  );
}

function Group({ label }: { label: string }) {
  return (
    <Node flex_direction="Column" padding={UiRect.all(px(2))} align_items="Stretch" background_color={BLACK}>
      <Node flex_direction="Row" justify_content="End" padding={UiRect.all(px(2))}>
        <Node min_width={px(200)} max_width={px(200)} justify_content="Center" align_items="Center">
          <Text font={FONT} font_size={SIZE} color={new Srgba(0.9, 0.9, 0.9)}>{label}</Text>
        </Node>
        <Node>
          {VALUES.map((v, i) => <Radio key={v} label={v} active={i === 0} />)}
        </Node>
      </Node>
    </Node>
  );
}

function Shot() {
  return (
    <AbsoluteFill>
      <Canvas3D layered clearColor="rgb(43, 44, 47)" />
      <Node width={percent(100)} height={percent(100)} justify_content="Center" align_items="Center" background_color={BLACK}>
        <Node flex_direction="Column" align_items="Center" justify_content="Center">
          <Text font={FONT} font_size={SIZE} color={new Srgba(0.9, 0.9, 0.9)} margin={UiRect.bottom(px(25))}>Size Constraints Example</Text>
          <Node flex_basis={percent(100)} align_self="Stretch" padding={UiRect.all(px(10))} background_color={YELLOW}>
            <Node align_items="Stretch" width={percent(100)} height={px(100)} padding={UiRect.all(px(4))} background_color={BLACK}>
              <Node background_color={WHITE} />
            </Node>
          </Node>
          <Node flex_direction="Column" align_items="Stretch" padding={UiRect.all(px(10))} margin={UiRect.top(px(50))} background_color={YELLOW}>
            {["min_size", "flex_basis", "size", "max_size"].map((l) => <Group key={l} label={l} />)}
          </Node>
        </Node>
      </Node>
    </AbsoluteFill>
  );
}

registerRoot(() => <Composition id="main" component={Shot} durationInFrames={11} fps={60} width={1280} height={720} />);
