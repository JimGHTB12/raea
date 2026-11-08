// Bevy v0.20.0 examples/ui/text/text_wrap_debug.rs in raea's TSX, at its defaults (no scale factor override, UiScale
// 1): on black, a column of four rows (each 50% tall, so two show), one per LineBreak (AnyCharacter, WordBoundary,
// WordOrCharacter, NoWrap); each row spaces around six columns 16% wide and 95% tall, clipped in x, one per
// JustifyContent, coloured (0.5, c, 1 - c) for c from 0.3 by 0.1; each column holds five 12 px FiraSans-Bold texts on
// reds from 0.8 down by 0.2: its JustifyContent and LineBreak by Debug name, two lines, a sentence and a long word.
// Bevy's Camera2d over the default clear colour is the layered <Canvas3D>. In Bevy's default 1280 x 720 window.
import { AbsoluteFill, Composition, registerRoot, staticFile } from "raea";
import { Canvas3D } from "raea/3d";
import { Node, Overflow, Text, percent } from "raea/ui";
import { Srgba } from "raea/color";

const FONT = staticFile("fonts/FiraSans-Bold.ttf");
const BREAKS = ["AnyCharacter", "WordBoundary", "WordOrCharacter", "NoWrap"] as const;
const JUSTIFY = ["Center", "FlexStart", "FlexEnd", "SpaceAround", "SpaceBetween", "SpaceEvenly"] as const;

function Shot() {
  return (
    <AbsoluteFill>
      <Canvas3D layered clearColor="rgb(43, 44, 47)" />
      <Node width={percent(100)} height={percent(100)} flex_direction="Column" background_color={new Srgba(0, 0, 0)}>
        {BREAKS.map((linebreak) => (
          <Node key={linebreak} flex_direction="Row" justify_content="SpaceAround" align_items="Center" width={percent(100)} height={percent(50)}>
            {JUSTIFY.map((justification, i) => {
              const c = 0.3 + i * 0.1;
              const messages = [`JustifyContent::${justification}`, `LineBreakOn::${linebreak}`, "Line 1\nLine 2", "Lorem ipsum dolor sit amet, consectetur adipiscing elit. Maecenas auctor, nunc ac faucibus fringilla.", "pneumonoultramicroscopicsilicovolcanoconiosis"];
              return (
                <Node key={justification} justify_content={justification} flex_direction="Column" width={percent(16)} height={percent(95)} overflow={Overflow.clip_x()} background_color={new Srgba(0.5, Math.fround(c), Math.fround(1 - c))}>
                  {messages.map((m, j) => (
                    <Text key={j} font={FONT} font_size={{ Px: 12 }} justify="Left" linebreak={linebreak} background_color={new Srgba(Math.fround(0.8 - j * 0.2), 0, 0)}>{m}</Text>
                  ))}
                </Node>
              );
            })}
          </Node>
        ))}
      </Node>
    </AbsoluteFill>
  );
}

registerRoot(() => <Composition id="main" component={Shot} durationInFrames={11} fps={60} width={1280} height={720} />);
