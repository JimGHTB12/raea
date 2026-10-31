// A wheel event's deltas, in the DOM's sign (positive deltaY: toward the end).
// proves: react-dom.event.onWheel, api.Element.wheel_event
import { trace } from "parity";
export const steps = [{ wheel: "pad", dy: 100 }, { wheel: "pad", dy: -40 }];
export default function Case() {
  return (
    <div data-testid="root" className="h-full w-full bg-white p-4 text-slate-900">
      <div data-testid="pad" className="h-24 w-48 bg-lime-200" onWheel={(e) => trace("wheel", "pad", { dx: e.deltaX, dy: e.deltaY })} />
    </div>
  );
}
