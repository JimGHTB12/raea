// Bevy v0.20.0 examples/ui/text/text_debug.rs in raea's TSX: two columns spread across the window (each growing, 15 x 5
// px margins, children spaced between), every text in FiraSans-Bold on maroon. Left, at the start: four lines broken by
// hand; a right-justified yellow paragraph at most 300 px wide; and a 300 px block of hand and wrapped breaks. Right, at
// the end: a centred pink 33 px paragraph at most 400 px wide; left-justified yellow and fully justified green-yellow
// 29 px paragraphs; and a 21 px Text whose string and spans change every frame to the frame rate (its own average over
// the frames so far, that average's deviation, and the FrameTimeDiagnosticsPlugin rate and frame time), with a
// zero-size and a negative-size span bevy_text leaves out. The parity host steps Bevy by 1/60 s, so each figure is
// worked out here as the example works it out. Bevy's Camera2d over the default clear colour is the layered
// <Canvas3D>. In Bevy's default 1280 x 720 window.
import { AbsoluteFill, Composition, registerRoot, staticFile, useCurrentFrame } from "raea";
import { Canvas3D } from "raea/3d";
import { Node, Text, TextSpan, UiRect, percent, px } from "raea/ui";
import { Srgba } from "raea/color";

const FONT = staticFile("fonts/FiraSans-Bold.ttf");
const css = (r: number, g: number, b: number) => new Srgba(r / 255, g / 255, b / 255);
const [MAROON, YELLOW, GREEN_YELLOW, RED, ORANGE_RED, LIME, BLUE] = [css(128, 0, 0), css(255, 255, 0), css(173, 255, 47), css(255, 0, 0), css(255, 69, 0), css(0, 255, 0), css(0, 0, 255)];
const STEP_NS = 16_666_667;
// the first update's Time::elapsed, in steps
const FIRST = 0;

// change_text_system's figures after the updates up to frame f: the average rate over its elapsed-time history and
// that average's (population) deviation over the frames, and the diagnostics' smoothed rate and frame time (in ms)
function figures(f: number) {
  const times: number[] = [];
  const fps: number[] = [];
  for (let k = FIRST; k <= f; k++) {
    times.unshift(k * STEP_NS);
    times.length = Math.min(times.length, 120);
    const span = (times[0] - times[times.length - 1]) / 1e9;
    fps.unshift(times.length / Math.max(span, 0.0001));
    fps.length = Math.min(fps.length, 120);
  }
  const mean = fps.reduce((a, b) => a + b, 0) / fps.length;
  const dev = Math.sqrt(fps.reduce((a, b) => a + (mean - b) * (mean - b), 0) / fps.length);
  return { avg: fps[0], dev, rate: 1e9 / STEP_NS, ms: STEP_NS / 1e6 };
}

function Paragraph({ children, ...p }: { children: string } & Parameters<typeof Text>[0]) {
  return <Text font={FONT} background_color={MAROON} {...p}>{children}</Text>;
}

function Shot() {
  const { avg, dev, rate, ms } = figures(useCurrentFrame());
  return (
    <AbsoluteFill>
      <Canvas3D layered clearColor="rgb(43, 44, 47)" />
      <Node width={percent(100)} height={percent(100)} justify_content="SpaceBetween">
        <Node flex_direction="Column" justify_content="SpaceBetween" align_items="Start" flex_grow={1} margin={UiRect.axes(px(15), px(5))}>
          <Paragraph font_size={{ Px: 25 }}>{"This is\ntext with\nline breaks\nin the top left."}</Paragraph>
          <Paragraph font_size={{ Px: 25 }} color={YELLOW} justify="Right" max_width={px(300)}>This text is right-justified. The `Justify` component controls the horizontal alignment of the lines of multi-line text relative to each other, and does not affect the text node's position in the UI layout.</Paragraph>
          <Paragraph font_size={{ Px: 25 }} max_width={px(300)}>{"This\ntext has\nline breaks and also a set width in the bottom left."}</Paragraph>
        </Node>
        <Node flex_direction="Column" justify_content="SpaceBetween" align_items="End" flex_grow={1} margin={UiRect.axes(px(15), px(5))}>
          <Paragraph font_size={{ Px: 33 }} color={new Srgba(0.8, 0.2, 0.7)} justify="Center" max_width={px(400)}>This text is very long, has a limited width, is center-justified, is positioned in the top right and is also colored pink.</Paragraph>
          <Paragraph font_size={{ Px: 29 }} color={YELLOW} justify="Left" max_width={px(300)}>This text is left-justified and is vertically positioned to distribute the empty space equally above and below it.</Paragraph>
          <Paragraph font_size={{ Px: 29 }} color={GREEN_YELLOW} justify="Justified" max_width={px(300)}>This text is fully justified and is positioned in the same way.</Paragraph>
          <Text font={FONT} font_size={{ Px: 21 }} background_color={MAROON}>
            {`${avg.toFixed(1)} avg fps, ${dev.toFixed(1)} frametime variance`}
            <TextSpan font={FONT} font_size={{ Px: 21 }}>{`\nThis text changes in the bottom right - ${rate.toFixed(1)} fps, ${ms.toFixed(3)} ms/frame`}</TextSpan>
            <TextSpan font={FONT} font_size={{ Px: 0 }} color={BLUE}> this text has zero font size</TextSpan>
            <TextSpan font={FONT} font_size={{ Px: 21 }} color={RED}>{"\nThis text changes in the bottom right - "}</TextSpan>
            <TextSpan font={FONT} font_size={{ Px: 21 }} color={ORANGE_RED}>{rate.toFixed(1)}</TextSpan>
            <TextSpan font={FONT} font_size={{ Px: 10 }} color={YELLOW}> fps, </TextSpan>
            <TextSpan font={FONT} font_size={{ Px: 21 }} color={LIME}>{ms.toFixed(3)}</TextSpan>
            <TextSpan font={FONT} font_size={{ Px: 42 }} color={BLUE}> ms/frame</TextSpan>
            <TextSpan font={FONT} font_size={{ Px: -42 }} color={BLUE}> this text has negative font size</TextSpan>
          </Text>
        </Node>
      </Node>
    </AbsoluteFill>
  );
}

registerRoot(() => <Composition id="main" component={Shot} durationInFrames={11} fps={60} width={1280} height={720} />);
