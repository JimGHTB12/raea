// Bevy v0.20.0 examples/ui/text/font_weights.rs in raea's TSX (docs/control.feature FC0a): a column centred in the
// window (align_self and justify_self Center in its root's viewport), "Font Weights" underlined over a padded column
// of "Weight N" lines, 8 px apart, each in fonts/MonaSans-VariableFont.ttf at 32 px and its own FontWeight. Bevy's
// Camera2d over the default clear colour is the layered <Canvas3D>. In Bevy's default 1280 x 720 window.
import { AbsoluteFill, Composition, registerRoot, staticFile } from "raea";
import { Canvas3D } from "raea/3d";
import { Node, Text, UiRect, px } from "raea/ui";

const FONT = staticFile("fonts/MonaSans-VariableFont.ttf");
const WEIGHTS = [100, 134, 200, 300, 400, 500, 600, 700, 800, 900, 950];

function Shot() {
  return (
    <AbsoluteFill>
      <Canvas3D layered clearColor="rgb(43, 44, 47)" />
      <Node flex_direction="Column" align_self="Center" justify_self="Center" align_items="Center">
        <Text font={FONT} font_size={{ Px: 32 }} underline>Font Weights</Text>
        <Node flex_direction="Column" padding={UiRect.all(px(8))} row_gap={px(8)}>
          {WEIGHTS.map((w) => <Text key={w} font={FONT} font_size={{ Px: 32 }} weight={w}>{`Weight ${w}`}</Text>)}
        </Node>
      </Node>
    </AbsoluteFill>
  );
}

registerRoot(() => <Composition id="main" component={Shot} durationInFrames={11} fps={60} width={1280} height={720} />);
