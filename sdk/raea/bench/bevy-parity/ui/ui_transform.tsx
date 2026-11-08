// Bevy v0.20.0 examples/ui/ui_transform.rs in raea's TSX: on black, centred, a row (25 px gaps, spaced evenly) of a
// column of two 50 px white buttons ("<--" and "-", GlobalZIndex 1), a 300 px dark-grey target holding four 80 px
// white buttons round branding/icon.png (Top; Left turned a quarter anticlockwise; Right a quarter clockwise; Bottom
// half a turn), and a column of "-->" and "+". The buttons turn and scale the target only when pressed, and the arrow
// keys move it, so it holds as set up. Bevy's Camera2d over the default clear colour is the layered <Canvas3D>. In
// Bevy's default 1280 x 720 window.
import { AbsoluteFill, Composition, registerRoot, staticFile } from "raea";
import { Canvas3D } from "raea/3d";
import { ImageNode, Node, Rot2, Text, UiRect, UiTransform, percent, px } from "raea/ui";
import { Srgba } from "raea/color";

const [BLACK, WHITE] = [new Srgba(0, 0, 0), new Srgba(1, 1, 1)];

function Button({ size, label, turn }: { size: number; label: string; turn?: number }) {
  return (
    <Node width={px(size)} height={px(size)} align_items="Center" justify_content="Center" background_color={WHITE} ui_transform={turn === undefined ? undefined : UiTransform.from_rotation(Rot2.radians(turn))}>
      <Text color={BLACK}>{label}</Text>
    </Node>
  );
}

function Controls({ labels }: { labels: [string, string] }) {
  return (
    <Node flex_direction="Column" justify_content="Center" row_gap={px(10)} column_gap={px(10)} padding={UiRect.all(px(10))} background_color={BLACK} global_z_index={1}>
      <Button size={50} label={labels[0]} />
      <Button size={50} label={labels[1]} />
    </Node>
  );
}

function Shot() {
  return (
    <AbsoluteFill>
      <Canvas3D layered clearColor="rgb(43, 44, 47)" />
      <Node width={percent(100)} height={percent(100)} align_items="Center" justify_content="Center" background_color={BLACK}>
        <Node align_items="Center" justify_content="SpaceEvenly" column_gap={px(25)} row_gap={px(25)} background_color={BLACK}>
          <Controls labels={["<--", "-"]} />
          <Node flex_direction="Column" justify_content="SpaceBetween" align_items="Center" width={px(300)} height={px(300)} background_color={new Srgba(169 / 255, 169 / 255, 169 / 255)}>
            <Button size={80} label="Top" />
            <Node align_self="Stretch" justify_content="SpaceBetween" align_items="Center">
              <Button size={80} label="Left" turn={Math.fround(-Math.PI / 2)} />
              <ImageNode image={staticFile("branding/icon.png")} width={px(100)} height={px(100)} />
              <Button size={80} label="Right" turn={Math.fround(Math.PI / 2)} />
            </Node>
            <Button size={80} label="Bottom" turn={Math.fround(Math.PI)} />
          </Node>
          <Controls labels={["-->", "+"]} />
        </Node>
      </Node>
    </AbsoluteFill>
  );
}

registerRoot(() => <Composition id="main" component={Shot} durationInFrames={11} fps={60} width={1280} height={720} />);
