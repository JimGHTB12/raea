// Bevy v0.20.0 examples/ui/scroll_and_overflow/overflow_debug.rs in raea's TSX: the instructions top left (12 px in,
// Bevy's default font) ending in a span of Overflow::clip()'s Debug, and centred a grid of 3 x 2 cells of 150 px, 80
// px apart, each a 0.25 grey node clipping its overflow around a centred node with a UiTransform: the bevy logo (100
// px high, absolute at -200, -50) in the top row, "Bevy" in 100 px FiraSans-Bold in the bottom; moved, scaled and
// turned per column. The animation starts paused (space plays it), so t stays 0: moved by (-50%, 0), scaled by (1.5,
// 1) and turned 45 radians. Bevy's Camera2d over the default clear colour is the layered <Canvas3D>. In Bevy's default
// 1280 x 720 window.
import type { ReactNode } from "react";
import { AbsoluteFill, Composition, registerRoot, staticFile } from "raea";
import { Canvas3D } from "raea/3d";
import { ImageNode, Node, RepeatedGridTrack, Rot2, Text, TextSpan, Val2, percent, px, type UiTransform } from "raea/ui";
import { Srgba } from "raea/color";

const f = Math.fround;
const T = 0;
const TAU = f(Math.PI * 2);
// each column's UpdateTransform at t (f32, as Bevy's ops)
const MOVE: UiTransform = { translation: Val2.percent(f(Math.sin(f(f(T * TAU) - f(Math.PI / 2))) * 50), f(-Math.cos(f(f(T * TAU) - f(Math.PI / 2))) * 50)) };
const SCALE: UiTransform = { scale: [f(1 + 0.5 * Math.max(f(Math.cos(f(T * TAU))), 0)), f(1 + 0.5 * Math.max(f(Math.cos(f(f(T * TAU) + f(Math.PI)))), 0))] };
const ROTATE: UiTransform = { rotation: Rot2.radians(f(f(Math.cos(f(T * TAU))) * 45)) };

function Container({ transform, children }: { transform: UiTransform; children: ReactNode }) {
  return (
    <Node width={percent(100)} height={percent(100)} align_items="Center" justify_content="Center" overflow={{ x: "Clip", y: "Clip" }} background_color={new Srgba(0.25, 0.25, 0.25)}>
      <Node align_items="Center" justify_content="Center" ui_transform={transform}>{children}</Node>
    </Node>
  );
}

const logo = <ImageNode image={staticFile("branding/bevy_logo_dark_big.png")} height={px(100)} position_type="Absolute" top={px(-50)} left={px(-200)} />;
const word = <Text font={staticFile("fonts/FiraSans-Bold.ttf")} font_size={{ Px: 100 }}>Bevy</Text>;

function Shot() {
  return (
    <AbsoluteFill>
      <Canvas3D layered clearColor="rgb(43, 44, 47)" />
      <Text position_type="Absolute" top={px(12)} left={px(12)}>
        {"Next Overflow Setting (O)\nNext Container Size (S)\nToggle Animation (space)\n\n"}
        <TextSpan>{"Overflow { x: Clip, y: Clip }"}</TextSpan>
      </Text>
      <Node width={percent(100)} height={percent(100)} justify_content="Center" align_items="Center">
        <Node display="Grid" grid_template_columns={[RepeatedGridTrack.px(3, 150)]} grid_template_rows={[RepeatedGridTrack.px(2, 150)]} row_gap={px(80)} column_gap={px(80)}>
          {[MOVE, SCALE, ROTATE].map((t, i) => <Container key={`i${i}`} transform={t}>{logo}</Container>)}
          {[MOVE, SCALE, ROTATE].map((t, i) => <Container key={`t${i}`} transform={t}>{word}</Container>)}
        </Node>
      </Node>
    </AbsoluteFill>
  );
}

registerRoot(() => <Composition id="main" component={Shot} durationInFrames={11} fps={60} width={1280} height={720} />);
