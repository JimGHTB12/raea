// useDeferredValue: the deferred copy catches up with the value.
// proves: react.hooks.useDeferredValue
import { useDeferredValue, useState } from "react";
export const steps = [{ click: "more" }, { wait: 6 }];
export default function Case() {
  const [n, setN] = useState(1);
  const d = useDeferredValue(n);
  return (
    <div data-testid="root" className="h-full w-full bg-white p-4 text-slate-900">
      <button data-testid="more" className="bg-slate-200 px-2" onClick={() => setN(n + 3)}>more</button>
      <div data-testid="bar" data-value={String(d)} className="mt-2 h-6 bg-lime-500" style={{ width: d * 30 }} />
    </div>
  );
}
