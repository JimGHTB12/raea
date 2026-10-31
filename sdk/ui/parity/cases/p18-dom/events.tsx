// P18: events through real capture and bubble paths over the host tree: addEventListener on window, document and
// elements and React's own handlers see a press in Chrome's order (pointerdown, mousedown, pointerup, mouseup,
// click), with currentTarget, eventPhase, target, stopPropagation, stopImmediatePropagation, preventDefault, a
// removed listener, and a CustomEvent dispatched from JS.
// proves: api.EventTarget.addEventListener, api.EventTarget.removeEventListener, api.EventTarget.dispatchEvent, api.Event.preventDefault, api.Event.defaultPrevented, api.Event.stopPropagation, api.Event.stopImmediatePropagation, api.Event.currentTarget, api.Event.eventPhase, api.Event.target, api.Event.type, api.Event.bubbles, api.CustomEvent, api.CustomEvent.detail, api.MouseEvent.clientX, api.MouseEvent.button, api.PointerEvent.pointerType
// tier: quick
import { useEffect, useRef } from "react";
import { trace } from "parity";

export const steps = [{ click: "inner" }, { click: "stopper" }, { click: "custom" }];

const name = (t: any): string => (t === window ? "window" : t === document ? "document" : (t?.getAttribute?.("data-testid") ?? t?.tagName ?? "?"));
const PHASE = ["none", "capture", "target", "bubble"];

export default function Case() {
  const outer = useRef<HTMLDivElement>(null);
  const inner = useRef<HTMLDivElement>(null);
  const stopper = useRef<HTMLDivElement>(null);
  useEffect(() => {
    const offs: (() => void)[] = [];
    const on = (t: any, type: string, fn: (e: any) => void, capture = false) => {
      t.addEventListener(type, fn, capture);
      offs.push(() => t.removeEventListener(type, fn, capture));
    };
    for (const t of [window, document, outer.current!, inner.current!]) {
      on(t, "click", (e) => trace("click", name(e.currentTarget), [PHASE[e.eventPhase], name(e.target)]), true);
      on(t, "click", (e) => trace("click", name(e.currentTarget), [PHASE[e.eventPhase], name(e.target)]));
    }
    for (const type of ["pointerdown", "mousedown", "pointerup", "mouseup"])
      on(inner.current!, type, (e) => {
        const r = inner.current!.getBoundingClientRect();
        trace(type, "inner", [e.button, e.pointerType ?? null, Math.round(e.clientX - r.left), Math.round(e.clientY - r.top), e.bubbles, e.cancelable]);
      });
    // removed before it can hear anything
    const never = () => trace("never", "inner");
    inner.current!.addEventListener("click", never);
    inner.current!.removeEventListener("click", never);
    // the first of two listeners stops the second
    on(stopper.current!, "click", (e) => {
      e.stopImmediatePropagation();
      e.preventDefault();
      trace("first", "stopper", e.defaultPrevented);
    });
    on(stopper.current!, "click", () => trace("second", "stopper"));
    on(outer.current!, "ping", (e) => trace("ping", "outer", [e.detail, PHASE[e.eventPhase], name(e.target)]));
    on(document, "ping", (e) => {
      e.preventDefault();
      trace("ping", "document", [e.detail, e.defaultPrevented]);
    });
    return () => offs.forEach((f) => f());
  }, []);
  const ping = () => {
    const ok = inner.current!.dispatchEvent(new CustomEvent("ping", { bubbles: true, cancelable: true, detail: 7 }));
    trace("dispatched", "inner", ok);
  };
  return (
    <div data-testid="root" className="flex h-full w-full flex-col bg-white" style={{ padding: 16, gap: 8 }}>
      <div
        data-testid="outer"
        ref={outer}
        className="flex flex-col"
        onClickCapture={(e) => trace("react capture", "outer", name(e.target))}
        onClick={(e) => trace("react click", "outer", [name(e.target), name(e.currentTarget)])}
        style={{ width: 240, height: 120, padding: 20, backgroundColor: "rgb(241, 245, 249)", flexShrink: 0 }}
      >
        <div data-testid="inner" ref={inner} onClick={(e) => trace("react click", "inner", name(e.target))} style={{ width: 100, height: 50, backgroundColor: "rgb(225, 29, 72)", flexShrink: 0 }} />
      </div>
      <div data-testid="stopper" ref={stopper} onClick={(e) => (trace("react click", "stopper", e.defaultPrevented), e.stopPropagation())} style={{ width: 100, height: 40, backgroundColor: "rgb(14, 165, 233)", flexShrink: 0 }} />
      <div data-testid="custom" onClick={ping} style={{ width: 100, height: 40, backgroundColor: "rgb(16, 185, 129)", flexShrink: 0 }} />
    </div>
  );
}
