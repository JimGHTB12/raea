// A double click: two clicks, then the dblclick, with the click count in detail.
// proves: react-dom.event.onDoubleClick, api.Element.dblclick_event
import { trace } from "parity";
export const steps = [{ click: "card", clicks: 2 }];
export default function Case() {
  return (
    <div data-testid="root" className="h-full w-full bg-white p-4 text-slate-900">
      <div data-testid="card" className="h-24 w-48 bg-indigo-200" onClick={(e) => trace("click", "card", e.detail)} onDoubleClick={(e) => trace("dblclick", "card", e.detail)} />
    </div>
  );
}
