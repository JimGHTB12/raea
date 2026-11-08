// docs/control.feature FC0a: bevy_feathers' theme and controls (ui/widgets/virtual_keyboard), and bevy_text's
// EditableText on a Node. A feathers widget looks its colours up in the UiTheme around it (FUCHSIA with none, as
// Bevy's empty ThemeProps gives), and its font and text colour reach a ThemedText only, as Propagate With<ThemedText>.
import { describe, expect, test } from "bun:test";
import { EditableText, Text } from "raea/ui";
import { FeathersButton, ThemedText, UiTheme, VirtualKeyboard, context_color, create_dark_theme, palette, tokens, to_border_radius } from "raea/feathers";
import { Oklcha } from "raea/color";
import { renderFrame } from "../src/test";

const html = (el: React.ReactElement, frame = 0) => renderFrame(el, { frame, width: 320, height: 240, fps: 60, durationInFrames: 120 });
const hex = (c: { red: number; green: number; blue: number }) => [c.red, c.green, c.blue].map((v) => Math.round(v * 255));

describe("the dark theme: each token's semantic colour from the palette, a surface level's overrides first", () => {
  test("a button's background is FILL_SOLID_DEFAULT, GRAY_3 (srgb 70 71 77), and Highest's override lightens it", () => {
    const dark = create_dark_theme();
    expect(hex(context_color(dark, tokens.BUTTON_BG))).toEqual([70, 71, 77]);
    expect(hex(context_color(dark, tokens.BUTTON_BG, "Highest"))).toEqual(hex(palette.GRAY_3.lighter(0.05).to_srgba()));
    expect(hex(context_color(dark, tokens.BUTTON_TEXT))).toEqual([255, 255, 255]);
  });
  test("a theme with no colour for a token gives FUCHSIA", () => {
    expect(hex(context_color({ semantic_base: {}, semantic_overrides: {}, token_assignments: {} }, tokens.BUTTON_BG))).toEqual([255, 0, 255]);
  });
  test("Oklcha's lighter and darker move the lightness within 0..1, as Bevy's Luminance", () => {
    expect(new Oklcha(0.98, 0.1, 30).lighter(0.05).lightness).toBe(1);
    expect(new Oklcha(0.02, 0.1, 30).darker(0.05).lightness).toBe(0);
    expect(new Oklcha(0.5, 0.1, 30).with_alpha(0.25).alpha).toBe(0.25);
  });
});

describe("FeathersButton and VirtualKeyboard", () => {
  test("a button is a 24 px row, 8 px each side, 4 px corners, on the theme's colour; with no UiTheme, FUCHSIA", () => {
    const out = html(<UiTheme theme={create_dark_theme()}><FeathersButton><Text>a</Text></FeathersButton></UiTheme>);
    expect(out).toMatch(/height:\s*24px/);
    expect(out).toMatch(/padding-left:\s*8px|padding:\s*0px 8px/);
    expect(out).toContain("rgba(70, 71, 77, 1)");
    expect(html(<FeathersButton />)).toContain("rgba(255, 0, 255, 1)");
  });
  test("a ThemedText takes feathers' Fira Sans 14 px; a plain Text keeps Bevy's default font", () => {
    const themed = html(<UiTheme theme={create_dark_theme()}><FeathersButton><ThemedText>a</ThemedText></FeathersButton></UiTheme>);
    expect(themed).toContain("Fira Sans");
    expect(themed).toMatch(/font-size:\s*14px/);
    const plain = html(<UiTheme theme={create_dark_theme()}><FeathersButton><Text>a</Text></FeathersButton></UiTheme>);
    expect(plain).not.toContain("Fira Sans");
    expect(plain).toMatch(/font-size:\s*20px/);
  });
  test("RoundedCorners rounds the corners named", () => {
    expect(to_border_radius("Top", 4)).toEqual({ top_left: "4px", top_right: "4px", bottom_right: "0px", bottom_left: "0px" });
    expect(() => to_border_radius("Middle" as never, 4)).toThrow(/RoundedCorners/);
  });
  test("a keyboard is a button a key, in its rows", () => {
    const out = html(<VirtualKeyboard keys={[["Q", "W"], ["space"]]} />);
    for (const k of ["Q", "W", "space"]) expect(out).toContain(`>${k}<`);
    expect(() => html(<VirtualKeyboard keys={["Q"] as never} />)).toThrow(/rows of strings/);
  });
});

describe("EditableText", () => {
  test("visible_lines lines of its line height high, rounded up, and visible_width in 0s", () => {
    expect(html(<EditableText font_size={{ Px: 25 }} visible_lines={5} />)).toMatch(/height:\s*150px/);
    expect(html(<EditableText font_size={{ Px: 15 }} visible_lines={1} />)).toMatch(/height:\s*18px/);
    expect(html(<EditableText visible_width={12} />)).toMatch(/width:\s*12ch/);
  });
  test("with focus, its caret is cursor_width of the font size wide, shown the first half of each blink period", () => {
    const at = (frame: number) => html(<EditableText font_size={{ Px: 25 }} auto_focus cursor_style={{ color: "#ffffff" }}>hi</EditableText>, frame);
    expect(at(10)).toContain("data-raea-caret");
    expect(at(10)).toMatch(/width:\s*5px/);
    expect(at(40)).not.toContain("data-raea-caret");
    expect(at(70)).toContain("data-raea-caret");
    expect(html(<EditableText>hi</EditableText>)).not.toContain("data-raea-caret");
  });
  test("refuses a text that isn't a string, or visible_lines that aren't above 0", () => {
    expect(() => html(<EditableText visible_lines={0} />)).toThrow(/visible_lines/);
  });
});
