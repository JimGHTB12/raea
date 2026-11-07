// docs/control.feature FC0a, 2d/sprite_tile: Bevy's SpriteImageMode::Tiled on a <Sprite> (tileX, tileY, tileStretch),
// the props reaching its row for raea-sys-sprite, and a stretch that can't tile refused.
import { describe, expect, test } from "bun:test";
import { Canvas3D, Sprite } from "raea/3d";
import { renderFrame } from "../src/test";

const html = (el: React.ReactElement) => renderFrame(<Canvas3D layered>{el}</Canvas3D>, { frame: 0, width: 320, height: 240, fps: 30, durationInFrames: 30 });

describe("Sprite: tiled", () => {
  test("tileX, tileY and tileStretch reach the sprite's props, with its size", () => {
    const out = html(<Sprite src="icon.png" width={300} height={200} tileX tileY tileStretch={0.5} />);
    for (const s of ["&quot;tileX&quot;:true", "&quot;tileY&quot;:true", "&quot;tileStretch&quot;:0.5", "&quot;width&quot;:300"]) expect(out).toContain(s);
  });
  test("a tileStretch of 0 or less, or one on a sprite that doesn't tile, is refused", () => {
    expect(() => html(<Sprite src="icon.png" tileX tileStretch={0} />)).toThrow(/tileStretch is above 0/);
    expect(() => html(<Sprite src="icon.png" tileStretch={0.5} />)).toThrow(/give tileX or tileY/);
    expect(() => html(<Sprite src="icon.png" tileY tileStretch={2} />)).not.toThrow();
  });
});
