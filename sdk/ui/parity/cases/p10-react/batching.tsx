// Automatic batching (React 18+): updates set together in a handler, a timeout and a promise each commit once.
// proves: react.hooks.useState, react-dom.unstable_batchedUpdates
import { useState } from "react";
import { trace } from "parity";
export const steps = [{ click: "handler" }, { click: "timeout" }, { wait: 6 }, { click: "promise" }, { wait: 6 }];
export default function Case() {
  const [a, setA] = useState(0);
  const [b, setB] = useState(0);
  const [c, setC] = useState(0);
  trace("render", "case", [a, b, c]);
  const all = () => {
    setA((x) => x + 1);
    setB((x) => x + 1);
    setC((x) => x + 1);
  };
  return (
    <div data-testid="root" className="h-full w-full bg-white p-4 text-slate-900">
      <div className="flex gap-2">
        <button data-testid="handler" className="bg-slate-200 px-2" onClick={all}>handler</button>
        <button data-testid="timeout" className="bg-slate-200 px-2" onClick={() => setTimeout(all, 0)}>timeout</button>
        <button data-testid="promise" className="bg-slate-200 px-2" onClick={() => Promise.resolve().then(all)}>promise</button>
      </div>
      <div data-testid="bar" data-value={[a, b, c].join(",")} className="mt-2 h-6 bg-teal-400" style={{ width: 20 + (a + b + c) * 10 }} />
    </div>
  );
}
