import { beforeAll, expect, test } from "bun:test";
import { install } from "../src/dom";

beforeAll(() => install());
const g = globalThis as any;

test("each tag makes its own class, a window global, as Chrome's (Excalidraw asks instanceof HTMLBRElement on every key)", () => {
  for (const [tag, name] of [["br", "HTMLBRElement"], ["p", "HTMLParagraphElement"], ["h2", "HTMLHeadingElement"], ["td", "HTMLTableCellElement"], ["div", "HTMLDivElement"]]) {
    const el = g.document.createElement(tag);
    expect(el instanceof g[name]).toBe(true);
    expect(el instanceof g.HTMLElement).toBe(true);
    expect(el.constructor.name).toBe(name);
  }
  expect(g.document.createElement("p") instanceof g.HTMLBRElement).toBe(false);
});
