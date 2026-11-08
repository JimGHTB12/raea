// docs/control.feature FC0a: bevy_feathers' colour controls and the parts of feathers_gallery its frame can't show (the
// Okhsl rows are below Bevy's 720 px window): a colour slider's track as update_track_color colours it, its thumb as
// update_slider_pos places it, the swatch over the alpha pattern, the colour plane's bitmap and thumb, the list view as
// Bevy 0.20 draws it, and two bevy_ui rules the gallery brought: a spreading box shadow, and a border blended over what
// lies under its node rather than the node's own background.
import { describe, expect, test } from "bun:test";
import { Hsla, Okhsla, Srgba } from "raea/color";
import { BoxShadow, Node, UiRect, px } from "raea/ui";
import { FeathersColorPlane, FeathersColorSlider, FeathersColorSwatch, FeathersListRow, FeathersListView, FeathersTextInput, UiTheme, caption, channel_range, create_dark_theme, gradient_ends } from "raea/feathers";
import { renderFrame } from "../src/test";

const html = (el: React.ReactElement) => renderFrame(<UiTheme theme={create_dark_theme()}>{el}</UiTheme>, { frame: 0, width: 320, height: 240, fps: 60, durationInFrames: 10 });
const amber800 = Srgba.hex("#92400E");
const byte = (v: number) => Math.round(Math.max(0, Math.min(1, v)) * 255);
const rgba = (c: Srgba) => `rgba(${byte(c.red)}, ${byte(c.green)}, ${byte(c.blue)}, ${c.alpha})`;

describe("ColorChannel", () => {
  test("a hue runs 0 to 360, the rest 0 to 1", () => {
    expect(channel_range("HslHue")).toEqual({ start: 0, end: 360 });
    expect(channel_range("OkhslHue")).toEqual({ start: 0, end: 360 });
    expect(channel_range("Alpha")).toEqual({ start: 0, end: 1 });
  });
  test("gradient_ends: the channel at 0, half and full, the base's other channels kept", () => {
    const [a, , c] = gradient_ends("Green", Srgba.hex("#065F46")) as Srgba[];
    expect([a.red, a.green, c.green, c.blue]).toEqual([6 / 255, 0, 1, 70 / 255]);
    const [, m] = gradient_ends("HslLightness", amber800) as Hsla[];
    expect(m.lightness).toBe(0.5);
    expect(m.hue).toBeCloseTo(Hsla.from(amber800).hue, 4);
  });
});

