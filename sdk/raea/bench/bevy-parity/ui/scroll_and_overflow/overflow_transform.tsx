// Bevy v0.20.0 examples/ui/scroll_and_overflow/overflow_transform.rs in raea's TSX: three nested square nodes (400,
// 350, 300 px; 4 px borders of 0.7 white; each clipping its overflow) turning about their centres at their own speeds,
// centred in the window; inside the last, an absolute navy box turned 45 degrees and breathing in scale, 20% rounded,
// a light-blue 5 px border, a drop shadow 30 px down and right (blur 4) of the innermost green made darker, holding the
// 400 px wide bevy_logo_dark_big.png and "transform + overflow" in 34 px FiraSans-Bold. Each turn and the scale are of
// Bevy's elapsed time, its f32 sum of 1/60 s steps up to the frame. Bevy's Camera2d over the default clear colour is
// the layered <Canvas3D>. In Bevy's default 1280 x 720 window.
import type { ReactNode } from "react";
import { AbsoluteFill, Composition, registerRoot, staticFile, useCurrentFrame } from "raea";
import { Canvas3D } from "raea/3d";
import { BorderRadius, BoxShadow, ImageNode, Node, Rot2, Text, UiRect, UiTransform, percent, px } from "raea/ui";
import { Srgba } from "raea/color";

// Time::elapsed_secs at frame f: f steps of 1/60 s, summed in f32
function elapsedAt(f: number): number {
  let t = 0;
  for (let k = 1; k <= f; k++) t = Math.fround(t + Math.fround(16_666_667e-9));
  return t;
}

const GREEN = new Srgba(0.15, 0.3, 0.25);

function Layer({ size, rotation, speed, color, t, children }: { size: number; rotation: number; speed: number; color: Srgba; t: number; children?: ReactNode }) {
  const turn = Math.fround(rotation + Math.fround(t * speed));
  return (
    <Node width={px(size)} height={px(size)} overflow={{ x: "Clip", y: "Clip" }} align_items="Center" justify_content="Center" border={UiRect.all(px(4))} background_color={color} border_color={new Srgba(1, 1, 1, 0.7)} ui_transform={UiTransform.from_rotation(Rot2.radians(turn))}>
      {children}
    </Node>
  );
}

function Shot() {
  const t = elapsedAt(useCurrentFrame());
  const s = 1 + 0.75 * Math.sin(0.4 * t);
  return (
    <AbsoluteFill>
      <Canvas3D layered clearColor="rgb(43, 44, 47)" />
      <Node width={percent(100)} height={percent(100)} align_items="Center" justify_content="Center">
        <Layer size={400} rotation={0.35} speed={0.18} color={new Srgba(0.12, 0.17, 0.22)} t={t}>
          <Layer size={350} rotation={-0.5} speed={-0.4} color={new Srgba(0.24, 0.18, 0.32)} t={t}>
            <Layer size={300} rotation={0.65} speed={0.55} color={GREEN} t={t}>
              <Node flex_direction="Column" position_type="Absolute" margin={UiRect.AUTO} border_radius={BorderRadius.all(percent(20))} align_items="Center" justify_content="SpaceAround" border={UiRect.all(px(5))} padding={UiRect.all(px(5))}
                box_shadow={BoxShadow.new(GREEN.darker(0.05), px(30), px(30), px(0), px(4))} background_color={new Srgba(0, 0, 128 / 255)} border_color={new Srgba(173 / 255, 216 / 255, 230 / 255)}
                ui_transform={{ rotation: Rot2.degrees(45), scale: [s, s] }}>
                <ImageNode image={staticFile("branding/bevy_logo_dark_big.png")} width={px(400)} />
                <Text font={staticFile("fonts/FiraSans-Bold.ttf")} font_size={{ Px: 34 }} color={new Srgba(1, 1, 1)}>transform + overflow</Text>
              </Node>
            </Layer>
          </Layer>
        </Layer>
      </Node>
    </AbsoluteFill>
  );
}

registerRoot(() => <Composition id="main" component={Shot} durationInFrames={11} fps={60} width={1280} height={720} />);
