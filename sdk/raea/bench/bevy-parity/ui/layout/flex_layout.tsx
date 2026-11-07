// Bevy v0.20.0 examples/ui/layout/flex_layout.rs in raea's TSX (docs/control.feature FC0a): a full-window black column
// (padding and row gap 12 px, items centred) holding a legend, "AlignItems" in pink and "JustifyContent" in blue, and
// a grid of 5 rows (AlignItems Baseline, FlexStart, Center, FlexEnd, Stretch) by 6 columns (JustifyContent FlexStart,
// Center, FlexEnd, SpaceEvenly, SpaceAround, SpaceBetween), each cell a grey column node laid out by its pair and
// holding the pair's two labels. Every label is black FiraSans-Bold at Bevy's default size on its colour, padded 5 x 1.
// Bevy's Camera2d over the default clear colour is the layered <Canvas3D>. In Bevy's default 1280 x 720 window.
import { AbsoluteFill, Composition, registerRoot, staticFile } from "raea";
import { Canvas3D } from "raea/3d";
import { Node, Text, UiRect, percent, px, type UiRect as Rect } from "raea/ui";
import { Srgba } from "raea/color";

const ALIGN_ITEMS_COLOR = new Srgba(1, 0.066, 0.349), JUSTIFY_CONTENT_COLOR = new Srgba(0.102, 0.522, 1);
const BLACK = new Srgba(0, 0, 0), GREY = new Srgba(0.25, 0.25, 0.25);
const MARGIN = px(12);
const FONT = staticFile("fonts/FiraSans-Bold.ttf");
const JUSTIFICATIONS = ["FlexStart", "Center", "FlexEnd", "SpaceEvenly", "SpaceAround", "SpaceBetween"] as const;
const ALIGNMENTS = ["Baseline", "FlexStart", "Center", "FlexEnd", "Stretch"] as const;

function Label({ color, margin, text }: { color: Srgba; margin: Rect; text: string }) {
  return (
    <Node margin={margin} padding={UiRect.axes(px(5), px(1))} background_color={color}>
      <Text font={FONT} color={BLACK}>{text}</Text>
    </Node>
  );
}

function Shot() {
  return (
    <AbsoluteFill>
      <Canvas3D layered clearColor="rgb(43, 44, 47)" />
      <Node width={percent(100)} height={percent(100)} flex_direction="Column" align_items="Center" padding={UiRect.all(MARGIN)} row_gap={MARGIN} background_color={BLACK}>
        <Node flex_direction="Row">
          <Label color={ALIGN_ITEMS_COLOR} margin={UiRect.right(MARGIN)} text="AlignItems" />
          <Label color={JUSTIFY_CONTENT_COLOR} margin={UiRect.all(px(0))} text="JustifyContent" />
        </Node>
        <Node width={percent(100)} height={percent(100)} flex_direction="Column" row_gap={MARGIN}>
          {ALIGNMENTS.map((align_items) => (
            <Node key={align_items} width={percent(100)} height={percent(100)} flex_direction="Row" column_gap={MARGIN}>
              {JUSTIFICATIONS.map((justify_content) => (
                <Node key={justify_content} flex_direction="Column" align_items={align_items} justify_content={justify_content} width={percent(100)} height={percent(100)} background_color={GREY}>
                  <Label color={ALIGN_ITEMS_COLOR} margin={UiRect.top(px(0))} text={align_items} />
                  <Label color={JUSTIFY_CONTENT_COLOR} margin={UiRect.top(px(3))} text={justify_content} />
                </Node>
              ))}
            </Node>
          ))}
        </Node>
      </Node>
    </AbsoluteFill>
  );
}

registerRoot(() => <Composition id="main" component={Shot} durationInFrames={11} fps={60} width={1280} height={720} />);
