// preventDefault on a checkbox's click: it stays unchecked, and the event says defaultPrevented.
// proves: api.Event.preventDefault, html.elements.input.type_checkbox
import { trace } from "parity";
export const steps = [{ click: "box" }];
export default function Case() {
  return (
    <div data-testid="root" className="h-full w-full bg-white p-4 text-slate-900">
      <input data-testid="box" type="checkbox" className="h-6 w-6" onClick={(e) => { e.preventDefault(); trace("click", "box", e.defaultPrevented); }} />
    </div>
  );
}
