// Bevy v0.20.0 examples/ui/text/letter_spacing.rs in raea's TSX (docs/control.feature FC0a): a full-window column
// (5 vw by 10 vh of padding, 6 vh between rows, centred) of an underlined "HELLO" (6 vh, 2 vh of padding under it),
// then "letter spacing" justified left, centre and right (6 vh, LetterSpacing::Px(0), which the arrow keys change)
// each under its label (2 vh), all FiraSans-Bold, and two labels pinned 2 vh from the bottom corners. Bevy's Camera2d
// over the default clear colour is the layered <Canvas3D>. In Bevy's default 1280 x 720 window.
import { AbsoluteFill, Composition, registerRoot, staticFile } from "raea";
import { Canvas3D } from "raea/3d";
import { Node, Text, UiRect, percent, vh, vw } from "raea/ui";

const FONT = staticFile("fonts/FiraSans-Bold.ttf");

function Justified({ justify }: { justify: "Left" | "Center" | "Right" }) {
  return (
    <Node flex_direction="Column" width={percent(100)}>
      <Text font={FONT} font_size={{ Vh: 2 }}>{`Justify::${justify}`}</Text>
      <Text font={FONT} font_size={{ Vh: 6 }} justify={justify} width={percent(100)} letter_spacing={{ Px: 0 }}>letter spacing</Text>
    </Node>
  );
}

function Shot() {
  return (
    <AbsoluteFill>
      <Canvas3D layered clearColor="rgb(43, 44, 47)" />
      <Node width={percent(100)} height={percent(100)}>
        <Node width={percent(100)} height={percent(100)} align_items="Center" padding={UiRect.axes(vw(5), vh(10))} row_gap={vh(6)} flex_direction="Column">
          <Text font={FONT} font_size={{ Vh: 6 }} underline padding={UiRect.bottom(vh(2))}>HELLO</Text>
          <Justified justify="Left" />
          <Justified justify="Center" />
          <Justified justify="Right" />
        </Node>
        <Text font={FONT} font_size={{ Vh: 3 }} position_type="Absolute" bottom={vh(2)} left={vw(2)}>LetterSpacing::Px(0.0)</Text>
        <Text font={FONT} font_size={{ Vh: 2.5 }} position_type="Absolute" bottom={vh(2)} right={vw(2)}>← → to adjust   Space to toggle Px / Rem</Text>
      </Node>
    </AbsoluteFill>
  );
}

registerRoot(() => <Composition id="main" component={Shot} durationInFrames={11} fps={60} width={1280} height={720} />);
