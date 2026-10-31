// useReducer: two buttons dispatch, the state shows as text.
// proves: react.hooks.useReducer, react-dom.event.onClick
import { useReducer } from "react";
import { trace } from "parity";
export const steps = [{ click: "up" }, { click: "up" }, { click: "down" }];
function reduce(s: number, a: "up" | "down") { return a === "up" ? s + 1 : s - 1; }
export default function Case() {
  const [n, dispatch] = useReducer(reduce, 5);
  return (
    <div data-testid="root" className="h-full w-full bg-white p-4 text-slate-900">
      <div className="flex gap-2">
        <button data-testid="up" className="bg-slate-200 px-2" onClick={() => { trace("click", "up"); dispatch("up"); }}>up</button>
        <button data-testid="down" className="bg-slate-200 px-2" onClick={() => { trace("click", "down"); dispatch("down"); }}>down</button>
      </div>
      <p data-testid="value" data-value={String(n)}>value {n}</p>
    </div>
  );
}
