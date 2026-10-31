// A drag: pointermove while pressed keeps going to the element pressed, even off it (setPointerCapture).
// proves: react-dom.event.onPointerMove, api.Element.setPointerCapture
import { trace } from "parity";
export const steps = [{ down: "knob" }, { move: [300, 60] }, { up: [300, 60] }];
export default function Case() {
  return (
    <div data-testid="root" className="h-full w-full bg-white p-4 text-slate-900">
      <div data-testid="knob" className="h-12 w-12 bg-violet-400"
        onPointerDown={(e) => { (e.target as Element).setPointerCapture?.(e.pointerId); trace("pointerdown", "knob"); }}
        onPointerMove={(e) => { if (e.buttons) trace("pointermove", "knob", Math.round(e.clientX)); }}
        onPointerUp={() => trace("pointerup", "knob")} />
    </div>
  );
}
