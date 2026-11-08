// Bevy v0.20.0 examples/2d/sprite_scale.rs in raea's TSX (docs/control.feature FC0a): sprites of slice_square_2.png,
// Bevy's banner and gabe's 7-cell run sheet at custom sizes, each in a SpriteImageMode (Auto stretched, or Scale's
// FillCenter/Start/End and FitCenter/Start/End), each labelled by a 15 px Text2d centred under it (Anchor::TOP_CENTER,
// 10 px below its bottom edge); gabe's cells stepping on a 0.1 s Repeating timer. Camera2d, Bevy's default clear
// colour and 1280 x 720 window.
import { AbsoluteFill, Composition, registerRoot, staticFile, useCurrentFrame } from "raea";
import { Canvas3D, Sprite } from "raea/3d";
import { Transform } from "raea/transform";
import { Text2d } from "raea/ui";

type Mode = "Auto" | "FillCenter" | "FillStart" | "FillEnd" | "FitCenter" | "FitStart" | "FitEnd";
// [width, height, label, x, y, picture, mode], in Bevy's order (the sprites, then the sheets)
const RECTS: [number, number, string, number, number, string, Mode][] = [
  [100, 225, "Stretched", -570, 230, "square", "Auto"],
  [100, 225, "Fill Center", -450, 230, "square", "FillCenter"],
  [100, 225, "Fill Start", -330, 230, "square", "FillStart"],
  [100, 225, "Fill End", -210, 230, "square", "FillEnd"],
  [300, 100, "Fill Start Horizontal", 10, 290, "square", "FillStart"],
  [300, 100, "Fill End Horizontal", 10, 155, "square", "FillEnd"],
  [200, 200, "Fill Center", 280, 230, "banner", "FillCenter"],
  [200, 100, "Fill Center", 500, 230, "square", "FillCenter"],
  [100, 100, "Stretched", -570, -40, "banner", "Auto"],
  [200, 200, "Fit Center", -400, -40, "banner", "FitCenter"],
  [200, 200, "Fit Start", -180, -40, "banner", "FitStart"],
  [200, 200, "Fit End", 40, -40, "banner", "FitEnd"],
  [100, 200, "Fit Center", 210, -40, "banner", "FitCenter"],
  [120, 50, "Stretched", -570, -200, "gabe", "Auto"],
  [120, 50, "Fill Center", -570, -300, "gabe", "FillCenter"],
  [120, 50, "Fill Start", -430, -200, "gabe", "FillStart"],
  [120, 50, "Fill End", -430, -300, "gabe", "FillEnd"],
  [50, 120, "Fill Center", -300, -250, "gabe", "FillCenter"],
  [50, 120, "Fill Start", -190, -250, "gabe", "FillStart"],
  [50, 120, "Fill End", -90, -250, "gabe", "FillEnd"],
  [120, 50, "Fit Center", 20, -200, "gabe", "FitCenter"],
  [120, 50, "Fit Start", 20, -300, "gabe", "FitStart"],
  [120, 50, "Fit End", 160, -200, "gabe", "FitEnd"],
];
const PICTURE: Record<string, string> = { square: "textures/slice_square_2.png", banner: "branding/banner.png", gabe: "textures/rpg/chars/gabe/gabe-idle-run.png" };
const DELTA = 16_666_667; // Bevy's 1/60 s Duration, in nanoseconds

/** animate_sprite up to frame f: a 0.1 s Repeating timer ticked by each delta (none on the first), each finish
 *  stepping the cell, 6 back to 0 */
function cellAt(f: number): number {
  let idx = 0, el = 0;
  for (let k = 1; k <= f; k++) {
    el += DELTA;
    if (el >= 100_000_000) {
      el -= 100_000_000;
      idx = idx === 6 ? 0 : idx + 1;
    }
  }
  return idx;
}

function Shot() {
  const cell = cellAt(useCurrentFrame());
  return (
    <AbsoluteFill>
      <Canvas3D layered clearColor="rgb(43, 44, 47)">
        {RECTS.map(([w, h, , x, y, pic, mode], i) => (
          <Sprite
            key={i}
            src={staticFile(PICTURE[pic])}
            {...(pic === "gabe" ? { grid: [7, 1] as [number, number], frame: cell } : {})}
            width={w}
            height={h}
            image_mode={mode === "Auto" ? "Auto" : { Scale: mode }}
            x={640 + x}
            y={360 - y}
          />
        ))}
      </Canvas3D>
      {RECTS.map(([, h, label, x, y], i) => (
        <Text2d key={i} transform={Transform.fromXyz(x, y - 0.5 * h - 10, 0)} anchor={[0, 0.5]} justify="Center" font_size={15}>
          {label}
        </Text2d>
      ))}
    </AbsoluteFill>
  );
}

registerRoot(() => <Composition id="main" component={Shot} durationInFrames={61} fps={60} width={1280} height={720} />);
