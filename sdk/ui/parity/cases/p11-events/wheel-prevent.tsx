// preventDefault in a non-passive wheel listener stops the scroll the wheel would make; an unprevented wheel scrolls.
// proves: api.Event.preventDefault, api.Element.wheel_event, api.EventTarget.addEventListener.options_parameter.options_passive_parameter
import { useEffect, useRef } from "react";
import { trace } from "parity";
export const steps = [{ wheel: "locked", dy: 120 }, { wheel: "free", dy: 120 }];
export default function Case() {
  const ref = useRef<HTMLDivElement>(null);
  useEffect(() => {
    const el = ref.current!;
    const f = (e: WheelEvent) => { e.preventDefault(); trace("wheel", "locked", e.defaultPrevented); };
    el.addEventListener("wheel", f, { passive: false });
    return () => el.removeEventListener("wheel", f);
  }, []);
  return (
    <div data-testid="root" className="flex h-full w-full gap-4 bg-white p-4 text-slate-900">
      <div ref={ref} data-testid="locked" className="h-32 w-48 overflow-y-auto bg-slate-100">
        <div className="h-96 w-48 bg-gradient-to-b from-sky-200 to-violet-300" />
      </div>
      <div data-testid="free" className="h-32 w-48 overflow-y-auto bg-slate-100" onWheel={() => trace("wheel", "free")}>
        <div className="h-96 w-48 bg-gradient-to-b from-lime-200 to-amber-300" />
      </div>
    </div>
  );
}
