// gap: PL12
// bevy_hui through raea/plugins/hui (docs/plugins.feature PL12): an HtmlNode naming plugin-hui.html, a pseudo-HTML
// template beside the case, which hui builds into bevy_ui nodes: a blue node over the whole frame with a 32 px green
// one centred in it (the green is the template's "fill" property, so its default flows through), drawn by the scene's
// camera as its default UI camera. Without the plugin nothing draws over the black frame.
// build: plugins
// frame: 1
// probe: 32 32 g>200 r<60 b<60
// probe: 4 4 b>200 r<60 g<60
import { AbsoluteFill, Composition, registerRoot } from "raea";
import { BevyComponent, Canvas3D, Entity, ToneMapping } from "raea/3d";
import { Camera3d } from "raea/camera";
import { Transform } from "raea/transform";
import { HtmlNode } from "raea/plugins/hui";

function Shot() {
  return (
    <AbsoluteFill style={{ background: "#000" }}>
      <Canvas3D>
        <Camera3d transform={Transform.fromXyz(0, 0, 3).lookingAt([0, 0, 0])} projection={{ Perspective: { fov: 0.785398 } }}>
          <BevyComponent type="bevy_ui::ui_node::IsDefaultUiCamera" value={{}} />
        </Camera3d>
        <ToneMapping mode="none" />
        <Entity name="ui">
          <HtmlNode value="plugin-hui.html" />
        </Entity>
      </Canvas3D>
    </AbsoluteFill>
  );
}
registerRoot(() => <Composition id="main" component={Shot} durationInFrames={2} fps={30} width={64} height={64} />);
