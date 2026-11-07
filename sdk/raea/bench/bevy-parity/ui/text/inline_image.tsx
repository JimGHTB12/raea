// Bevy v0.20.0 examples/ui/text/inline_image.rs in raea's TSX (docs/control.feature FC0a): one Text in Bevy's
// default font at 40 px, centred in the window with a 25 px margin on a maroon background inside a 2 px white
// outline, whose spans run around two InlineImages: bevy_logo_dark.png 30 px tall tinted red, and bevy_bird_dark.png
// 30 px wide tinted yellow. Bevy's Camera2d over the default clear colour is the layered <Canvas3D>. In Bevy's
// default 1280 x 720 window.
import { AbsoluteFill, Composition, registerRoot, staticFile } from "raea";
import { Canvas3D } from "raea/3d";
import { InlineImage, Text, TextSpan, UiRect, px } from "raea/ui";
import { Srgba } from "raea/color";

function Shot() {
  return (
    <AbsoluteFill>
      <Canvas3D layered clearColor="rgb(43, 44, 47)" />
      <Text font_size={40} margin={UiRect.all(px(25))} justify_self="Center" align_self="Center" background_color={new Srgba(0.5019608, 0, 0)} outline={{ width: px(2) }}>
        [Text]
        <TextSpan font_size={40}>[span before image]</TextSpan>
        <InlineImage image={staticFile("branding/bevy_logo_dark.png")} color={new Srgba(1, 0, 0)} height={30} />
        <TextSpan font_size={40}>[span between images]</TextSpan>
        <InlineImage image={staticFile("branding/bevy_bird_dark.png")} color={new Srgba(1, 1, 0)} width={30} />
        <TextSpan font_size={40}>[span after image]</TextSpan>
      </Text>
    </AbsoluteFill>
  );
}

registerRoot(() => <Composition id="main" component={Shot} durationInFrames={11} fps={60} width={1280} height={720} />);
