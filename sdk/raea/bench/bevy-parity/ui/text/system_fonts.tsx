// Bevy v0.20.0 examples/ui/text/system_fonts.rs in raea's TSX (docs/control.feature FC2b): the system's font families
// as Bevy's FontCx lists them (FontCx.family_names, sorted, each once), counted in a line over a scrolling column
// (6 px apart) of rows, each a two-column grid (1fr 1fr, 50 px apart, 6 px padding, srgb 0.2 0.2 0.25) of the family's
// name in that family and in Bevy's default font, neither wrapped; all in a full-window column (10 px apart, centred,
// srgb 0.1). Bevy's Camera2d over the default clear colour is the layered <Canvas3D>, in Bevy's default 1280 x 720
// window. (Its PointerScroll observer scrolls the list; nothing scrolls it in the frames compared.)
import { AbsoluteFill, Composition, registerRoot } from "raea";
import { Canvas3D } from "raea/3d";
import { Srgba } from "raea/color";
import { FontCx, FontSource, GridTrack, Node, Text, UiRect, percent, px } from "raea/ui";

function Shot() {
  const families = FontCx.family_names();
  return (
    <AbsoluteFill>
      <Canvas3D layered clearColor="rgb(43, 44, 47)" />
      <Node flex_direction="Column" width={percent(100)} height={percent(100)} align_items="Center" row_gap={px(10)} background_color={new Srgba(0.1, 0.1, 0.1)}>
        <Text>{`Total available fonts: ${families.length}`}</Text>
        <Node flex_direction="Column" row_gap={px(6)} overflow={{ x: "Visible", y: "Scroll" }} align_items="Stretch">
          {families.map((family) => (
            <Node key={family} display="Grid" grid_template_columns={[GridTrack.flex(1), GridTrack.flex(1)]} padding={UiRect.all(px(6))} column_gap={px(50)} background_color={new Srgba(0.2, 0.2, 0.25)}>
              <Text font={FontSource.family(family)} linebreak="NoWrap">{family}</Text>
              <Text linebreak="NoWrap">{family}</Text>
            </Node>
          ))}
        </Node>
      </Node>
    </AbsoluteFill>
  );
}

registerRoot(() => <Composition id="main" component={Shot} durationInFrames={11} fps={60} width={1280} height={720} />);
