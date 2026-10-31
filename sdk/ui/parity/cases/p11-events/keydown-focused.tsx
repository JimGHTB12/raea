// Keys go to the focused element: a tabIndex div focused by a click hears keydown with key and code.
// proves: react-dom.event.onKeyDown, api.Element.keydown_event, html.global_attributes.tabindex
// tier: quick
import { trace } from "parity";
export const steps = [{ click: "pad" }, { key: "a" }, { key: "ArrowDown" }, { key: "Enter" }];
export default function Case() {
  return (
    <div data-testid="root" className="h-full w-full bg-white p-4 text-slate-900">
      <div data-testid="pad" tabIndex={0} className="h-24 w-48 bg-slate-200 outline-none focus:bg-sky-200" onKeyDown={(e) => trace("keydown", "pad", { key: e.key, code: e.code })} />
    </div>
  );
}
