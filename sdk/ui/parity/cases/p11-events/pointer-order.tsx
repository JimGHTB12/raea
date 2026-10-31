// One press's events in order: pointerdown, mousedown, pointerup, mouseup, click.
// proves: react-dom.event.onPointerDown, react-dom.event.onPointerUp, react-dom.event.onMouseDown, react-dom.event.onMouseUp, react-dom.event.onClick
import { trace } from "parity";
export const steps = [{ click: "pad" }];
export default function Case() {
  return (
    <div data-testid="root" className="h-full w-full bg-white p-4 text-slate-900">
      <div data-testid="pad" className="h-24 w-48 bg-rose-200"
        onPointerDown={() => trace("pointerdown", "pad")} onMouseDown={() => trace("mousedown", "pad")}
        onPointerUp={() => trace("pointerup", "pad")} onMouseUp={() => trace("mouseup", "pad")}
        onClick={() => trace("click", "pad")} />
    </div>
  );
}
