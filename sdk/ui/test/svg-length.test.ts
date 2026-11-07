// SVG geometry lengths (#16): getTotalLength and getPointAtLength on path, line, polyline, polygon, circle, ellipse
// and rect, against Chrome 148's answers for the same shapes. Chrome flattens circles, ellipses and rounded corners
// coarsely (a circle of r 40 measures 250.92 there, 2πr is 251.33); raea measures the true outline, which its
// renderer dashes along, so those are compared within 0.2% and the rest within 0.03%.
import { beforeAll, describe, expect, test } from "bun:test";
import { install } from "../src/dom";

beforeAll(() => install());
const SVG = "http://www.w3.org/2000/svg";
const ROUND = new Set(["circle", "ellipse"]);

// [tag, attributes, Chrome's length, Chrome's points at 0, 0.3 and 0.77 of it]
const CHROME: [string, Record<string, string | number>, number, number[][]][] = [
  ["path", {"d": "M0 0 L100 0"}, 100, [[0, 0], [30, 0], [77, 0]]],
  ["path", {"d": "M10 10 h50 v30 H10 z"}, 160, [[10, 10], [58, 10], [16.8, 40]]],
  ["path", {"d": "M0 0 C 50 100 100 -100 150 0 S 250 100 300 0"}, 418.5452, [[0, 0], [94.92, -18.515], [239.315, 72.268]]],
  ["path", {"d": "M0 0 Q 50 80 100 0 T 200 0"}, 266.7412, [[0, 0], [62.973, 37.307], [155.309, -39.549]]],
  ["path", {"d": "M10 80 a25 25 0 1 1 50 0 a25 25 0 01 50 0"}, 157.1191, [[10, 80], [42.724, 56.207], [88.13, 55.194]]],
  ["path", {"d": "M0 0 A 100 50 30 0 1 120 40"}, 130.6555, [[0, 0], [38.844, -0.153], [95.584, 22.535]]],
  ["path", {"d": "M0 0 l10 0 M50 50 l0 20"}, 30, [[0, 0], [9, 0], [50, 63.1]]],
  ["line", {"x1": 0, "y1": 0, "x2": 30, "y2": 40}, 50, [[0, 0], [9, 12], [23.1, 30.8]]],
  ["polyline", {"points": "0,0 30,0 30,40"}, 70, [[0, 0], [21, 0], [30, 23.9]]],
  ["polygon", {"points": "0,0 30,0 30,40"}, 120, [[0, 0], [30, 6], [16.56, 22.08]]],
  ["circle", {"cx": 50, "cy": 50, "r": 40}, 250.9214, [[90, 50], [37.681, 88.056], [54.961, 10.309]]],
  ["ellipse", {"cx": 50, "cy": 50, "rx": 80, "ry": 20}, 342.5899, [[130, 50], [33.031, 69.545], [56.676, 30.07]]],
  ["rect", {"x": 5, "y": 5, "width": 100, "height": 60}, 320, [[5, 5], [101, 5], [18.6, 65]]],
  ["rect", {"x": 5, "y": 5, "width": 100, "height": 60, "rx": 10}, 302.427, [[15, 5], [103.828, 10.303], [13.396, 64.871]]],
];

describe("SVGGeometryElement (#16)", () => {
  for (const [tag, attrs, length, points] of CHROME) {
    test(`${tag} ${JSON.stringify(attrs)}`, () => {
      const el = (globalThis as any).document.createElementNS(SVG, tag);
      for (const [k, v] of Object.entries(attrs)) el.setAttribute(k, String(v));
      const L = el.getTotalLength();
      const coarse = ROUND.has(tag) || attrs.rx !== undefined;
      expect(Math.abs(L - length) / length).toBeLessThan(coarse ? 0.002 : 0.0003);
      [0, 0.3, 0.77].forEach((f, i) => {
        const p = el.getPointAtLength(L * f);
        expect(Math.hypot(p.x - points[i][0], p.y - points[i][1])).toBeLessThan(coarse ? 0.25 : 0.02);
      });
    });
  }

  test("a path made by the page parser has it too, and it is an SVGGeometryElement", () => {
    const d = (globalThis as any).document;
    const host = d.createElement("div");
    host.innerHTML = `<svg><path id="p" d="M0 0 L100 0"/></svg>`;
    const p = host.querySelector("path");
    expect(p instanceof (globalThis as any).SVGGeometryElement).toBe(true);
    expect(p.getTotalLength()).toBe(100);
    expect(p.getPointAtLength(1e9)).toMatchObject({ x: 100, y: 0 });
  });
});
