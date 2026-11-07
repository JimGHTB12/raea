// Bevy v0.20.0 examples/ui/images/ui_texture_slice.rs in raea's TSX (docs/control.feature FC0a): a full-window node
// centring three buttons, 150 x 150, 300 x 150 and 150 x 300 with 20 px margins, each an ImageNode of
// panel-border-010.png sliced by a TextureSlicer (22 px borders, sides and centre stretched, max_corner_scale 1) and
// holding "Button" in FiraSans-Bold 33 px, 0.9 grey. Their hover and press styles see no pointer here. Bevy's
// Camera2d over the default clear colour is the layered <Canvas3D>. In Bevy's default 1280 x 720 window.
import { AbsoluteFill, Composition, registerRoot, staticFile } from "raea";
import { Canvas3D } from "raea/3d";
import { ImageNode, Node, Text, UiRect, percent, px, type TextureSlicer } from "raea/ui";
import { Srgba } from "raea/color";

const IMAGE = staticFile("textures/fantasy_ui_borders/panel-border-010.png");
const FONT = staticFile("fonts/FiraSans-Bold.ttf");
const SLICER: TextureSlicer = { border: { left: 22, right: 22, top: 22, bottom: 22 }, center_scale_mode: "Stretch", sides_scale_mode: "Stretch", max_corner_scale: 1 };
const SIZES = [[150, 150], [300, 150], [150, 300]];

function Shot() {
  return (
    <AbsoluteFill>
      <Canvas3D layered clearColor="rgb(43, 44, 47)" />
      <Node width={percent(100)} height={percent(100)} align_items="Center" justify_content="Center">
        {SIZES.map(([w, h]) => (
          <ImageNode key={`${w}x${h}`} image={IMAGE} image_mode={{ Sliced: SLICER }} width={px(w)} height={px(h)} justify_content="Center" align_items="Center" margin={UiRect.all(px(20))}>
            <Text font={FONT} font_size={{ Px: 33 }} color={new Srgba(0.9, 0.9, 0.9)}>Button</Text>
          </ImageNode>
        ))}
      </Node>
    </AbsoluteFill>
  );
}

registerRoot(() => <Composition id="main" component={Shot} durationInFrames={11} fps={60} width={1280} height={720} />);
