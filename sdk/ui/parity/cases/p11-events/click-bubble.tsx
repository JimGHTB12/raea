// A click bubbles: the inner handler runs, then the outer one, each tracing its event phase and currentTarget.
// proves: react-dom.event.onClick, api.Element.click_event
// tier: quick
import { trace } from "parity";
export const steps = [{ click: "inner" }];
export default function Case() {
  return (
    <div data-testid="root" className="h-full w-full bg-white p-4 text-slate-900">
      <div data-testid="outer" className="w-64 bg-slate-200 p-6" onClick={(e) => trace("click", "outer", { phase: e.eventPhase, target: (e.target as HTMLElement).dataset?.testid })}>
        <div data-testid="inner" className="h-12 bg-sky-400" onClick={(e) => trace("click", "inner", { phase: e.eventPhase, target: (e.target as HTMLElement).dataset?.testid })} />
      </div>
    </div>
  );
}
