// P18, a real library: TanStack Virtual (@tanstack/react-virtual 3.14.22) over 10,000 rows: the rows it renders follow
// the scroller's scrollTop, clientHeight and ResizeObserver, its scroll listener and scrollToIndex (scrollTo).
// proves: api.Element.scrollTop, api.Element.scrollTo, api.Element.scroll_event, api.ResizeObserver, api.Element.getBoundingClientRect, api.Node.ownerDocument, api.Document.defaultView
import { useRef } from "react";
import { useVirtualizer } from "@tanstack/react-virtual";
import { trace } from "parity";

export const steps = [{ click: "report" }, { wheel: "scroller", dy: 3000 }, { wait: 6 }, { click: "report" }, { click: "jump" }, { wait: 6 }, { click: "report" }];

export default function Case() {
  const parent = useRef<HTMLDivElement>(null);
  const v = useVirtualizer({ count: 10000, getScrollElement: () => parent.current, estimateSize: () => 30, overscan: 2 });
  const items = v.getVirtualItems();
  const report = () => trace("range", "scroller", [items[0]?.index ?? null, items[items.length - 1]?.index ?? null, items.length, Math.round(parent.current!.scrollTop), v.getTotalSize()]);
  return (
    <div data-testid="root" className="flex h-full w-full flex-col bg-white" style={{ padding: 16, gap: 8 }}>
      <div data-testid="scroller" ref={parent} className="flex flex-col overflow-y-auto" style={{ width: 300, height: 300, backgroundColor: "rgb(241, 245, 249)", flexShrink: 0 }}>
        <div className="relative" style={{ height: v.getTotalSize(), width: 300, flexShrink: 0 }}>
          {items.map((it) => (
            <div key={it.key} data-testid={`row${it.index}`} className="absolute" style={{ top: it.start, left: 0, width: 300, height: 30, backgroundColor: it.index % 2 ? "rgb(203, 213, 225)" : "rgb(241, 245, 249)" }} />
          ))}
        </div>
      </div>
      <div className="flex flex-row" style={{ gap: 8, flexShrink: 0 }}>
        <div data-testid="report" onClick={report} style={{ width: 80, height: 28, backgroundColor: "rgb(30, 41, 59)" }} />
        <div data-testid="jump" onClick={() => v.scrollToIndex(5000, { align: "start" })} style={{ width: 80, height: 28, backgroundColor: "rgb(225, 29, 72)" }} />
      </div>
    </div>
  );
}
