// MaterialNode (docs/control.feature FC2a, Bevy's ui/ui_material): the spec its picture element carries for the
// engine, and what it refuses.
import { describe, expect, test } from "bun:test";
import { BorderRadius, CornerRadius, MaterialNode, UiRect, percent, px } from "raea/ui";
import { renderFrame } from "../src/test";

const html = (el: React.ReactElement) => renderFrame(el, { frame: 0, width: 320, height: 240, fps: 60, durationInFrames: 1 });
const spec = (out: string) => {
  const m = /data-raea-ui-material="([^"]*)"/.exec(out);
  if (!m) throw new Error("no data-raea-ui-material");
  return JSON.parse(m[1].replaceAll("&quot;", '"').replaceAll("&amp;", "&"));
};

describe("MaterialNode", () => {
  test("carries its shader, uniforms, textures, border widths in px and corners as Vals, over its border box", () => {
    const out = html(
      <MaterialNode width={px(100)} height={px(50)} border={UiRect.new(px(1), px(2), px(3), px(4))} border_radius={BorderRadius.new(px(5), percent(10), CornerRadius.new(px(6), px(7)), px(0))}
        fragment="/media/shaders/x.wesl" uniforms={{ tint: [1, 0, 0, 1] }} textures={{ pic: "/media/a.png" }} />,
    );
    const s = spec(out);
    expect(s.fragment).toBe("/media/shaders/x.wesl");
    expect(s.uniforms).toEqual({ tint: [1, 0, 0, 1] });
    expect(s.textures).toEqual({ pic: "/media/a.png" });
    expect(s.border).toEqual([1, 3, 2, 4]);
    expect(s.radius).toEqual([["5px", "auto"], ["10%", "auto"], ["6px", "7px"], ["0px", "auto"]]);
    // the picture covers the border box: out by the border on each side
    expect(out).toMatch(/left:\s*-1px;\s*top:\s*-3px;\s*width:\s*calc\(100% \+ 3px\);\s*height:\s*calc\(100% \+ 7px\)/);
  });
  test("refuses a border not in px and a missing fragment", () => {
    expect(() => html(<MaterialNode border={UiRect.all(percent(5))} fragment="/media/x.wesl" />)).toThrow(/border\.left in px/);
    expect(() => html(<MaterialNode fragment="" />)).toThrow(/fragment/);
  });
});
