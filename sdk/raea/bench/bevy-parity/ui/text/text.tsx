// Bevy v0.20.0 examples/ui/text/text.rs in raea's TSX: "hello\nbevy!" underlined, centred, in FiraSans-Bold at 20vh
// with the default TextShadow, 5 px in from the bottom right, its colour a sine of Bevy's elapsed time per channel;
// "FPS: " (42 px FiraSans-Bold) with the smoothed frame rate in a 33 px gold FiraMono-Medium span; at the top right
// (5 px in, a 12 px margin) "Opentype features:" in 32 px EB Garamond and seven 24 px rows, each a title and a sample
// with one OpenType feature on (small caps, ligatures, fractions, superscript, subscript, oldstyle and lining figures);
// and the default font's sentence at the bottom left. The parity host steps Bevy by 1/60 s, so the smoothed rate is
// 60.00 (FPS below). Bevy's Camera2d over the default clear colour is the layered <Canvas3D>. In Bevy's
// default 1280 x 720 window.
import { AbsoluteFill, Composition, registerRoot, staticFile, useCurrentFrame } from "raea";
import { Canvas3D } from "raea/3d";
import { FontFeatureTag, Node, Text, TextShadow, TextSpan, UiRect, px } from "raea/ui";
import { Srgba } from "raea/color";

const FIRA_BOLD = staticFile("fonts/FiraSans-Bold.ttf");
const GARAMOND = staticFile("fonts/EBGaramond12-Regular.otf");
// FrameTimeDiagnosticsPlugin's smoothed rate over 1/60 s steps
const FPS = "60.00";
const ROWS: [string, string, string][] = [
  ["Smallcaps: ", FontFeatureTag.SMALL_CAPS, "Hello World"],
  ["Ligatures: ", FontFeatureTag.STANDARD_LIGATURES, "fi fl ff ffi ffl"],
  ["Fractions: ", FontFeatureTag.FRACTIONS, "12/134"],
  ["Superscript: ", FontFeatureTag.SUPERSCRIPT, "Up here!"],
  ["Subscript: ", FontFeatureTag.SUBSCRIPT, "Down here!"],
  ["Oldstyle figures: ", FontFeatureTag.OLDSTYLE_FIGURES, "1234567890"],
  ["Lining figures: ", FontFeatureTag.LINING_FIGURES, "1234567890"],
];

// Time::elapsed_secs at frame f: f steps of 1/60 s, summed in f32
function elapsedAt(f: number): number {
  let t = 0;
  for (let k = 1; k <= f; k++) t = Math.fround(t + Math.fround(16_666_667e-9));
  return t;
}

function Shot() {
  const t = elapsedAt(useCurrentFrame());
  const wave = (k: number) => Math.sin(k * t) / 2 + 0.5;
  return (
    <AbsoluteFill>
      <Canvas3D layered clearColor="rgb(43, 44, 47)" />
      <Text underline font={FIRA_BOLD} font_size={{ Vh: 20 }} shadow={TextShadow.default()} justify="Center" color={new Srgba(wave(1.25), wave(0.75), wave(0.5))} position_type="Absolute" bottom={px(5)} right={px(5)}>{"hello\nbevy!"}</Text>
      <Text font={FIRA_BOLD} font_size={{ Px: 42 }}>
        FPS: <TextSpan font={staticFile("fonts/FiraMono-Medium.ttf")} font_size={{ Px: 33 }} color={new Srgba(1, 215 / 255, 0)}>{FPS}</TextSpan>
      </Text>
      <Text margin={UiRect.all(px(12))} position_type="Absolute" top={px(5)} right={px(5)} font={GARAMOND} font_size={{ Px: 32 }}>
        {"Opentype features:\n"}
        {ROWS.flatMap(([title, feature, sample]) => [
          <TextSpan key={`${title}t`} font={GARAMOND} font_size={{ Px: 24 }}>{title}</TextSpan>,
          <TextSpan key={`${title}s`} font={GARAMOND} font_size={{ Px: 24 }} font_features={{ [feature]: 1 }}>{`${sample}\n`}</TextSpan>,
        ])}
      </Text>
      <Text position_type="Absolute" bottom={px(5)} left={px(15)}>From an &amp;str into a Text with the default font!</Text>
    </AbsoluteFill>
  );
}

registerRoot(() => <Composition id="main" component={Shot} durationInFrames={11} fps={60} width={1280} height={720} />);
