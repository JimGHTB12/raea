// pointermove and mousemove as the pointer moves over an element with no button held (buttons 0).
// proves: react-dom.event.onPointerMove, react-dom.event.onMouseMove, api.Element.mousemove_event
import { trace } from "parity";
export const steps = [{ move: "pad", at: [0.25, 0.5] }, { move: "pad", at: [0.75, 0.5] }];
export default function Case() {
  return (
    <div data-testid="root" className="h-full w-full bg-white p-4 text-slate-900">
      <div data-testid="pad" className="h-24 w-48 bg-teal-200"
        onPointerMove={(e) => trace("pointermove", "pad", { x: Math.round(e.clientX), b: e.buttons })}
        onMouseMove={(e) => trace("mousemove", "pad", { x: Math.round(e.clientX), b: e.buttons })} />
    </div>
  );
}
