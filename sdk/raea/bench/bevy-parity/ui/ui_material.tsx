// Bevy v0.20.0 examples/ui/ui_material.rs in raea's TSX (docs/control.feature FC2a): a full-window node centring an
// absolute 452.5 x 181.5 px node (the banner's 905 x 363 at half scale) with a 20 px border and 20 px corners, drawn by
// the example's own shader, assets/shaders/custom_ui_material.wesl, as written: branding/banner.png times `color`, cut
// at `slider` across, inside a border of `border_color` that the shader rounds itself. Its DARK_BLUE outline, 2 px wide
// and 100 px out, is the page's. The animate system's values at Bevy's elapsed time (its f32 sum of 1/60 s steps):
// `color` hsl(60 t, 1, 0.5) and `border_color` hsl(60 t, 0.75, 0.75), both linear, and `slider` |t mod 4 - 2| / 2 in x (its other three stay 0.5).
// Bevy's Camera2d over the default clear colour is the layered <Canvas3D>, in Bevy's default 1280 x 720 window.
import { AbsoluteFill, Composition, registerRoot, staticFile, useCurrentFrame } from "raea";
import { Canvas3D } from "raea/3d";
import { Hsla, Srgba } from "raea/color";
import { BorderRadius, MaterialNode, Node, UiRect, percent, px } from "raea/ui";

// Time::elapsed_secs at frame f: f steps of 1/60 s, summed in f32
function elapsedAt(f: number): number {
  let t = 0;
  for (let k = 1; k <= f; k++) t = Math.fround(t + Math.fround(16_666_667e-9));
  return t;
}

/** Color::hsl(...).to_linear().to_vec4() */
function linear(h: number, s: number, l: number): number[] {
  const c = new Hsla(h, s, l).to_linear();
  return [c.red, c.green, c.blue, c.alpha];
}

// css DARK_BLUE #00008b
const DARK_BLUE = new Srgba(0, 0, 0x8b / 255);
const SCALE = 0.5;

function Shot() {
  const t = elapsedAt(useCurrentFrame());
  const hue = Math.fround(Math.fround(t * 60) % 360);
  const slider = Math.abs(Math.fround(t % 4) - 2) / 2;
  return (
    <AbsoluteFill>
      <Canvas3D layered clearColor="rgb(43, 44, 47)" />
      <Node width={percent(100)} height={percent(100)} align_items="Center" justify_content="Center">
        <MaterialNode position_type="Absolute" width={px(905 * SCALE)} height={px(363 * SCALE)} border={UiRect.all(px(20))} border_radius={BorderRadius.all(px(20))}
          outline={{ width: px(2), offset: px(100), color: DARK_BLUE }}
          fragment={staticFile("shaders/custom_ui_material.wesl")}
          uniforms={{ color: linear(hue, 1, 0.5), slider: [slider, 0.5, 0.5, 0.5], border_color: linear(hue, 0.75, 0.75) }}
          textures={{ material_color_texture: staticFile("branding/banner.png") }} />
      </Node>
    </AbsoluteFill>
  );
}

registerRoot(() => <Composition id="main" component={Shot} durationInFrames={11} fps={60} width={1280} height={720} />);
