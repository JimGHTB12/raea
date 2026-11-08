// Bevy v0.20.0 examples/ui/scroll_and_overflow/scroll.rs in raea's TSX: a column spread down the window; at the top a
// heading and a horizontally scrolling 0.1 grey list 80% wide (10 px margin) of a hundred "Item i" texts at least
// 200 px wide; below, a row spread across of four 200 px columns, each a heading over a list half the window high:
// one scrolling on y with a 20 px scrollbar gutter (25 rows 21 px high), one scrolling both ways (25 rows of ten),
// a 30-column grid of "|y,x|" cells (black; the first row and column blue headers kept in place by IgnoreScroll, the
// corner red) and five nested lists, each scrolling on y, 40% to 200% of their parent's height. Nothing scrolls until
// the mouse wheel turns. All in FiraSans-Bold. Bevy's Camera2d over the default clear colour is the layered
// <Canvas3D>. In Bevy's default 1280 x 720 window.
import type { ReactNode } from "react";
import { AbsoluteFill, Composition, registerRoot, staticFile } from "raea";
import { Canvas3D } from "raea/3d";
import { Node, Overflow, RepeatedGridTrack, Text, UiRect, percent, px } from "raea/ui";
import { Srgba } from "raea/color";

const FONT = staticFile("fonts/FiraSans-Bold.ttf");
const LINE_HEIGHT = 21;
const GREY = new Srgba(0.1, 0.1, 0.1);
const range = (n: number) => Array.from({ length: n }, (_, i) => i);

function Column({ title, children }: { title: string; children: ReactNode }) {
  return (
    <Node flex_direction="Column" justify_content="Center" align_items="Center" width={px(200)}>
      <Text font={FONT} font_size={{ Px: 20 }}>{title}</Text>
      {children}
    </Node>
  );
}

function Shot() {
  return (
    <AbsoluteFill>
      <Canvas3D layered clearColor="rgb(43, 44, 47)" />
      <Node width={percent(100)} height={percent(100)} justify_content="SpaceBetween" flex_direction="Column">
        <Node width={percent(100)} flex_direction="Column">
          <Text font={FONT} font_size={{ Px: 20 }}>Horizontally Scrolling list (Ctrl + MouseWheel)</Text>
          <Node width={percent(80)} margin={UiRect.all(px(10))} flex_direction="Row" overflow={Overflow.scroll_x()} background_color={GREY}>
            {range(100).map((i) => <Text key={i} font={FONT} min_width={px(200)} align_content="Center">{`Item ${i}`}</Text>)}
          </Node>
        </Node>
        <Node width={percent(100)} height={percent(100)} flex_direction="Row" justify_content="SpaceBetween">
          <Column title="Vertically Scrolling List">
            <Node flex_direction="Column" align_self="Stretch" height={percent(50)} overflow={Overflow.scroll_y()} scrollbar_width={20} background_color={GREY}>
              {range(25).map((i) => (
                <Node key={i} min_height={px(LINE_HEIGHT)} max_height={px(LINE_HEIGHT)}>
                  <Text font={FONT}>{`Item ${i}`}</Text>
                </Node>
              ))}
            </Node>
          </Column>
          <Column title="Bidirectionally Scrolling List">
            <Node flex_direction="Column" align_self="Stretch" height={percent(50)} overflow={Overflow.scroll()} background_color={GREY}>
              {range(25).map((oi) => (
                <Node key={oi} flex_direction="Row">
                  {range(10).map((i) => <Text key={i} font={FONT}>{`Item ${oi * 10 + i}`}</Text>)}
                </Node>
              ))}
            </Node>
          </Column>
          <Column title="Bidirectionally Scrolling List With Sticky Nodes">
            <Node display="Grid" align_self="Stretch" height={percent(50)} overflow={Overflow.scroll()} grid_template_columns={[RepeatedGridTrack.auto(30)]}>
              {range(30).flatMap((y) => range(30).map((x) => {
                const [z, bg] = x === 0 && y === 0 ? [2, new Srgba(1, 0, 0)] : x === 0 || y === 0 ? [1, new Srgba(0, 0, 1)] : [0, new Srgba(0, 0, 0)];
                return <Text key={`${y},${x}`} font={FONT} linebreak="NoWrap" ignore_scroll={{ x: x === 0, y: y === 0 }} z_index={z} background_color={bg}>{`|${y},${x}|`}</Text>;
              }))}
            </Node>
          </Column>
          <Column title="Nested Scrolling Lists">
            <Node column_gap={px(20)} flex_direction="Row" align_self="Stretch" height={percent(50)} overflow={Overflow.scroll()} background_color={GREY}>
              {range(5).map((oi) => (
                <Node key={oi} flex_direction="Column" align_self="Stretch" height={percent((200 / 5) * (oi + 1))} overflow={Overflow.scroll_y()} background_color={new Srgba(0.05, 0.05, 0.05)}>
                  {range(20).map((i) => <Text key={i} font={FONT}>{`Item ${oi * 20 + i}`}</Text>)}
                </Node>
              ))}
            </Node>
          </Column>
        </Node>
      </Node>
    </AbsoluteFill>
  );
}

registerRoot(() => <Composition id="main" component={Shot} durationInFrames={11} fps={60} width={1280} height={720} />);
