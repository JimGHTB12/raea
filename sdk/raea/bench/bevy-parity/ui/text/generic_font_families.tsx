// Bevy v0.20.0 examples/ui/text/generic_font_families.rs in raea's TSX: a grid of three equal columns, centred in the
// window (margin auto), headed "Generic Font Families" (underlined, across all three); then for each generic family
// (sans serif, serif, fantasy, cursive, monospace) a row of three outlined 25 px texts: its description in that
// family in wheat, its FontSource's name in yellow, and the family the system's fonts resolve it to. Bevy's Camera2d
// over the default clear colour is the layered <Canvas3D>. In Bevy's default 1280 x 720 window.
import { AbsoluteFill, Composition, registerRoot } from "raea";
import { Canvas3D } from "raea/3d";
import { FontSource, GridPlacement, Node, RepeatedGridTrack, Text, UiRect, px } from "raea/ui";
import { Srgba } from "raea/color";

const FONT_SIZE = { Px: 25 };
const WHEAT = new Srgba(0.9607843, 0.87058824, 0.7019608);
const YELLOW = new Srgba(1, 1, 0);
const OUTLINE = { color: new Srgba(0.32156864, 0.32156864, 0.35686275), width: px(2), offset: px(4) };

// each generic family, its Debug name, and the family fontique resolves it to here (FontCx::get_family)
const FAMILIES: [FontSource, string, string, string][] = [
  [FontSource.sans_serif(), "generic sans serif font", "Generic(SansSerif)", "Helvetica"],
  [FontSource.serif(), "generic serif font", "Generic(Serif)", "Times"],
  [FontSource.fantasy(), "generic fantasy font", "Generic(Fantasy)", "Papyrus"],
  [FontSource.cursive(), "generic cursive font", "Generic(Cursive)", "Apple Chancery"],
  [FontSource.monospace(), "generic monospace font", "Generic(Monospace)", "Courier"],
];

function Shot() {
  return (
    <AbsoluteFill>
      <Canvas3D layered clearColor="rgb(43, 44, 47)" />
      <Node display="Grid" grid_template_columns={[RepeatedGridTrack.fr(3, 1)]} margin={UiRect.AUTO} row_gap={px(25)} column_gap={px(15)}>
        <Text justify_self="Center" grid_column={GridPlacement.span(3)} margin={UiRect.bottom(px(15))} font_size={FONT_SIZE} underline>Generic Font Families</Text>
        {FAMILIES.flatMap(([source, description, debug, family]) => [
          <Text key={`${debug}-a`} font={source} font_size={FONT_SIZE} color={WHEAT} justify="Center" outline={OUTLINE}>{description}</Text>,
          <Text key={`${debug}-b`} font_size={FONT_SIZE} color={YELLOW} justify="Center" outline={OUTLINE}>{`FontSource::${debug}`}</Text>,
          <Text key={`${debug}-c`} font_size={FONT_SIZE} justify="Center" outline={OUTLINE}>{family}</Text>,
        ])}
      </Node>
    </AbsoluteFill>
  );
}

registerRoot(() => <Composition id="main" component={Shot} durationInFrames={11} fps={60} width={1280} height={720} />);
