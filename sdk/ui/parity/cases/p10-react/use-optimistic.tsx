// useOptimistic: the optimistic item shows while its async action runs, and the real state replaces it when the action
// ends (released by a second click, so no clock decides what each side shows).
// proves: react.hooks.useOptimistic, react.api.startTransition
import { startTransition, useOptimistic, useState } from "react";
import { trace } from "parity";
let release: (() => void) | null = null;
export const steps = [{ click: "add" }, { click: "finish" }];
export default function Case() {
  const [items, setItems] = useState(["a"]);
  const [shown, addShown] = useOptimistic(items, (s: string[], n: string) => [...s, n + "?"]);
  const add = () =>
    startTransition(async () => {
      addShown("b");
      trace("action", "start");
      await new Promise<void>((r) => (release = r));
      trace("action", "end");
      startTransition(() => setItems((s) => [...s, "b"]));
    });
  return (
    <div data-testid="root" className="h-full w-full bg-white p-4 text-slate-900">
      <div className="flex gap-2">
        <button data-testid="add" className="bg-slate-200 px-2" onClick={add}>add</button>
        <button data-testid="finish" className="bg-slate-200 px-2" onClick={() => release?.()}>finish</button>
      </div>
      <div data-testid="list" data-value={shown.join(",")} className="mt-2">
        {shown.map((i) => <div key={i} data-testid={"item-" + i} className={i.endsWith("?") ? "mb-1 h-6 w-32 bg-amber-200" : "mb-1 h-6 w-32 bg-sky-300"}>{i}</div>)}
      </div>
    </div>
  );
}
