// Bevy v0.20.0 examples/ui/images/image_node_resizing.rs in raea's TSX: a white grid filling the window, its rows
// min-content and the rest: the black instructions (6 px padding, Bevy's default font) over a black column (10 px
// padding, spaced around) of two light-blue (tailwind BLUE_100) groups 75% wide and 30% high: four icon.png
// ImageNodes in NodeImageMode::Auto, then four in Stretch, 10%, 20%, 30% and 40% wide and the group's height. Bevy's
// GlobalUiDebugOptions outlines every node but the text's, each coloured by its entity's index: setup's Commands
// reserve them in spawn order from the camera's, 430 (DefaultPlugins' startup entities before it; the text's
// observer comes later, when the commands apply). The arrows resize the groups only while held. Bevy's Camera2d over
// the default clear colour is the layered <Canvas3D>. In Bevy's default 1280 x 720 window.
import type { ReactNode } from "react";
import { AbsoluteFill, Composition, registerRoot, staticFile } from "raea";
import { Canvas3D } from "raea/3d";
import { GlobalUiDebugOptions, GridPlacement, GridTrack, ImageNode, Node, Text, UiRect, percent, px } from "raea/ui";
import { Srgba } from "raea/color";

const ICON = staticFile("branding/icon.png");
// tailwind BLUE_100 (bevy_color's palettes::tailwind): #dbeafe
const BLUE_100 = new Srgba(0xdb / 255, 0xea / 255, 0xfe / 255);
const PREFIX = "Compare NodeImageMode(Auto, Stretch) press `Up`/`Down` to resize height, press `Left`/`Right` to resize width\n";
// each node's entity index: the camera's, then setup's spawns in order
const CAMERA = 430;
const at = (k: number) => CAMERA + k;

const Group = ({ index, children }: { index: number; children: ReactNode }) => (
  <Node debug_color_index={index} display="Flex" justify_content="Start" width={percent(75)} height={percent(30)} background_color={BLUE_100}>{children}</Node>
);

function Shot() {
  return (
    <AbsoluteFill>
      <Canvas3D layered clearColor="rgb(43, 44, 47)" />
      <GlobalUiDebugOptions enabled>
        <Node debug_color_index={at(1)} display="Grid" width={percent(100)} height={percent(100)} grid_template_rows={[GridTrack.min_content(), GridTrack.flex(1)]} background_color={new Srgba(1, 1, 1)}>
          <Text ui_debug={{ enabled: false }} grid_row={GridPlacement.span(1)} padding={UiRect.all(px(6))} color={new Srgba(0, 0, 0)}>{`${PREFIX}height : 30%, width : 75%`}</Text>
          <Node debug_color_index={at(3)} display="Flex" grid_row={GridPlacement.span(1)} flex_direction="Column" justify_content="SpaceAround" padding={UiRect.all(px(10))} background_color={new Srgba(0, 0, 0)}>
            <Group index={at(4)}>{[0, 1, 2, 3].map((i) => <ImageNode key={i} debug_color_index={at(5 + i)} image={ICON} />)}</Group>
            <Group index={at(9)}>{[10, 20, 30, 40].map((w, i) => <ImageNode key={w} debug_color_index={at(10 + i)} image={ICON} image_mode="Stretch" height={percent(100)} width={percent(w)} />)}</Group>
          </Node>
        </Node>
      </GlobalUiDebugOptions>
    </AbsoluteFill>
  );
}

registerRoot(() => <Composition id="main" component={Shot} durationInFrames={11} fps={60} width={1280} height={720} />);
