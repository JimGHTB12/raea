// ResizeObserver and IntersectionObserver as the viewport changes: a box sized in % of the window reports its new
// size when the window narrows, and an element pushed below the fold stops intersecting the viewport.
// proves: api.ResizeObserver, api.ResizeObserver.observe, api.ResizeObserver.unobserve, api.ResizeObserver.disconnect, api.ResizeObserverEntry.contentRect, api.ResizeObserverEntry.contentBoxSize, api.ResizeObserverSize.inlineSize, api.ResizeObserverSize.blockSize, api.IntersectionObserver, api.IntersectionObserver.observe, api.IntersectionObserver.disconnect, api.IntersectionObserverEntry.isIntersecting, api.IntersectionObserverEntry.intersectionRatio, api.IntersectionObserverEntry.target
import { useEffect, useRef } from "react";
import { trace } from "parity";
export const steps = [{ resize: [600, 300] }, { wait: 4 }, { resize: [800, 600] }, { wait: 4 }];
export default function Case() {
  const box = useRef<HTMLDivElement>(null);
  const low = useRef<HTMLDivElement>(null);
  useEffect(() => {
    const ro = new ResizeObserver((es) => es.forEach((e) => trace("resize", "box", [Math.round(e.contentRect.width), Math.round(e.contentRect.height), Math.round(e.contentBoxSize[0].inlineSize), Math.round(e.contentBoxSize[0].blockSize)])));
    ro.observe(box.current!);
    const io = new IntersectionObserver((es) => es.forEach((e) => trace("intersect", e.target.getAttribute("data-testid")!, [e.isIntersecting, Math.round(e.intersectionRatio * 10) / 10])), { threshold: [0, 1] });
    io.observe(low.current!);
    return () => {
      ro.disconnect();
      io.disconnect();
    };
  }, []);
  return (
    <div data-testid="root" className="h-full w-full bg-white">
      <div data-testid="box" ref={box} className="bg-sky-300" style={{ width: "50%", height: 40 }} />
      <div style={{ height: 380 }} />
      <div data-testid="low" ref={low} className="bg-rose-400" style={{ width: 100, height: 40 }} />
    </div>
  );
}
