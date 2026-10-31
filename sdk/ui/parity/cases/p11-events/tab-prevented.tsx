// Tab's focus move is its keydown's default action: prevented, the focus stays where it is.
// proves: api.Event.preventDefault, api.Element.keydown_event, html.global_attributes.tabindex
import { trace } from "parity";
export const steps = [{ click: "a" }, { key: "Tab" }];
export default function Case() {
  return (
    <div data-testid="root" className="h-full w-full bg-white p-4 text-slate-900">
      <div data-testid="a" tabIndex={0} className="h-12 w-48 bg-slate-200 outline-none focus:bg-sky-200" onKeyDown={(e) => { if (e.key === "Tab") e.preventDefault(); trace("keydown", "a", e.key); }} />
      <div data-testid="b" tabIndex={0} className="mt-2 h-12 w-48 bg-slate-200 outline-none focus:bg-sky-200" onFocus={() => trace("focus", "b")} />
    </div>
  );
}
