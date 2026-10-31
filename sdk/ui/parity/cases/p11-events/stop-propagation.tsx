// stopPropagation: the inner handler stops the click; the outer one never hears it.
// proves: api.Event.stopPropagation, react-dom.event.onClick
import { trace } from "parity";
export const steps = [{ click: "inner" }, { click: "outer", at: [0.5, 0.1] }];
export default function Case() {
  return (
    <div data-testid="root" className="h-full w-full bg-white p-4 text-slate-900">
      <div data-testid="outer" className="w-64 bg-slate-200 p-6" onClick={() => trace("click", "outer")}>
        <div data-testid="inner" className="h-12 bg-sky-400" onClick={(e) => { e.stopPropagation(); trace("click", "inner"); }} />
      </div>
    </div>
  );
}
