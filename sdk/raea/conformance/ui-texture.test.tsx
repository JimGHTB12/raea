// docs/control.feature FC0a: bevy_ui drawn into an image a material samples (ui/render_ui_to_texture). A UiTexture's UI
// is a screen's picture (painted on its own, off the frame's view), and the material names that screen's image.
import { describe, expect, test } from "bun:test";
import { Node, UiTexture, targetImage } from "raea/ui";
import { renderFrame } from "../src/test";

const html = (el: React.ReactElement) => renderFrame(el, { frame: 0, width: 320, height: 240, fps: 30, durationInFrames: 30 });

describe("targetImage: an image a camera renders into, as a material's texture names it", () => {
  test("its name and size, as raea-sys-screen's named_image reads them", () => {
    expect(targetImage("ui", 512, 256)).toBe("raea-screen:ui@512x256");
  });
  test("refuses a name the page couldn't carry, or a size that isn't whole px", () => {
    expect(() => targetImage("a b", 1, 1)).toThrow(/name/);
    expect(() => targetImage("ui", 0, 1)).toThrow(/whole px/);
    expect(() => targetImage("ui", 1.5, 1)).toThrow(/whole px/);
  });
});

describe("UiTexture: the roots inside drawn into the image instead of the window", () => {
  test("a screen's picture of the image's name and size, see-through, off the frame's view", () => {
    const out = html(<UiTexture target={targetImage("ui", 512, 256)}><Node /></UiTexture>);
    expect(out).toContain('data-raea-screen="ui"');
    expect(out).toContain('data-raea-screen-w="512"');
    expect(out).toContain('data-raea-screen-h="256"');
    expect(out).toMatch(/background-color:\s*transparent/);
    expect(out).toMatch(/left:\s*-20000px/);
  });
  test("refuses a target that isn't a targetImage", () => {
    expect(() => html(<UiTexture target="branding/icon.png" />)).toThrow(/targetImage/);
  });
});