describe("FeathersColorSlider", () => {
  test("an Hsla track keeps its stops' hues, so the hue track runs red, cyan and back to red the long way", () => {
    const out = html(<FeathersColorSlider channel="HslHue" value={180} />);
    expect(out).toMatch(/in hsl, hsl\(0\.000\d+ 100% 50% \/ 1\) 0px/);
    expect(out).toMatch(/hsl\(359\.99\d+ 100% 50% \/ 1\) 0px/);
  });
  test("an Okhsla track is the okhsl mix sampled into stops: half way from 180 to 360 is okhsl hue 270", () => {
    const out = html(<FeathersColorSlider channel="OkhslHue" value={0} />);
    expect(out).toContain("in oklab");
    expect(out).toContain(rgba(new Okhsla(270, 1, 0.5).to_srgba()));
    expect((out.match(/rgba\(/g) ?? []).length).toBeGreaterThan(48);
  });
  test("an Alpha track runs the base colour from clear to opaque in its own space", () => {
    const out = html(<FeathersColorSlider channel="Alpha" value={0.7} base_color={Srgba.hex("#065F46")} />);
    expect(out).toContain("in srgb, rgba(6, 95, 70, 0) 0px, rgba(6, 95, 70, 0) 7px, rgba(6, 95, 70, 0.5) 50%");
  });
  test("its thumb sits at the value's place along the track inside half a thumb each side", () => {
    expect(html(<FeathersColorSlider channel="HslHue" value={90} />)).toMatch(/left:\s*25%[^"]*transform:\s*translate\(-7px, -50%\)/);
  });
});

describe("FeathersColorSwatch", () => {
  test("its colour over the 16 px checkerboard centred on it, the right part opaque", () => {
    const out = html(<FeathersColorSwatch color={Srgba.hex("#065F46").with_alpha(0.7)} opaque_color_percentage={30} />);
    expect(out).toContain("conic-gradient(rgb(203, 203, 203) 0deg 90deg, rgb(124, 124, 124) 90deg 180deg");
    expect(out).toMatch(/background-size:\s*16px 16px/);
    expect(out).toMatch(/background-position:\s*50% 50%/);
    expect(out).toContain("rgba(6, 95, 70, 0.7)");
    expect(out).toMatch(/left:\s*70%[^"]*background-color:\s*rgba\(6, 95, 70, 1\)/);
  });
});

describe("FeathersColorPlane", () => {
  test("a bitmap of the plane, made again when the fixed channel moves, and its thumb at the value", () => {
    const a = html(<FeathersColorPlane plane="RedBlue" value={[0.25, 0.75, 0.2]} />);
    const b = html(<FeathersColorPlane plane="RedBlue" value={[0.25, 0.75, 0.6]} />);
    const url = (s: string) => s.match(/data:image\/png;base64,[A-Za-z0-9+/=]+/)?.[0];
    expect(url(a)).toBeDefined();
    expect(url(a)).not.toBe(url(b));
    expect(a).toMatch(/left:\s*25%[^"]*top:\s*25%/);
  });
  test("an unknown plane is refused", () => {
    expect(() => html(<FeathersColorPlane plane={"RedYellow" as never} />)).toThrow(/plane is one of/);
  });
});

describe("FeathersListView, as Bevy 0.20 draws it", () => {
  test("a scrollbar track in a 14 px gutter and no thumb, whether its rows overflow or not", () => {
    for (const rows of [2, 12]) {
      const out = html(<FeathersListView max_height={px(130)}>{Array.from({ length: rows }, (_, i) => <FeathersListRow key={i}>{caption(`row ${i}`)}</FeathersListRow>)}</FeathersListView>);
      expect(out).toMatch(/padding:\s*0px 14px 0px 0px/);
      expect(out).not.toContain("data-raea-bevy-thumb");
    }
  });
});

describe("FeathersTextInput", () => {
  test("visible_width sets its width in the font's zeros and stops it growing", () => {
    expect(html(<FeathersTextInput visible_width={10}>#065F46B3</FeathersTextInput>)).toMatch(/width:\s*10ch[^"]*flex:\s*0 0 auto/);
  });
});

describe("bevy_ui rules the gallery brought", () => {
  test("a spreading box shadow is sent with its spread and marked for bevy_ui's rule", () => {
    const out = html(<Node width={px(100)} height={px(40)} box_shadow={BoxShadow.new(new Srgba(0, 0, 0, 0.9), px(0), px(0), px(1), px(4))} />);
    expect(out).toContain("0px 0px 8px 1px rgba(0, 0, 0, 0.9)");
    expect(out).toContain("data-raea-bevy-shadow");
    expect(html(<Node width={px(100)} box_shadow={BoxShadow.new(new Srgba(0, 0, 0, 1), px(2), px(2), px(0), px(4))} />)).not.toContain("data-raea-bevy-shadow");
  });
  test("a translucent border blends over what lies under the node, not the node's own background", () => {
    const half = new Srgba(1, 1, 1, 0.5);
    const out = html(
      <Node width={px(100)} height={px(100)} background_color={new Srgba(0, 0, 0)}>
        <Node width={px(40)} height={px(20)} border={UiRect.all(px(2))} background_color={half} border_color={half} />
      </Node>,
    );
    // white at 0.5 over black in linear light: 0.5 linear, sRGB 188, for the background and the border alike
    expect(out).toMatch(/background-color:\s*rgba\(188, 188, 188, 1\)/);
    expect(out).toMatch(/border-color:\s*rgba\(188, 188, 188, 1\)/);
  });
});
