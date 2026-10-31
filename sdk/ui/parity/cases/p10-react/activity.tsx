// Activity (React 19.2): a hidden subtree is not shown and keeps its state; its effects are cleaned up while hidden
// and run again when it's visible.
// proves: react.components.Activity
import { Activity, useEffect, useState } from "react";
import { trace } from "parity";
function Panel() {
  const [n, setN] = useState(0);
  useEffect(() => {
    trace("effect", "panel");
    return () => trace("cleanup", "panel");
  }, []);
  return <button data-testid="panel" data-value={String(n)} className="h-10 bg-indigo-300 px-2" style={{ width: 80 + n * 20 }} onClick={() => setN(n + 1)}>{n}</button>;
}
export const steps = [{ click: "panel" }, { click: "toggle" }, { click: "toggle" }];
export default function Case() {
  const [shown, setShown] = useState(true);
  return (
    <div data-testid="root" className="h-full w-full bg-white p-4 text-slate-900">
      <button data-testid="toggle" data-value={String(shown)} className="mb-2 block bg-slate-200 px-2" onClick={() => setShown(!shown)}>toggle</button>
      <Activity mode={shown ? "visible" : "hidden"}><Panel /></Activity>
      <div data-testid="after" className="mt-2 h-6 w-24 bg-slate-300" />
    </div>
  );
}
