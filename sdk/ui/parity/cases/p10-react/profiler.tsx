// Profiler renders its children and leaves layout alone (production React calls no onRender; both sides run it).
// proves: react.components.Profiler
import { Profiler, useState } from "react";
import { trace } from "parity";
export const steps = [{ click: "inc" }];
export default function Case() {
  const [n, setN] = useState(0);
  return (
    <div data-testid="root" className="h-full w-full bg-white p-4 text-slate-900">
      <Profiler id="bar" onRender={(id, phase) => trace("render", id, phase)}>
        <button data-testid="inc" data-value={String(n)} className="bg-slate-200 px-2" style={{ width: 50 + n * 30 }} onClick={() => setN(n + 1)}>{n}</button>
      </Profiler>
    </div>
  );
}
