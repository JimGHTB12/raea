// Bevy v0.20.0 examples/ui/text/font_query.rs in raea's TSX: centred in the window, a column (16 px padding and gap)
// of "Font Weights, Widths & Styles" (32 px, underlined) over a row of three columns 32 px apart (8 px padding and
// gap), every line in MonaSans-VariableFont.ttf at the default 20 px: the nine weights 100 to 900, the nine FontWidths
// from ULTRA_CONDENSED to ULTRA_EXPANDED, and the styles Normal, Oblique and Italic. Bevy's Camera2d over the default
// clear colour is the layered <Canvas3D>. In Bevy's default 1280 x 720 window.
import type { ReactNode } from "react";
import { AbsoluteFill, Composition, registerRoot, staticFile } from "raea";
import { Canvas3D } from "raea/3d";
import { FontWidth, Node, Text, UiRect, px } from "raea/ui";

const FONT = staticFile("fonts/MonaSans-VariableFont.ttf");
const WEIGHTS: [number, string][] = [[100, "Thin"], [200, "Extra Light"], [300, "Light"], [400, "Normal"], [500, "Medium"], [600, "Semibold"], [700, "Bold"], [800, "Extra Bold"], [900, "Black"]];
const WIDTHS = ["ULTRA_CONDENSED", "EXTRA_CONDENSED", "CONDENSED", "SEMI_CONDENSED", "NORMAL", "SEMI_EXPANDED", "EXPANDED", "EXTRA_EXPANDED", "ULTRA_EXPANDED"] as const;
const STYLES = ["Normal", "Oblique", "Italic"] as const;

const Column = ({ children }: { children: ReactNode }) => <Node flex_direction="Column" padding={UiRect.all(px(8))} row_gap={px(8)}>{children}</Node>;

function Shot() {
  return (
    <AbsoluteFill>
      <Canvas3D layered clearColor="rgb(43, 44, 47)" />
      <Node flex_direction="Column" align_self="Center" justify_self="Center" align_items="Center" padding={UiRect.all(px(16))} row_gap={px(16)}>
        <Text font={FONT} font_size={{ Px: 32 }} underline>{"Font Weights, Widths & Styles"}</Text>
        <Node flex_direction="Row" column_gap={px(32)}>
          <Column>{WEIGHTS.map(([w, name]) => <Text key={w} font={FONT} weight={w}>{`Weight ${w} (${name})`}</Text>)}</Column>
          <Column>{WIDTHS.map((k) => <Text key={k} font={FONT} font_width={FontWidth[k]}>{`FontWidth::${k}`}</Text>)}</Column>
          <Column>{STYLES.map((s) => <Text key={s} font={FONT} style={s}>{`FontStyle::${s}`}</Text>)}</Column>
        </Node>
      </Node>
    </AbsoluteFill>
  );
}

registerRoot(() => <Composition id="main" component={Shot} durationInFrames={11} fps={60} width={1280} height={720} />);
