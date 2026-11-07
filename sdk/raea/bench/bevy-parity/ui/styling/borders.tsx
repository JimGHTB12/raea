// Bevy v0.20.0 examples/ui/styling/borders.rs in raea's TSX (docs/control.feature FC0a): on a 0.25 grey panel (25 px
// margins, stretched over the window), "Borders" over sixteen 50 x 50 maroon nodes, each with a different set of
// border sides (10 or 20 px), its sides red, blue, yellow and green, a 6 px white outline 6 px out and a 10 px yellow
// square in its centre, labelled in 9 px; then "Borders Rounded": the same sixteen with each corner between two
// bordered sides as round as it can be (BorderRadius::px of f32::MAX), the square round, and four elliptical corners.
// Bevy's Camera2d over the default clear colour is the layered <Canvas3D>. In Bevy's default 1280 x 720 window.
import { AbsoluteFill, Composition, registerRoot } from "raea";
import { Canvas3D } from "raea/3d";
import { BorderRadius, CornerRadius, Node, Text, UiRect, percent, px, type Val } from "raea/ui";
import { Srgba } from "raea/color";

const MAROON = new Srgba(0.5019608, 0, 0);
const YELLOW = new Srgba(1, 1, 0);
const SIDES = { top: new Srgba(1, 0, 0), bottom: YELLOW, left: new Srgba(0, 0.5019608, 0), right: new Srgba(0, 0, 1) };
const OUTLINE = { width: px(6), offset: px(6), color: new Srgba(1, 1, 1) };
const F32_MAX = 3.4028234663852886e38;

const LABELS = ["None", "All", "Left", "Right", "Top", "Bottom", "Horizontal", "Vertical", "Top Left", "Bottom Left", "Top Right", "Bottom Right", "Top Bottom Right", "Top Bottom Left", "Top Left Right", "Bottom Left Right"];
const side = (left = 0, right = 0, top = 0, bottom = 0): UiRect => UiRect.new(px(left), px(right), px(top), px(bottom));
const BORDERS: UiRect[] = [
  side(), UiRect.all(px(10)), UiRect.left(px(10)), UiRect.right(px(10)), UiRect.top(px(10)), UiRect.bottom(px(10)), UiRect.horizontal(px(10)), UiRect.vertical(px(10)),
  side(20, 0, 10), side(10, 0, 0, 20), side(0, 20, 10), side(0, 10, 0, 10), side(0, 10, 20, 10), side(10, 0, 10, 10), side(20, 10, 10), side(10, 10, 0, 20),
];
// a corner between two bordered sides is as round as it can be, the rest square
const both = (x: Val, y: Val) => (x !== "0px" && y !== "0px" ? F32_MAX : 0);
const ROUNDED = BORDERS.map((b) => BorderRadius.px(both(b.left, b.top), both(b.right, b.top), both(b.right, b.bottom), both(b.left, b.bottom)));
const ELLIPSES: [string, BorderRadius][] = [
  ["Ellipse Wide", BorderRadius.all(CornerRadius.new(px(25), px(8)))],
  ["Ellipse Tall", BorderRadius.all(CornerRadius.new(px(8), px(25)))],
  ["Ellipse Mixed", BorderRadius.new(CornerRadius.new(px(25), px(12)), CornerRadius.new(px(12), px(25)), CornerRadius.new(px(25), px(12)), CornerRadius.new(px(12), px(25)))],
  ["Ellipse Percent", BorderRadius.new(CornerRadius.new(percent(50), percent(20)), CornerRadius.new(percent(20), percent(50)), CornerRadius.new(percent(50), percent(20)), CornerRadius.new(percent(20), percent(50)))],
];

type Example = { label: string; border: UiRect; radius?: BorderRadius };

function Example({ label, border, radius }: Example) {
  return (
    <Node flex_direction="Column" align_items="Center">
      <Node width={px(50)} height={px(50)} border={border} margin={UiRect.all(px(20))} align_items="Center" justify_content="Center" border_radius={radius} background_color={MAROON} border_color={SIDES} outline={OUTLINE}>
        <Node width={px(10)} height={px(10)} border_radius={radius && BorderRadius.MAX} background_color={YELLOW} />
      </Node>
      <Text font_size={9}>{label}</Text>
    </Node>
  );
}

function Label({ text }: { text: string }) {
  return (
    <Node margin={UiRect.all(px(25))}>
      <Text font_size={20}>{text}</Text>
    </Node>
  );
}

const PLAIN: Example[] = LABELS.map((label, i) => ({ label, border: BORDERS[i] }));
const ROUND: Example[] = [...LABELS.map((label, i) => ({ label, border: BORDERS[i], radius: ROUNDED[i] })), ...ELLIPSES.map(([label, radius]) => ({ label, border: UiRect.all(px(10)), radius }))];

function Shot() {
  return (
    <AbsoluteFill>
      <Canvas3D layered clearColor="rgb(43, 44, 47)" />
      <Node margin={UiRect.all(px(25))} flex_direction="Column" align_self="Stretch" justify_self="Stretch" background_color={new Srgba(0.25, 0.25, 0.25)}>
        <Label text="Borders" />
        <Node margin={UiRect.all(px(25))} flex_wrap="Wrap">{PLAIN.map((e) => <Example key={e.label} {...e} />)}</Node>
        <Label text="Borders Rounded" />
        <Node margin={UiRect.all(px(25))} flex_wrap="Wrap">{ROUND.map((e) => <Example key={e.label} {...e} />)}</Node>
      </Node>
    </AbsoluteFill>
  );
}

registerRoot(() => <Composition id="main" component={Shot} durationInFrames={11} fps={60} width={1280} height={720} />);
