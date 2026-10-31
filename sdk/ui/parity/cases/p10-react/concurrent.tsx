// Concurrent rendering: an urgent update and a transition set in the same handler commit separately, the urgent one
// first; a second transition before the first commits supersedes it (only the latest commits).
// proves: react.hooks.useTransition, react.api.startTransition, react-dom.client.createRoot
import { startTransition, useState } from "react";
import { trace } from "parity";
export const steps = [{ click: "both" }, { wait: 6 }, { click: "twice" }, { wait: 6 }];
export default function Case() {
  const [urgent, setUrgent] = useState(0);
  const [slow, setSlow] = useState(0);
  trace("render", "case", { urgent, slow });
  return (
    <div data-testid="root" className="h-full w-full bg-white p-4 text-slate-900">
      <div className="flex gap-2">
        <button data-testid="both" className="bg-slate-200 px-2" onClick={() => { startTransition(() => setSlow((s) => s + 10)); setUrgent((u) => u + 1); }}>both</button>
        <button data-testid="twice" className="bg-slate-200 px-2" onClick={() => { startTransition(() => setSlow(100)); startTransition(() => setSlow(200)); }}>twice</button>
      </div>
      <div data-testid="urgent" data-value={String(urgent)} className="mt-2 h-6 bg-rose-300" style={{ width: 20 + urgent * 20 }} />
      <div data-testid="slow" data-value={String(slow)} className="mt-2 h-6 bg-indigo-300" style={{ width: 20 + slow }} />
    </div>
  );
}
