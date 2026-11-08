// Bevy v0.20.0 examples/ui/layout/anchor_layout.rs in raea's TSX: a black 3 x 3 grid filling the window (12 px padding
// and gaps), each cell a 0.25 grey box holding one absolute pink label (block, 5 x 1 px padding) anchored by offsets
// and auto margins to a corner, an edge's middle or the centre, its two lines in black FiraSans-Bold saying how. Bevy's
// Camera2d over the default clear colour is the layered <Canvas3D>. In Bevy's default 1280 x 720 window.
import { AbsoluteFill, Composition, registerRoot, staticFile } from "raea";
import { Canvas3D } from "raea/3d";
import { GridPlacement, Node, RepeatedGridTrack, Text, UiRect, percent, px } from "raea/ui";
import { Srgba } from "raea/color";

const FONT = staticFile("fonts/FiraSans-Bold.ttf");
const MARGIN = px(12);
const AUTO = "auto" as const;
type Anchor = { left?: ReturnType<typeof px>; right?: ReturnType<typeof px>; top?: ReturnType<typeof px>; bottom?: ReturnType<typeof px>; margin?: UiRect };
const ROWS: [string, Anchor][] = [
  ["left: 10px\ntop: 10px", { left: px(10), top: px(10) }],
  ["center: 10px\ntop: 10px", { margin: UiRect.horizontal(AUTO), top: px(10) }],
  ["right: 10px\ntop: 10px", { right: px(10), top: px(10) }],
  ["left: 10px\ncenter: 10px", { left: px(10), margin: UiRect.vertical(AUTO) }],
  ["center: 10px\ncenter: 10px", { margin: UiRect.AUTO }],
  ["right: 10px\ncenter: 10px", { right: px(10), margin: UiRect.vertical(AUTO) }],
  ["left: 10px\nbottom: 10px", { left: px(10), bottom: px(10) }],
  ["center: 10px\nbottom: 10px", { margin: UiRect.horizontal(AUTO), bottom: px(10) }],
  ["right: 10px\nbottom: 10px", { right: px(10), bottom: px(10) }],
];

function Shot() {
  return (
    <AbsoluteFill>
      <Canvas3D layered clearColor="rgb(43, 44, 47)" />
      <Node width={percent(100)} height={percent(100)} padding={UiRect.all(MARGIN)} row_gap={MARGIN} column_gap={MARGIN} display="Grid" grid_template_columns={[RepeatedGridTrack.fr(3, 1)]} grid_template_rows={[RepeatedGridTrack.fr(3, 1)]} background_color={new Srgba(0, 0, 0)}>
        {ROWS.map(([label, anchor]) => (
          <Node key={label} grid_column={GridPlacement.span(1)} grid_row={GridPlacement.span(1)} background_color={new Srgba(0.25, 0.25, 0.25)}>
            <Node display="Block" padding={UiRect.axes(px(5), px(1))} position_type="Absolute" {...anchor} background_color={new Srgba(1, 0.066, 0.349)}>
              <Text font={FONT} color={new Srgba(0, 0, 0)}>{label}</Text>
            </Node>
          </Node>
        ))}
      </Node>
    </AbsoluteFill>
  );
}

registerRoot(() => <Composition id="main" component={Shot} durationInFrames={11} fps={60} width={1280} height={720} />);
