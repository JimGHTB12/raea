// useId: ids stable across renders and the same as React DOM's (the same algorithm in every renderer).
// proves: react.hooks.useId
import { useId, useState } from "react";
export const steps = [{ click: "again" }];
export default function Case() {
  const a = useId();
  const b = useId();
  const [, setN] = useState(0);
  return (
    <div data-testid="root" className="h-full w-full bg-white p-4 text-slate-900">
      <button data-testid="again" className="bg-slate-200 px-2" onClick={() => setN((n) => n + 1)}>again</button>
      <div data-testid="a" data-value={a} />
      <div data-testid="b" data-value={b} />
    </div>
  );
}
