// docs/control.feature FC0a: bevy_ui's UiTransform and BoxShadow on the page (ui/scroll_and_overflow/overflow_transform),
// and bevy_color's Luminance darker and lighter.
import { describe, expect, test } from "bun:test";
import { BoxShadow, Node, Overflow, Rot2, UiRect, UiTransform, Val2, percent, px } from "raea/ui";
import { LinearRgba, Srgba } from "raea/color";
import { renderFrame } from "../src/test";

const html = (el: React.ReactElement) => renderFrame(el, { frame: 0, width: 320, height: 240, fps: 30, durationInFrames: 30 });

describe("UiTransform: a node moved, turned and scaled about its centre after layout", () => {
  test("translation, then rotation (clockwise), then scale, as CSS's transform", () => {
    const out = html(<Node ui_transform={{ translation: Val2.px(10, -5), rotation: Rot2.degrees(90), scale: [2, 0.5] }} />);
    const m = out.match(/transform:translate\(10px, -5px\) rotate\(([\d.]+)rad\) scale\(2, 0.5\)/);
    expect(m).not.toBeNull();
    expect(Number(m![1])).toBeCloseTo(Math.PI / 2, 6);
  });
  test("a percent translation is of the node's own size, and auto is 0", () => {
    expect(html(<Node ui_transform={UiTransform.from_translation(Val2.new(percent(50), "auto"))} />)).toContain("transform:translate(50%, 0px)");
  });
  test("Rot2's radians, degrees and turn fractions are one angle; identity adds no transform", () => {
    const angle = (r: Rot2) => Math.atan2(r.sin, r.cos);
    expect(angle(Rot2.degrees(45))).toBeCloseTo(Math.PI / 4, 6);
    expect(angle(Rot2.turn_fraction(-0.25))).toBeCloseTo(-Math.PI / 2, 6);
    expect(angle(Rot2.radians(0.35))).toBeCloseTo(0.35, 6);
    expect(html(<Node ui_transform={UiTransform.IDENTITY} />)).not.toContain("transform");
    expect(html(<Node ui_transform={UiTransform.from_scale([1, 1])} />)).not.toContain("transform");
  });
  test("a bad transform throws", () => {
    expect(() => html(<Node ui_transform={{ scale: [NaN, 1] }} />)).toThrow(/scale/);
    expect(() => html(<Node ui_transform={{ rotation: { cos: "1", sin: 0 } as never }} />)).toThrow(/rotation/);
  });
});

describe("BoxShadow: drop shadows under a node", () => {
  test("Bevy's blur radius (its Gaussian's deviation) is half CSS's; offsets as they are; listed top first", () => {
    const out = html(<Node box_shadow={[
      { color: new Srgba(1, 0, 0), x_offset: px(1), y_offset: px(2), spread_radius: px(0), blur_radius: px(3) },
      { color: new Srgba(0, 0, 1), x_offset: px(-4), y_offset: px(5), spread_radius: px(0), blur_radius: px(0) },
    ]} />);
    expect(out).toMatch(/box-shadow:-4px 5px 0px 0px rgba\(0, 0, 255, 1\), 1px 2px 6px 0px rgba\(255, 0, 0, 1\)/);
    expect(html(<Node box_shadow={BoxShadow.new(new Srgba(0, 0, 0), px(30), px(30), px(0), px(4))} />)).toMatch(/box-shadow:30px 30px 8px 0px /);
  });
  test("a spread is sent as it is and its node marked, for raea's Blitz to spread it by bevy_ui's rule", () => {
    const out = html(<Node box_shadow={BoxShadow.new(new Srgba(0, 0, 0), px(1), px(1), px(2), px(4))} />);
    expect(out).toMatch(/box-shadow:1px 1px 8px 2px /);
    expect(out).toContain("data-raea-bevy-shadow");
  });
  test("a percent or a negative blur is refused, not drawn wrong", () => {
    expect(() => html(<Node box_shadow={BoxShadow.new(new Srgba(0, 0, 0), percent(20), px(1), px(0), px(4))} />)).toThrow(/x_offset/);
    expect(() => html(<Node box_shadow={BoxShadow.new(new Srgba(0, 0, 0), px(1), px(1), px(0), px(-1))} />)).toThrow(/blur_radius/);
  });
});

describe("Luminance: darker and lighter mix toward black or white in linear light", () => {
  test("darker lowers the linear luminance by the amount and keeps its chromaticity", () => {
    const c = new Srgba(0.15, 0.3, 0.25);
    const [a, b] = [LinearRgba.from(c), LinearRgba.from(c.darker(0.05))];
    expect(b.luminance()).toBeCloseTo(a.luminance() - 0.05, 4);
    expect(b.green / b.red).toBeCloseTo(a.green / a.red, 4);
    expect(c.darker(1).red).toBe(0);
  });
  test("lighter mixes toward white, at most to white", () => {
    const a = LinearRgba.rgb(0.2, 0.1, 0.05);
    expect(a.lighter(0.1).luminance()).toBeCloseTo(a.luminance() + 0.1, 5);
    const w = a.lighter(2);
    expect([w.red, w.green, w.blue]).toEqual([1, 1, 1]);
  });
});

describe("Scrolling: no scrollbar drawn, scrollbar_width a gutter beside a Scroll axis, IgnoreScroll", () => {
  test("a Scroll axis draws no scrollbar (bevy_ui draws none); Clip and Hidden leave it alone", () => {
    expect(html(<Node overflow={Overflow.scroll_y()} />)).toContain("scrollbar-width:none");
    expect(html(<Node overflow={Overflow.clip()} />)).not.toContain("scrollbar-width");
  });
  test("scrollbar_width is kept on the right of a y scroller and below an x scroller, past its padding", () => {
    expect(html(<Node overflow={Overflow.scroll_y()} scrollbar_width={20} />)).toContain("padding-right:20px");
    expect(html(<Node overflow={Overflow.scroll_x()} scrollbar_width={20} padding={UiRect.all(px(5))} />)).toMatch(/padding:5px 5px 5px 5px;[^"]*padding-bottom:25px/);
    expect(html(<Node overflow={Overflow.scroll()} scrollbar_width={8} padding={UiRect.all(percent(10))} />)).toMatch(/padding-right:calc\(10% \+ 8px\);padding-bottom:calc\(10% \+ 8px\)/);
    expect(html(<Node overflow={Overflow.clip_y()} scrollbar_width={20} />)).not.toContain("padding-right");
    expect(() => html(<Node overflow={Overflow.scroll()} scrollbar_width={-1} />)).toThrow(/scrollbar_width/);
  });
  test("IgnoreScroll is { x, y } booleans", () => {
    expect(() => html(<Node ignore_scroll={{ x: true, y: true }} />)).not.toThrow();
    expect(() => html(<Node ignore_scroll={{ x: 1 } as never} />)).toThrow(/ignore_scroll/);
  });
});
