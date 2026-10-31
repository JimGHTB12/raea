// Timers and microtasks in the order a browser runs them: microtasks before any timer, timers by due time then by
// the order they were set, setTimeout's extra arguments, clearTimeout, setInterval until it's cleared, and a zero
// timer set from a timer running after the ones already due. The delays leave room (400 ms and up) for late zero
// timers: Chrome ran its zero timers 10+ ms after the click, behind a 10 ms timer, and under load ran the nested one
// after a 60 ms timer; step 1 waits for all of them before the interval starts.
// proves: api.setTimeout, api.setTimeout.supports_parameters_for_callback, api.clearTimeout, api.setInterval, api.setInterval.supports_parameters_for_callback, api.clearInterval, api.queueMicrotask, wintertc.setTimeout, wintertc.queueMicrotask
import { useState } from "react";
import { trace } from "parity";
export const steps = [{ click: "go", wait: 60 }, { click: "interval", wait: 40 }];
export default function Case() {
  const [n, setN] = useState(0);
  const go = () => {
    const log = (s: string, d?: unknown) => trace("timer", s, d);
    setTimeout(() => log("t500"), 500);
    setTimeout(() => log("t400"), 400);
    setTimeout((a: number, b: string) => log("t0-args", [a, b]), 0, 7, "x");
    const dead = setTimeout(() => log("cleared"), 0);
    clearTimeout(dead);
    setTimeout(() => {
      log("t0-b");
      setTimeout(() => log("t0-nested"), 0);
      queueMicrotask(() => log("micro-in-timer"));
    }, 0);
    setTimeout(() => log("t0-c"), 0);
    queueMicrotask(() => log("micro"));
    Promise.resolve().then(() => log("promise"));
    log("sync");
    setTimeout(() => setN(1), 600);
  };
  const interval = () => {
    let k = 0;
    const id = setInterval((step: number) => {
      k += step;
      trace("interval", "tick", k);
      if (k >= 3) {
        clearInterval(id);
        setN(2);
      }
    }, 5, 1);
  };
  return (
    <div data-testid="root" className="flex h-full w-full gap-2 bg-white p-4">
      <div data-testid="go" className="h-8 w-16 bg-slate-300" onClick={go} />
      <div data-testid="interval" className="h-8 w-16 bg-slate-400" onClick={interval} />
      <div data-testid="state" data-value={String(n)} className="h-8 bg-emerald-400" style={{ width: 20 + n * 40 }} />
    </div>
  );
}
