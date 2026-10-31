// AbortController and AbortSignal (abort with and without a reason, the abort event once, throwIfAborted,
// AbortSignal.abort/timeout/any), EventTarget and CustomEvent made by hand, reportError and DOMException.
// proves: api.AbortController, api.AbortController.AbortController, api.AbortController.abort, api.AbortController.signal, api.AbortSignal, api.AbortSignal.aborted, api.AbortSignal.reason, api.AbortSignal.throwIfAborted, api.AbortSignal.abort_static, api.AbortSignal.timeout_static, api.AbortSignal.any_static, wintertc.AbortController, wintertc.AbortSignal, wintertc.EventTarget, wintertc.CustomEvent, wintertc.Event, wintertc.DOMException, wintertc.reportError
import { useState } from "react";
import { trace } from "parity";
export const steps = [{ click: "go" }, { wait: 20 }];
export default function Case() {
  const [done, setDone] = useState(false);
  const go = () => {
    const ac = new AbortController();
    let n = 0;
    ac.signal.addEventListener("abort", (e) => {
      n++;
      trace("abort", "event", [e.type, ac.signal.aborted]);
    });
    ac.signal.onabort = () => trace("abort", "onabort");
    trace("abort", "before", [ac.signal.aborted, ac.signal.reason]);
    ac.abort();
    ac.abort("again");
    const r = ac.signal.reason as DOMException;
    trace("abort", "after", [n, r instanceof DOMException, r.name, r.message]);
    try {
      ac.signal.throwIfAborted();
    } catch (e) {
      trace("abort", "throw", (e as Error).name);
    }
    const why = AbortSignal.abort("why");
    trace("static", "abort", [why.aborted, why.reason]);
    const t = AbortSignal.timeout(5);
    const other = new AbortController();
    const any = AbortSignal.any([t, other.signal]);
    any.addEventListener("abort", () => {
      trace("static", "any", [any.reason instanceof DOMException, (any.reason as DOMException).name, t.aborted, other.signal.aborted]);
      setDone(true);
    });
    const et = new EventTarget();
    et.addEventListener("ping", (e) => trace("target", "ping", [(e as CustomEvent).detail, e instanceof Event, e.cancelable]), { once: true });
    const okd = et.dispatchEvent(new CustomEvent("ping", { detail: { x: 1 }, cancelable: true }));
    et.dispatchEvent(new CustomEvent("ping", { detail: "second" }));
    trace("target", "dispatched", okd);
    const ex = new DOMException("msg", "NotFoundError");
    trace("domexception", "made", [ex.name, ex.message, ex.code, ex instanceof Error, DOMException.NOT_FOUND_ERR]);
    const onErr = (e: ErrorEvent) => {
      e.preventDefault();
      trace("reportError", "window", [e.message, (e.error as Error).message]);
    };
    window.addEventListener("error", onErr);
    reportError(new Error("reported"));
    window.removeEventListener("error", onErr);
  };
  return (
    <div data-testid="root" className="flex h-full w-full gap-2 bg-white p-4">
      <div data-testid="go" className="h-8 w-16 bg-slate-300" onClick={go} />
      <div data-testid="done" data-value={String(done)} className={done ? "h-8 w-40 bg-lime-400" : "h-8 w-10 bg-slate-200"} />
    </div>
  );
}
