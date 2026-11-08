// docs/control.feature FC0a: bevy_ui's ScrollPosition and bevy_ui_widgets' Scrollbar on the page
// (ui/scroll_and_overflow/scrollbars). Blitz sizes and places the thumb after layout (vendor/blitz-dom bevy_scroll.rs).
import { describe, expect, test } from "bun:test";
import { Node, Overflow, Scrollbar, ScrollbarThumb, UiRect, UiScale, px } from "raea/ui";
import { Srgba } from "raea/color";
import { renderFrame } from "../src/test";

const html = (el: React.ReactElement) => renderFrame(el, { frame: 0, width: 320, height: 240, fps: 30, durationInFrames: 30 });

describe("ScrollPosition: a scroller's offset, on its Scroll axes only", () => {
  test("the offset the page asked for, in physical px under a UiScale, rides along for the thumb", () => {
    const out = html(<UiScale value={1.25}><Node overflow={Overflow.scroll()} scroll_position={[3, 10]} /></UiScale>);
    expect(out).toContain('data-raea-bevy-scroll="1 3.75 12.5"');
    expect(out).toMatch(/zoom:1.25/);
  });
  test("an axis that doesn't scroll stays at 0", () => {
    const out = html(<Node overflow={Overflow.scroll_y()} scroll_position={[30, 10]} />);
    expect(out).toContain('data-raea-bevy-scroll="1 30 10"');
  });
  test("is [x, y] in px", () => {
    expect(() => html(<Node overflow={Overflow.scroll()} scroll_position={[0, NaN]} />)).toThrow(/scroll_position is \[x, y\]/);
  });
});

describe("Scrollbar: a track whose thumb shows its target's view", () => {
  const bar = (p: Partial<React.ComponentProps<typeof Scrollbar>> = {}) => (
    <Node>
      <Node name="area" overflow={Overflow.scroll()} />
      <Scrollbar target="area" orientation="Vertical" min_thumb_length={8} min_width={px(8)} {...p}>
        <ScrollbarThumb background_color={new Srgba(0.5, 0.5, 0.5)} border={UiRect.all(px(1))} />
      </Scrollbar>
    </Node>
  );
  test("Name is the target's id; the track names orientation, least thumb length and target", () => {
    const out = html(bar());
    expect(out).toContain('id="area"');
    expect(out).toContain('data-raea-bevy-scrollbar="v 8 area"');
    expect(html(bar({ orientation: "Horizontal", min_thumb_length: 12 }))).toContain('data-raea-bevy-scrollbar="h 12 area"');
  });
  test("the track is laid out as a Node", () => {
    expect(html(bar())).toMatch(/min-width:8px[^>]*data-raea-bevy-scrollbar/);
  });
  test("the thumb is out of the track's layout, with its own colour and border", () => {
    const out = html(bar());
    const thumb = out.match(/<div style="([^"]*)" data-raea-bevy-thumb="1"/);
    expect(thumb).not.toBeNull();
    expect(thumb![1]).toContain("position:absolute");
    expect(thumb![1]).toContain("border-width:1px");
    expect(thumb![1]).toContain("background-color:");
  });
  test("refuses a bad target, orientation or length, and a thumb or bar outside the UI", () => {
    expect(() => html(bar({ target: "two words" }))).toThrow(/target is a Node's name/);
    expect(() => html(bar({ orientation: "Up" as "Vertical" }))).toThrow(/orientation/);
    expect(() => html(bar({ min_thumb_length: -1 }))).toThrow(/min_thumb_length/);
    expect(() => html(<ScrollbarThumb />)).toThrow(/goes inside a Scrollbar/);
    expect(() => html(<Scrollbar target="a" orientation="Vertical" min_thumb_length={0} />)).toThrow(/goes inside a Node/);
  });
});
