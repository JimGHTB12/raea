// Bevy v0.20.0 examples/ui/text/font_atlas_debug.rs in raea's TSX (docs/control.feature FC0a): a yellow Text in
// FiraSans-Bold at 50 px at the window's bottom left, which every 0.05 s (each third frame at 1/60 s: the Timer's
// 50 ms reached at 50.000001) takes a character from the example's ChaCha8Rng (seed 19878367467713, rand's
// random::<u8>() as char: Latin-1, a control character too) when it hasn't it yet; and over the black clear colour at
// the top left, side by side, the pages of bevy_text's glyph atlas that text fills (FontAtlasSet, natively by
// bevy_text's own code), one more each frame while there are more than shown (the atlas_render_system, which runs a
// frame after the text first fills one). Bevy's Camera2d is the layered <Canvas3D>, in Bevy's default 1280 x 720 window.
import { AbsoluteFill, Composition, registerRoot, staticFile, useCurrentFrame } from "raea";
import { Canvas3D } from "raea/3d";
import { Srgba } from "raea/color";
import { FontAtlasSet, ImageNode, Node, Text, px } from "raea/ui";
import { ChaCha8Rng } from "../../../../../../bench/bevy-parity/lib/rand";

const FONT = staticFile("fonts/FiraSans-Bold.ttf");
const YELLOW = new Srgba(1, 1, 0);

/** the Text's string after `f` updates: the Timer finishes on every third */
function textAt(f: number): string {
  const rng = ChaCha8Rng.seed_from_u64(19878367467713n);
  let s = "a";
  for (let k = 3; k <= f; k += 3) {
    const c = String.fromCharCode(rng.next_u32() & 0xff);
    if (!s.includes(c)) s += c;
  }
  return s;
}

function Shot() {
  const f = useCurrentFrame();
  const text = textAt(f);
  const atlases = FontAtlasSet.atlases(FONT, 50, text);
  // the pages the atlas_render_system has spawned by now: one an update, from the update after the first text layout
  const shown = atlases.slice(0, Math.max(0, f - 1));
  return (
    <AbsoluteFill>
      <Canvas3D layered clearColor="rgb(0, 0, 0)" />
      <Node position_type="Absolute" bottom={px(0)}>
        <Text font={FONT} font_size={{ Px: 50 }} color={YELLOW}>{text}</Text>
      </Node>
      {shown.map((a, i) => <ImageNode key={i} image={a.texture} position_type="Absolute" top={px(0)} left={px(a.width * i)} />)}
    </AbsoluteFill>
  );
}

registerRoot(() => <Composition id="main" component={Shot} durationInFrames={11} fps={60} width={1280} height={720} />);
