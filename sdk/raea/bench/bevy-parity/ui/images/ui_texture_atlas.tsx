// Bevy v0.20.0 examples/ui/images/ui_texture_atlas.rs in raea's TSX (docs/control.feature FC0a): a full-window
// column, 40 px apart, of a 256 x 256 ImageNode of the first 24 x 24 cell of gabe-idle-run.png's 7 x 1 grid on
// antique white with an 8 px crimson outline, over "press space to advance frames" with "space" in yellow, 20 px,
// under ImagePlugin::default_nearest() (pixel art unblurred). Space isn't pressed, so the cell stays the first.
// Bevy's Camera2d over the default clear colour is the layered <Canvas3D>. In Bevy's default 1280 x 720 window.
import { AbsoluteFill, Composition, registerRoot, staticFile } from "raea";
import { Canvas3D } from "raea/3d";
import { ImageNode, ImagePlugin, Node, Text, TextSpan, TextureAtlasLayout, percent, px } from "raea/ui";
import { Srgba } from "raea/color";

const ANTIQUE_WHITE = new Srgba(0.98, 0.922, 0.843);
const CRIMSON = new Srgba(0.863, 0.078, 0.235);
const YELLOW = new Srgba(1, 1, 0);
const LAYOUT = TextureAtlasLayout.from_grid([24, 24], 7, 1);

function Shot() {
  return (
    <AbsoluteFill>
      <Canvas3D layered clearColor="rgb(43, 44, 47)" />
      <ImagePlugin default_sampler="Nearest">
      <Node width={percent(100)} height={percent(100)} flex_direction="Column" justify_content="Center" align_items="Center" row_gap={px(40)}>
        <ImageNode image={staticFile("textures/rpg/chars/gabe/gabe-idle-run.png")} texture_atlas={{ layout: LAYOUT, index: 0 }} width={px(256)} height={px(256)} background_color={ANTIQUE_WHITE} outline={{ width: px(8), offset: px(0), color: CRIMSON }} />
        <Text font_size={20}>
          {"press "}
          <TextSpan color={YELLOW} font_size={20}>space</TextSpan>
          <TextSpan font_size={20}> to advance frames</TextSpan>
        </Text>
      </Node>
      </ImagePlugin>
    </AbsoluteFill>
  );
}

registerRoot(() => <Composition id="main" component={Shot} durationInFrames={11} fps={60} width={1280} height={720} />);
