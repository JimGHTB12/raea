// <Screen> (spec/motion.feature, "3D: screens"): its props checked; a free-standing screen's panel row; a model's
// screens and hidden nodes on its row; the nested composition laid out at the screen's resolution, out of view, on the
// screen's own clock, with the page's input.
import { describe, expect, test } from "bun:test";
import { Canvas3D, Model, Screen } from "../src/3d";
import { useCurrentFrame, useVideoConfig } from "../src/index";
import { useInput } from "raea/games";
import { screenSpec } from "@raea/render/screen";
import { renderFrame } from "../src/test";

const props = (html: string, kind: string) => {
  const m = html.match(new RegExp(`data-3d="${kind}" data-props="([^"]*)"`));
  return m ? JSON.parse(m[1].replace(/&quot;/g, '"')) : undefined;
};
const cfg = { fps: 30, width: 64, height: 36, durationInFrames: 60 };

function Probe() {
  const f = useCurrentFrame();
  const { width, height } = useVideoConfig();
  const keys = useInput().keys.join("+");
  return <i data-probe={`${width}x${height}@${f}:${keys}`} />;
}

describe("Screen", () => {
  test("props are checked, naming the prop", () => {
    expect(() => screenSpec("Screen", "bad name!", {})).toThrow(/name/);
    expect(() => screenSpec("Screen", "s", { resolution: [4, 4] })).toThrow(/resolution/);
    expect(() => screenSpec("Screen", "s", { resolution: [320.5, 200] })).toThrow(/resolution/);
    expect(() => screenSpec("Screen", "s", { power: 2 })).toThrow(/power/);
    expect(() => screenSpec("Screen", "s", { crt: { maskKind: "dots" as never } })).toThrow(/maskKind/);
    expect(() => screenSpec("Screen", "s", { crt: { glass: "green" } })).toThrow(/glass/);
    expect(() => screenSpec("Screen", "s", { crt: { scanlines: -1 } })).toThrow(/scanlines/);
    expect(screenSpec("Screen", "s", {})).toEqual({ name: "s", w: 320, h: 200, crt: true });
    expect(() => renderFrame(<Canvas3D><Screen size={[0]} /></Canvas3D>, { ...cfg, frame: 0 })).toThrow(/size/);
  });

  test("a see-through background marks the spec (alpha), and brightness is a flat screen's", () => {
    for (const bg of ["#ffffff00", "#fff8", "transparent", "rgba(255, 255, 255, 0.5)", "rgb(0 0 0 / 0%)"]) {
      expect(screenSpec("Screen", "s", { crt: false, background: bg }).alpha).toBe(true);
    }
    for (const bg of [undefined, "#ffffff", "#ffffffff", "#fff", "white", "rgba(0, 0, 0, 1)"]) {
      expect(screenSpec("Screen", "s", { crt: false, background: bg }).alpha).toBeUndefined();
    }
    expect(screenSpec("Screen", "s", { crt: false, brightness: 2 }).brightness).toBe(2);
    expect(() => screenSpec("Screen", "s", { brightness: 2 })).toThrow(/crt.brightness/);
    expect(() => screenSpec("Screen", "s", { crt: false, brightness: -1 })).toThrow(/brightness/);
    const html = renderFrame(<Canvas3D><Screen name="l" crt={false} background="#ffffff00" brightness={1.5} /></Canvas3D>, { ...cfg, frame: 0 });
    expect(props(html, "Screen").screen).toEqual({ name: "l", w: 320, h: 200, crt: false, alpha: true, brightness: 1.5 });
    expect(props(html, "Screen").brightness).toBeUndefined();
  });

  test("a free-standing screen is a panel its resolution's shape, with the screen's spec", () => {
    const html = renderFrame(<Canvas3D><Screen name="tv" resolution={[400, 200]} size={[2]} position={[1, 2, 3]} power={0.5} crt={{ scanlines: 0.2 }} /></Canvas3D>, { ...cfg, frame: 0 });
    const p = props(html, "Screen");
    expect(p.width).toBe(2);
    expect(p.height).toBe(1);
    expect(p.position).toEqual([1, 2, 3]);
    expect(p.screenPower).toBe(0.5);
    expect(p.screen).toEqual({ name: "tv", w: 400, h: 200, crt: { scanlines: 0.2 } });
  });

  test("an app screen marks its module for the screen pass and lays out nothing of its own", () => {
    const html = renderFrame(<Canvas3D><Screen name="chat" app="chat.tsx" resolution={[480, 640]} crt={false}><Probe /></Screen></Canvas3D>, { ...cfg, frame: 0 });
    expect(html).toContain('data-raea-screen-app="chat.tsx"');
    expect(html).not.toContain("data-probe");
    expect(props(html, "Screen").app).toBeUndefined();
    expect(() => renderFrame(<Canvas3D><Screen app="" /></Canvas3D>, { ...cfg, frame: 0 })).toThrow(/app/);
  });

  test("a model's screens and hidden nodes go on its row", () => {
    const html = renderFrame(
      <Canvas3D><Model src="tv.glb" hide={["eyes"]} screens={{ glass: <Screen resolution={[384, 272]} crt={false}><Probe /></Screen> }} /></Canvas3D>,
      { ...cfg, frame: 0 },
    );
    const p = props(html, "Model");
    expect(p.hide).toEqual(["eyes"]);
    expect(p.screens).toEqual([{ material: "glass", name: "tv:glass", w: 384, h: 272, crt: false }]);
    expect(html).toContain('data-raea-screen="tv:glass"');
  });

  test("the nested composition lays out at the screen's size, on its own clock, with the page's input", () => {
    const html = renderFrame(
      <Canvas3D><Screen name="s" resolution={[320, 200]} from={10}><Probe /></Screen></Canvas3D>,
      { ...cfg, frame: 25, input: { 25: { keys: ["ArrowLeft"] } } },
    );
    expect(html).toContain('data-raea-screen="s"');
    expect(html).toContain('data-raea-screen-w="320"');
    expect(html).toContain('data-probe="320x200@15:ArrowLeft"');
  });
});
