// Mouse enter and leave (the mouse events, beside the pointer ones).
// proves: react-dom.event.onMouseEnter, react-dom.event.onMouseLeave, api.Element.mouseenter_event
import { trace } from "parity";
export const steps = [{ move: "card" }, { leave: true }];
export default function Case() {
  return (
    <div data-testid="root" className="h-full w-full bg-white p-4 text-slate-900">
      <div data-testid="card" className="h-24 w-48 bg-teal-200" onMouseEnter={() => trace("mouseenter", "card")} onMouseLeave={() => trace("mouseleave", "card")} />
    </div>
  );
}
