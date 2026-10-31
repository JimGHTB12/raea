// useActionState: an action's state threaded through dispatches (sync and async), isPending while the async one runs.
// proves: react.hooks.useActionState, react.api.startTransition
import { startTransition, useActionState } from "react";
import { trace } from "parity";
let release: (() => void) | null = null;
async function act(prev: number, n: number) {
  trace("action", "run", { prev, n });
  if (n < 0) await new Promise<void>((r) => (release = r));
  return prev + Math.abs(n);
}
export const steps = [{ click: "inc" }, { click: "inc" }, { click: "slow" }, { click: "finish" }];
export default function Case() {
  const [count, dispatch, pending] = useActionState(act, 0);
  return (
    <div data-testid="root" className="h-full w-full bg-white p-4 text-slate-900">
      <div className="flex gap-2">
        <button data-testid="inc" className="bg-slate-200 px-2" onClick={() => startTransition(() => dispatch(1))}>+1</button>
        <button data-testid="slow" className="bg-slate-200 px-2" onClick={() => startTransition(() => dispatch(-5))}>+5 later</button>
        <button data-testid="finish" className="bg-slate-200 px-2" onClick={() => release?.()}>finish</button>
      </div>
      <div data-testid="bar" data-value={count + (pending ? " pending" : "")} className={pending ? "mt-2 h-6 bg-amber-300" : "mt-2 h-6 bg-emerald-400"} style={{ width: 20 + count * 20 }} />
    </div>
  );
}
