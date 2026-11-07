// `name in el.style` answers as Chrome's does (#17): true for CSS properties, custom properties and webkit-prefixed
// names, false for an element's own properties, so GSAP's CSSPlugin sets `textContent` on the element.
import { beforeAll, expect, test } from "bun:test";
import { install } from "../src/dom";

beforeAll(() => install());
const style = () => (globalThis as any).document.createElement("div").style;

test("CSS properties are in a style declaration", () => {
  const s = style();
  for (const p of ["transform", "opacity", "backgroundColor", "cssFloat", "x", "d", "--accent", "webkitTransform", "cssText", "setProperty"]) expect(p in s).toBe(true);
});

test("an element's own properties are not", () => {
  const s = style();
  for (const p of ["textContent", "innerHTML", "innerText", "value", "scrollTop", "currentTime", "notAProperty"]) expect(p in s).toBe(false);
});
