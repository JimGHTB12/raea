// An async transition (React 19): isPending holds from the click through the await, and the update after it commits
// when the action ends.
// proves: react.hooks.useTransition, react.api.startTransition
import { useState, useTransition } from "react";
import { trace } from "parity";
let release: (() => void) | null = null;
export const steps = [{ click: "save" }, { click: "finish" }];
export default function Case() {
  const [pending, start] = useTransition();
  const [saved, setSaved] = useState(0);
  return (
    <div data-testid="root" className="h-full w-full bg-white p-4 text-slate-900">
      <div className="flex gap-2">
      <button
        data-testid="save"
        data-value={pending ? "pending" : "idle"}
        className={pending ? "bg-amber-300 px-2" : "bg-slate-200 px-2"}
        onClick={() =>
          start(async () => {
            trace("start", "save");
            await new Promise<void>((r) => (release = r));
            start(() => setSaved((s) => s + 1));
            trace("end", "save");
          })
        }
      >save</button>
      <button data-testid="finish" className="bg-slate-200 px-2" onClick={() => release?.()}>finish</button>
      </div>
      <div data-testid="saved" data-value={String(saved)} className="mt-2 h-6 bg-green-400" style={{ width: 30 + saved * 50 }} />
    </div>
  );
}
