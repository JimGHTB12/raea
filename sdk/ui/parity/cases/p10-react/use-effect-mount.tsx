// useEffect after the mount: the effect runs once and sets state, which shows on the next render.
// proves: react.hooks.useEffect, react.hooks.useState
// tier: quick
import { useEffect, useState } from "react";
import { trace } from "parity";
export const steps = [{ wait: 4 }];
export default function Case() {
  const [ready, setReady] = useState(false);
  useEffect(() => { trace("effect", "root"); setReady(true); }, []);
  return (
    <div data-testid="root" className="h-full w-full bg-white p-4 text-slate-900">
      <div data-testid="badge" data-value={ready ? "ready" : "waiting"} className={ready ? "w-32 bg-emerald-500 p-2" : "w-32 bg-amber-400 p-2"}>{ready ? "ready" : "waiting"}</div>
    </div>
  );
}
