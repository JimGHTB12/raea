// Bevy v0.20.0 examples/ui/images/image_node.rs in raea's TSX (docs/control.feature FC0a): a full-window node centring
// an ImageNode of branding/icon.png, 256 x 256 with a 5 px white border and 10 px of padding (the picture fills what's
// inside them). Bevy's Camera2d over the default clear colour is the layered <Canvas3D>. In Bevy's default
// 1280 x 720 window.
import { AbsoluteFill, Composition, registerRoot, staticFile } from "raea";
import { Canvas3D } from "raea/3d";
import { ImageNode, Node, UiRect, percent, px } from "raea/ui";
import { Srgba } from "raea/color";

function Shot() {
  return (
    <AbsoluteFill>
      <Canvas3D layered clearColor="rgb(43, 44, 47)" />
      <Node width={percent(100)} height={percent(100)} align_items="Center" justify_content="Center">
        <ImageNode image={staticFile("branding/icon.png")} border={UiRect.all(px(5))} padding={UiRect.all(px(10))} width={px(256)} height={px(256)} border_color={new Srgba(1, 1, 1)} />
      </Node>
    </AbsoluteFill>
  );
}

registerRoot(() => <Composition id="main" component={Shot} durationInFrames={11} fps={60} width={1280} height={720} />);
