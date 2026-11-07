// Bevy v0.20.0 examples/ui/scroll_and_overflow/overflow_clip_margin.rs in raea's TSX (docs/control.feature FC0a): on a
// full-window css ANTIQUE_WHITE column (centred, row gap 40), four rows, one per OverflowClipMargin: border_box()
// with_margin(25), border_box(), padding_box() and content_box(). Each row is its margin's Debug text in a grey box,
// beside a 100 x 100 GRAY node (padding 20, black border 5, Overflow::clip()) clipping a LIGHT_CYAN 50 px node that
// holds a 100 px branding/icon.png, so each shows its clip box. Bevy's Camera2d over the default clear colour is the
// layered <Canvas3D>. In Bevy's default 1280 x 720 window.
import { AbsoluteFill, Composition, registerRoot, staticFile } from "raea";
import { Canvas3D } from "raea/3d";
import { ImageNode, Node, Overflow, OverflowClipMargin, Text, UiRect, percent, px } from "raea/ui";
import { Srgba } from "raea/color";

const ANTIQUE_WHITE = new Srgba(0.98, 0.922, 0.843), GRAY = new Srgba(0.5019608, 0.5019608, 0.5019608);
const LIGHT_CYAN = new Srgba(0.878, 1, 1), DARK = new Srgba(0.25, 0.25, 0.25), BLACK = new Srgba(0, 0, 0);
const ICON = staticFile("branding/icon.png");
const MARGINS = [OverflowClipMargin.border_box().with_margin(25), OverflowClipMargin.border_box(), OverflowClipMargin.padding_box(), OverflowClipMargin.content_box()];
// Rust's {:#?} of the margin
const debug = (m: { visual_box: string; margin: number }) => `OverflowClipMargin {\n    visual_box: ${m.visual_box},\n    margin: ${m.margin.toFixed(1)},\n}`;

function Shot() {
  return (
    <AbsoluteFill>
      <Canvas3D layered clearColor="rgb(43, 44, 47)" />
      <Node width={percent(100)} height={percent(100)} align_items="Center" justify_content="Center" row_gap={px(40)} flex_direction="Column" background_color={ANTIQUE_WHITE}>
        {MARGINS.map((m, i) => (
          <Node key={i} flex_direction="Row" column_gap={px(20)}>
            <Node padding={UiRect.all(px(10))} margin={UiRect.bottom(px(25))} background_color={DARK}>
              <Text>{debug(m)}</Text>
            </Node>
            <Node margin={UiRect.top(px(10))} width={px(100)} height={px(100)} padding={UiRect.all(px(20))} border={UiRect.all(px(5))} overflow={Overflow.clip()} overflow_clip_margin={{ visual_box: m.visual_box, margin: m.margin }} background_color={GRAY} border_color={BLACK}>
              <Node min_width={px(50)} min_height={px(50)} background_color={LIGHT_CYAN}>
                <ImageNode image={ICON} min_width={px(100)} min_height={px(100)} />
              </Node>
            </Node>
          </Node>
        ))}
      </Node>
    </AbsoluteFill>
  );
}

registerRoot(() => <Composition id="main" component={Shot} durationInFrames={11} fps={60} width={1280} height={720} />);
