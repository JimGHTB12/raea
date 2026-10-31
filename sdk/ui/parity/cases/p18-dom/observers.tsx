// P18: the three observers from layout and the tree: ResizeObserver on a box that grows, IntersectionObserver on a
// row a scroller brings into view, MutationObserver on a list React adds to and re-classes.
// proves: api.ResizeObserver, api.ResizeObserver.observe, api.ResizeObserverEntry.contentRect, api.ResizeObserverEntry.borderBoxSize, api.ResizeObserverEntry.target, api.IntersectionObserver, api.IntersectionObserver.observe, api.IntersectionObserverEntry.isIntersecting, api.IntersectionObserverEntry.intersectionRatio, api.MutationObserver, api.MutationObserver.observe, api.MutationRecord.type, api.MutationRecord.addedNodes, api.MutationRecord.attributeName, api.MutationRecord.oldValue
// tier: quick
import { useEffect, useRef, useState } from "react";
import { trace } from "parity";

export const steps = [{ click: "grow" }, { wheel: "scroller", dy: 120 }, { wait: 3 }];

export default function Case() {
  const box = useRef<HTMLDivElement>(null);
  const list = useRef<HTMLDivElement>(null);
  const scroller = useRef<HTMLDivElement>(null);
  const row = useRef<HTMLDivElement>(null);
  const [wide, setWide] = useState(false);
  const [n, setN] = useState(1);
  useEffect(() => {
    const ro = new ResizeObserver((es) => es.forEach((e) => trace("resize", e.target.getAttribute("data-testid")!, [Math.round(e.contentRect.width), Math.round(e.contentRect.height), Math.round(e.borderBoxSize[0].inlineSize)])));
    ro.observe(box.current!);
    const io = new IntersectionObserver((es) => es.forEach((e) => trace("intersect", e.target.getAttribute("data-testid")!, [e.isIntersecting, Math.round(e.intersectionRatio * 10) / 10])), { root: scroller.current, threshold: [0, 0.5, 1] });
    io.observe(row.current!);
    const mo = new MutationObserver((rs) => rs.forEach((r) => trace("mutation", (r.target as Element).getAttribute("data-testid")!, [r.type, r.addedNodes.length, r.removedNodes.length, r.attributeName, r.oldValue])));
    mo.observe(list.current!, { childList: true, attributes: true, attributeFilter: ["class"], attributeOldValue: true });
    return () => {
      ro.disconnect();
      io.disconnect();
      mo.disconnect();
    };
  }, []);
  return (
    <div data-testid="root" className="flex h-full w-full flex-col bg-white" style={{ padding: 16, gap: 8 }}>
      <div data-testid="box" ref={box} style={{ width: wide ? 200 : 100, height: 40, padding: 4, backgroundColor: "rgb(225, 29, 72)", flexShrink: 0 }} />
      <div data-testid="list" ref={list} className={n > 1 ? "flex flex-col many" : "flex flex-col"} style={{ gap: 2, flexShrink: 0 }}>
        {Array.from({ length: n }, (_, i) => (
          <div key={i} data-testid={`item${i}`} style={{ width: 60, height: 10, backgroundColor: "rgb(14, 165, 233)" }} />
        ))}
      </div>
      <div data-testid="scroller" ref={scroller} className="flex flex-col overflow-y-auto" style={{ width: 200, height: 100, backgroundColor: "rgb(241, 245, 249)", flexShrink: 0 }}>
        <div style={{ height: 150, flexShrink: 0 }} />
        <div data-testid="row" ref={row} style={{ height: 40, flexShrink: 0, backgroundColor: "rgb(16, 185, 129)" }} />
        <div style={{ height: 100, flexShrink: 0 }} />
      </div>
      <div
        data-testid="grow"
        onClick={() => {
          setWide(true);
          setN(2);
        }}
        style={{ width: 80, height: 32, backgroundColor: "rgb(30, 41, 59)", flexShrink: 0 }}
      />
    </div>
  );
}
