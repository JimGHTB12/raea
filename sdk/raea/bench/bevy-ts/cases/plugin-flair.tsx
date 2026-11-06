// gap: PL12
// bevy_flair through raea/plugins/flair (docs/plugins.feature PL12): a bevy_ui node with no size or colour of its own,
// styled by plugin-flair.css (a stylesheet beside the case, through flair's Styled) whose .box rule, matched by the
// node's ClassList ["box"], makes it a 32 px green square at (16, 16). Unstyled, the node has no size and the frame
// stays black.
// build: plugins
// frame: 1
// probe: 32 32 g>200 r<60 b<60
// probe: 4 4 r<30 g<30 b<30
import { AbsoluteFill, Composition, registerRoot } from "raea";
import { BevyComponent, Entity, ToneMapping } from "raea/3d";
import { Camera3d } from "raea/camera";
import { Transform } from "raea/transform";
import { ClassList, Styled } from "raea/plugins/flair";

const black = { Srgba: { red: 0, green: 0, blue: 0, alpha: 0 } };

function Shot() {
  return (
    <AbsoluteFill style={{ background: "#000" }}>
      <>
        <Camera3d transform={Transform.fromXyz(0, 0, 3).lookingAt([0, 0, 0])} projection={{ Perspective: { fov: 0.785398 } }}>
          <BevyComponent type="bevy_ui::ui_node::IsDefaultUiCamera" value={{}} />
        </Camera3d>
        <ToneMapping mode="none" />
        <Entity name="box">
          <BevyComponent type="bevy_ui::ui_node::Node" value={{}} />
          <BevyComponent type="bevy_ui::ui_node::BackgroundColor" value={black} />
          <Styled value={{ StyleSheet: "plugin-flair.css" }} />
          <ClassList value={["box"]} />
        </Entity>
      </>
    </AbsoluteFill>
  );
}
registerRoot(() => <Composition id="main" component={Shot} durationInFrames={2} fps={30} width={64} height={64} />);
