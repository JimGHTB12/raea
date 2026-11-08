// docs/control.feature FC0a: bevy_ui's debug overlay (GlobalUiDebugOptions, UiDebugOptions) on the page
// (ui/images/image_node_resizing). Blitz draws the outlines over every UI node (vendor/blitz-paint bevy_debug.rs).
import { describe, expect, test } from "bun:test";
import { GlobalUiDebugOptions, Node, Text, UiScale } from "raea/ui";
import { Srgba } from "raea/color";
import { sequentialDispersed } from "../../render/src/bevy3d/ui-debug";
import { renderFrame } from "../src/test";

const html = (el: React.ReactElement) => renderFrame(el, { frame: 0, width: 320, height: 240, fps: 30, durationInFrames: 30 });
const hex = (i: number) => "#" + sequentialDispersed(i).map((v) => Math.round(v * 255).toString(16).padStart(2, "0")).join("");

describe("Hsla::sequential_dispersed: an outline's colour by its node's index", () => {
  test("bevy_color's golden-ratio hues at s 1, l 0.5 (as Bevy drew image_node_resizing's entities 431 to 443)", () => {
    expect(hex(0)).toBe("#ff0000");
    expect([431, 433, 434, 443].map(hex)).toEqual(["#00ff3c", "#0059ff", "#a3ff00", "#bb00ff"]);
  });
});

describe("GlobalUiDebugOptions and a node's own UiDebugOptions", () => {
  test("off unless enabled; enabled, every node inside outlines its border box 1 px in its index's colour", () => {
    expect(html(<Node debug_color_index={3} />)).not.toContain("data-raea-bevy-debug");
    const [r, g, b] = sequentialDispersed(3);
    expect(html(<GlobalUiDebugOptions enabled><Node debug_color_index={3} /></GlobalUiDebugOptions>)).toContain(`data-raea-bevy-debug="1 ${r} ${g} ${b} 1 b 0"`);
  });
  test("a node's own options replace the global ones (Text's too); an override colour, more boxes, show_clipped", () => {
    const off = html(<GlobalUiDebugOptions enabled><Text ui_debug={{ enabled: false }}>a</Text></GlobalUiDebugOptions>);
    expect(off).not.toContain("data-raea-bevy-debug");
    const own = html(<Node ui_debug={{ enabled: true, line_width: 2, line_color_override: new Srgba(1, 0, 0, 0.5), outline_padding_box: true, outline_content_box: true, show_clipped: true }} />);
    expect(own).toContain('data-raea-bevy-debug="2 1 0 0 0.5 bpc 1"');
  });
  test("its width in the page's px: times a UiScale's zoom", () => {
    expect(html(<UiScale value={2}><GlobalUiDebugOptions enabled line_width={1.5}><Node /></GlobalUiDebugOptions></UiScale>)).toContain('data-raea-bevy-debug="3 ');
  });
  test("refuses a negative width or a fractional index", () => {
    expect(() => html(<Node ui_debug={{ enabled: true, line_width: -1 }} />)).toThrow(/line_width/);
    expect(() => html(<GlobalUiDebugOptions enabled><Node debug_color_index={1.5} /></GlobalUiDebugOptions>)).toThrow(/debug_color_index/);
  });
});
