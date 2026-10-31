// Suspense with transitions: a tab switched in a transition to one that suspends keeps the old tab (isPending) until
// its data comes; switched urgently, the boundary shows its fallback and hides what it showed, then reveals it again.
// proves: react.components.Suspense, react.hooks.useTransition, react.api.use
import { Suspense, use, useState, useTransition } from "react";
const waits: Record<string, { p: Promise<string>; r: (v: string) => void }> = {};
function wait(k: string) {
  if (!waits[k]) {
    let r: (v: string) => void = () => {};
    const p = new Promise<string>((res) => (r = res));
    waits[k] = { p, r };
  }
  return waits[k];
}
// the first tab's data is already there, as React reads a settled promise (status "fulfilled"): a bare
// Promise.resolve suspends once and React holds its fallback 300 ms before revealing, so the mount read raced Bevy's
// first frames, which take 300 ms of wall clock to get through a read's settling (Chrome reveals at 326 ms, Bevy at
// 307 to 352: the same; Chrome's read comes at about 90, Bevy's after 300)
waits.a = { p: Object.assign(Promise.resolve("A"), { status: "fulfilled", value: "A" }), r: () => {} };
function Tab({ k }: { k: string }) {
  const v = use(wait(k).p);
  return <div data-testid="tab" data-value={v} className="h-12 w-56 bg-cyan-300">{v}</div>;
}
export const steps = [{ wait: 4 }, { click: "to-b" }, { click: "load-b" }, { wait: 4 }, { click: "to-c" }, { click: "load-c" }, { wait: 4 }];
export default function Case() {
  const [k, setK] = useState("a");
  const [pending, start] = useTransition();
  return (
    <div data-testid="root" className="h-full w-full bg-white p-4 text-slate-900">
      <div data-testid="state" data-value={k + (pending ? " pending" : "")} className="mb-2 flex gap-2">
        <button data-testid="to-b" className="bg-slate-200 px-2" onClick={() => start(() => setK("b"))}>b (transition)</button>
        <button data-testid="load-b" className="bg-slate-200 px-2" onClick={() => wait("b").r("B")}>load b</button>
        <button data-testid="to-c" className="bg-slate-200 px-2" onClick={() => setK("c")}>c (urgent)</button>
        <button data-testid="load-c" className="bg-slate-200 px-2" onClick={() => wait("c").r("C")}>load c</button>
      </div>
      <Suspense fallback={<div data-testid="fallback" className="h-12 w-24 bg-slate-300" />}><Tab k={k} /></Suspense>
    </div>
  );
}
