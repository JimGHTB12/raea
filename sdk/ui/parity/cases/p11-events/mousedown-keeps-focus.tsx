// A press focuses what it lands on after mousedown, unless mousedown is prevented: a toolbar button that prevents it
// leaves the field focused; a plain button takes the focus (blur on the field, then focus on it).
// proves: api.Event.preventDefault, react-dom.event.onMouseDown, api.Element.focus_event, api.Element.blur_event
import { trace } from "parity";
export const steps = [{ click: "field" }, { click: "keep" }, { click: "take" }];
export default function Case() {
  const f = (id: string) => ({ onFocus: () => trace("focus", id), onBlur: () => trace("blur", id) });
  return (
    <div data-testid="root" className="h-full w-full bg-white p-4 text-slate-900">
      <input data-testid="field" className="block h-9 w-56 border border-slate-400 px-2 text-sm" {...f("field")} />
      <div data-testid="keep" className="mt-2 h-8 w-24 bg-slate-200" onMouseDown={(e) => e.preventDefault()} onClick={() => trace("click", "keep")} />
      <div data-testid="take" tabIndex={0} className="mt-2 h-8 w-24 bg-slate-300" onClick={() => trace("click", "take")} {...f("take")} />
    </div>
  );
}
