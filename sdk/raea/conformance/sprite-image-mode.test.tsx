// <Sprite image_mode> (Bevy's SpriteImageMode, docs/control.feature FC0a: 2d/sprite_scale, 2d/sprite_slice): the
// reflected value reaches the sprite's row as given, and anything else is refused.
import { expect, test } from "bun:test";
import { Sprite } from "raea/3d";
import { renderFrame } from "../src/test";

const html = (el: React.ReactElement) => renderFrame(el, { frame: 0, width: 320, height: 240, fps: 30, durationInFrames: 30 });

test("Scale, Sliced and Auto reach the row", () => {
  expect(html(<Sprite src="a.png" image_mode={{ Scale: "FitEnd" }} />)).toContain("FitEnd");
  expect(html(<Sprite src="a.png" image_mode={{ Sliced: { border: 80, center_scale_mode: { Tile: { stretch_value: 0.5 } }, max_corner_scale: 0.2 } }} />)).toContain("stretch_value");
  expect(() => html(<Sprite src="a.png" image_mode="Auto" />)).not.toThrow();
});

test("refuses an unknown mode, a bad slicer and tileX beside it", () => {
  expect(() => html(<Sprite src="a.png" image_mode={{ Scale: "Crop" } as never} />)).toThrow(/SpriteImageMode/);
  expect(() => html(<Sprite src="a.png" image_mode={{ Sliced: { border: "wide" } } as never} />)).toThrow(/SpriteImageMode/);
  expect(() => html(<Sprite src="a.png" tileX image_mode="Auto" />)).toThrow(/not both/);
});
