// Bevy v0.20.0 examples/ui/scroll_and_overflow/overflow.rs in raea's TSX (docs/control.feature FC0a): on antique
// white, four columns 25 px apart, each a grey label box with its Overflow's pretty Debug text over a 100 x 100 grey
// node (5 px black border, 25 px padding top and left) whose ImageNode of branding/icon.png (at least 100 x 100)
// overflows it: visible, clipped in x, in y, and both (Bevy clips to the padding box, as CSS's overflow clip does).
// The hover outline is Color::NONE without a pointer. Bevy's Camera2d over the default clear colour is the layered
// <Canvas3D>. In Bevy's default 1280 x 720 window.
import { AbsoluteFill, Composition, registerRoot, staticFile } from "raea";
import { Canvas3D } from "raea/3d";
import { ImageNode, Node, Overflow, Text, UiRect, percent, px } from "raea/ui";
import { Srgba } from "raea/color";

const ANTIQUE_WHITE = new Srgba(0.98, 0.922, 0.843);
const GRAY = new Srgba(0.5019608, 0.5019608, 0.5019608);
const OVERFLOWS = [Overflow.visible(), Overflow.clip_x(), Overflow.clip_y(), Overflow.clip()];
// format!("{overflow:#?}")
const label = (o: Overflow) => `Overflow {\n    x: ${o.x},\n    y: ${o.y},\n}`;

function Shot() {
  return (
    <AbsoluteFill>
      <Canvas3D layered clearColor="rgb(43, 44, 47)" />
      <Node width={percent(100)} height={percent(100)} align_items="Center" justify_content="Center" background_color={ANTIQUE_WHITE}>
        {OVERFLOWS.map((o, i) => (
          <Node key={i} flex_direction="Column" align_items="Center" margin={UiRect.horizontal(px(25))}>
            <Node padding={UiRect.all(px(10))} margin={UiRect.bottom(px(25))} background_color={new Srgba(0.25, 0.25, 0.25)}>
              <Text>{label(o)}</Text>
            </Node>
            <Node width={px(100)} height={px(100)} padding={UiRect.new(px(25), px(0), px(25), px(0))} border={UiRect.all(px(5))} overflow={o} border_color={new Srgba(0, 0, 0)} background_color={GRAY}>
              <ImageNode image={staticFile("branding/icon.png")} min_width={px(100)} min_height={px(100)} />
            </Node>
          </Node>
        ))}
      </Node>
    </AbsoluteFill>
  );
}

registerRoot(() => <Composition id="main" component={Shot} durationInFrames={11} fps={60} width={1280} height={720} />);
