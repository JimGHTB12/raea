// useTransition: an update in startTransition commits, and isPending goes back to false.
// proves: react.hooks.useTransition, react.api.startTransition
import { useState, useTransition } from "react";
export const steps = [{ click: "go" }, { wait: 6 }];
export default function Case() {
  const [tab, setTab] = useState("a");
  const [pending, start] = useTransition();
  return (
    <div data-testid="root" className="h-full w-full bg-white p-4 text-slate-900">
      <button data-testid="go" className="bg-slate-200 px-2" onClick={() => start(() => setTab("b"))}>go</button>
      <div data-testid="tab" data-value={tab + (pending ? "-pending" : "")} className={tab === "a" ? "mt-2 h-10 w-40 bg-slate-300" : "mt-2 h-10 w-80 bg-sky-300"} />
    </div>
  );
}
