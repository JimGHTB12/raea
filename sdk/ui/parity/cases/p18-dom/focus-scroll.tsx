// P18: focus and scroll from JS, as Chrome does them: focus() moves the focus synchronously (activeElement, then
// blur, focusout, focus, focusin with relatedTarget), blur() lets it go, Tab moves it, a press focuses a button;
// scrollTop set reads back clamped at once, a scroll event follows, scrollIntoView and scrollTo move the scroller.
// proves: api.HTMLElement.focus, api.HTMLElement.blur, api.Document.activeElement, api.FocusEvent.relatedTarget, api.Element.focusin_event, api.Element.focusout_event, api.Element.scrollTop, api.Element.scrollIntoView, api.Element.scrollTo, api.Element.scroll_event, api.KeyboardEvent.key, api.HTMLElement.tabIndex
// tier: quick
import { useEffect, useRef } from "react";
import { trace } from "parity";

export const steps = [{ click: "go" }, { key: "Tab" }, { click: "drop" }, { click: "scroll" }, { wait: 3 }, { click: "into" }, { wait: 3 }];

const tid = (e: Element | null) => e?.getAttribute("data-testid") ?? (e === document.body ? "body" : null);

export default function Case() {
  const a = useRef<HTMLButtonElement>(null);
  const list = useRef<HTMLDivElement>(null);
  useEffect(() => {
    const fns: [string, (e: any) => void][] = ["focusin", "focusout"].map((t) => [t, (e: FocusEvent) => trace(t, tid(e.target as Element)!, [tid(e.relatedTarget as Element), tid(document.activeElement)])]);
    fns.forEach(([t, f]) => document.addEventListener(t, f));
    const k = (e: KeyboardEvent) => trace("keydown", tid(e.target as Element)!, e.key);
    document.addEventListener("keydown", k);
    const s = () => trace("scroll", "list", Math.round(list.current!.scrollTop));
    list.current!.addEventListener("scroll", s);
    return () => {
      fns.forEach(([t, f]) => document.removeEventListener(t, f));
      document.removeEventListener("keydown", k);
    };
  }, []);
  // no focus ring: drawing one is P12's (outline, :focus-visible)
  const box = (bg: string) => ({ width: 80, height: 28, backgroundColor: bg, flexShrink: 0, outline: "none" });
  return (
    <div data-testid="root" className="flex h-full w-full flex-col bg-white" style={{ padding: 16, gap: 8 }}>
      <button data-testid="go" onClick={() => (a.current!.focus(), trace("active", "go", tid(document.activeElement)))} style={box("rgb(30, 41, 59)")} />
      <button data-testid="a" ref={a} style={box("rgb(225, 29, 72)")} />
      <button data-testid="b" style={box("rgb(14, 165, 233)")} />
      <button
        data-testid="drop"
        onClick={(e) => {
          trace("tabIndex", "drop", [e.currentTarget.tabIndex, (document.querySelector("[data-testid=root]") as HTMLElement).tabIndex]);
          e.currentTarget.blur();
          trace("active", "drop", tid(document.activeElement));
        }}
        style={box("rgb(16, 185, 129)")}
      />
      <div data-testid="list" ref={list} className="flex flex-col overflow-y-auto" style={{ width: 200, height: 100, backgroundColor: "rgb(241, 245, 249)", flexShrink: 0 }}>
        {Array.from({ length: 20 }, (_, i) => (
          <div key={i} data-testid={`r${i}`} style={{ height: 30, flexShrink: 0, backgroundColor: i % 2 ? "rgb(203, 213, 225)" : "rgb(241, 245, 249)" }} />
        ))}
      </div>
      <div
        data-testid="scroll"
        onClick={() => {
          const l = list.current!;
          l.scrollTop = 150;
          trace("scrollTop", "list", Math.round(l.scrollTop));
          l.scrollTop = 99999;
          trace("scrollTop", "list", Math.round(l.scrollTop));
        }}
        style={box("rgb(124, 58, 237)")}
      />
      <div
        data-testid="into"
        onClick={() => {
          const l = list.current!;
          l.scrollTo({ top: 0 });
          trace("scrollTo", "list", Math.round(l.scrollTop));
          l.querySelector("[data-testid=r10]")!.scrollIntoView({ block: "nearest" });
          trace("intoView", "list", Math.round(l.scrollTop));
        }}
        style={box("rgb(234, 88, 12)")}
      />
    </div>
  );
}
