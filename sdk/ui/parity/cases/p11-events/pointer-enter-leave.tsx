// Pointer enter and leave as the pointer moves onto an element, onto its child (no leave: enter doesn't bubble) and off.
// proves: react-dom.event.onPointerEnter, react-dom.event.onPointerLeave, api.Element.pointerenter_event, api.Element.pointerleave_event
// tier: quick
import { trace } from "parity";
export const steps = [{ move: "card", at: [0.1, 0.1] }, { move: "dot" }, { move: [700, 500] }];
export default function Case() {
  return (
    <div data-testid="root" className="h-full w-full bg-white p-4 text-slate-900">
      <div data-testid="card" className="flex h-32 w-64 items-center justify-center bg-slate-200" onPointerEnter={() => trace("pointerenter", "card")} onPointerLeave={() => trace("pointerleave", "card")}>
        <div data-testid="dot" className="h-8 w-8 bg-rose-500" onPointerEnter={() => trace("pointerenter", "dot")} onPointerLeave={() => trace("pointerleave", "dot")} />
      </div>
    </div>
  );
}
