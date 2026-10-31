// P18: a host element's geometry read synchronously, as Chrome lays it out: getBoundingClientRect, client, offset and
// scroll sizes, the window's size, matchMedia, getComputedStyle and requestAnimationFrame.
// proves: api.Element.getBoundingClientRect, api.DOMRect, api.Element.clientWidth, api.Element.clientHeight, api.Element.scrollHeight, api.Element.scrollWidth, api.Element.scrollTop, api.HTMLElement.offsetWidth, api.HTMLElement.offsetHeight, api.HTMLElement.offsetTop, api.HTMLElement.offsetLeft, api.Window.innerWidth, api.Window.innerHeight, api.Window.matchMedia, api.MediaQueryList.matches, api.Window.requestAnimationFrame, api.Window.getComputedStyle, api.CSSStyleDeclaration.getPropertyValue
// tier: quick
import { useRef } from "react";
import { trace } from "parity";

export const steps = [{ click: "measure" }];

export default function Case() {
  const box = useRef<HTMLDivElement>(null);
  const list = useRef<HTMLDivElement>(null);
  const measure = () => {
    const b = box.current!;
    const r = b.getBoundingClientRect();
    trace("rect", "box", [r.x, r.y, r.width, r.height, r.top, r.right, r.bottom, r.left].map(Math.round));
    trace("sizes", "box", [b.clientWidth, b.clientHeight, b.offsetWidth, b.offsetHeight, b.offsetTop, b.offsetLeft]);
    const l = list.current!;
    trace("scroll", "list", [l.clientHeight, l.scrollHeight, l.clientWidth, l.scrollWidth, l.scrollTop]);
    trace("window", "", [innerWidth, innerHeight, matchMedia("(min-width: 640px)").matches, matchMedia("(max-width: 500px)").matches, matchMedia("(prefers-reduced-motion: reduce)").matches, matchMedia("(hover: hover) and (pointer: fine)").matches]);
    const cs = getComputedStyle(b);
    trace("computed", "box", [cs.display, cs.position, cs.getPropertyValue("animation-name"), cs.visibility, cs.width, cs.height]);
    trace("computed", "list", [getComputedStyle(l).overflowY, getComputedStyle(l).getPropertyValue("overflow-y")]);
    requestAnimationFrame(() => trace("raf", "", typeof performance.now()));
  };
  return (
    <div data-testid="root" className="flex h-full w-full flex-col bg-white" style={{ padding: 16, gap: 8 }}>
      <div data-testid="box" ref={box} className="flex flex-col" style={{ width: 120, height: 60, backgroundColor: "rgb(225, 29, 72)", flexShrink: 0 }} />
      <div data-testid="list" ref={list} className="flex flex-col overflow-y-auto" style={{ width: 200, height: 100, backgroundColor: "rgb(241, 245, 249)", flexShrink: 0 }}>
        {[0, 1, 2, 3, 4].map((i) => (
          <div key={i} data-testid={`row${i}`} style={{ height: 40, flexShrink: 0, backgroundColor: i % 2 ? "rgb(14, 165, 233)" : "rgb(16, 185, 129)" }} />
        ))}
      </div>
      <div data-testid="measure" onClick={measure} style={{ width: 80, height: 32, backgroundColor: "rgb(30, 41, 59)", flexShrink: 0 }} />
    </div>
  );
}
