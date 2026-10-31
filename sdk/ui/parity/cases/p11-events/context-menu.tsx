// A right click: contextmenu (prevented, so no menu), and no click.
// proves: react-dom.event.onContextMenu, api.Element.contextmenu_event
import { trace } from "parity";
export const steps = [{ click: "card", button: "right" }];
export default function Case() {
  return (
    <div data-testid="root" className="h-full w-full bg-white p-4 text-slate-900">
      <div data-testid="card" className="h-24 w-48 bg-amber-200" onClick={() => trace("click", "card")} onContextMenu={(e) => { e.preventDefault(); trace("contextmenu", "card", e.button); }} />
    </div>
  );
}
