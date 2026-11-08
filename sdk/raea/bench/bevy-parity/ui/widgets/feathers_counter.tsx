// Bevy v0.20.0 examples/ui/widgets/feathers_counter.rs in raea's TSX (docs/control.feature FC0a): under feathers' dark
// UiTheme, a full-window node on WINDOW_BG, centred both ways, holding a row of a FeathersButton captioned "-1", the
// count "0" (a caption with 10 px margins each side, outside any widget, so in Bevy's default font, 20 px white) and
// a FeathersButton captioned "+1"; each button's caption is a ThemedText, so in feathers' Fira Sans 14 px,
// BUTTON_TEXT. The buttons change the count only under input. Bevy's Camera2d over the default clear colour is the
// layered <Canvas3D>, in Bevy's default 1280 x 720 window.
import { AbsoluteFill, Composition, registerRoot } from "raea";
import { Canvas3D } from "raea/3d";
import { Node, UiRect, percent, px } from "raea/ui";
import { FeathersButton, ThemeBackgroundColor, ThemedText, UiTheme, caption, create_dark_theme, tokens } from "raea/feathers";

function Shot() {
  return (
    <AbsoluteFill>
      <Canvas3D layered clearColor="rgb(43, 44, 47)" />
      <UiTheme theme={create_dark_theme()}>
        <ThemeBackgroundColor token={tokens.WINDOW_BG}>
          <Node width={percent(100)} height={percent(100)} align_items="Center" justify_content="Center">
            <Node align_items="Center" justify_content="Center">
              <FeathersButton>{caption("-1")}</FeathersButton>
              <ThemedText margin={UiRect.horizontal(px(10))}>0</ThemedText>
              <FeathersButton>{caption("+1")}</FeathersButton>
            </Node>
          </Node>
        </ThemeBackgroundColor>
      </UiTheme>
    </AbsoluteFill>
  );
}

registerRoot(() => <Composition id="main" component={Shot} durationInFrames={11} fps={60} width={1280} height={720} />);
