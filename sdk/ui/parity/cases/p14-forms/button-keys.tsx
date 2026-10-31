// A focused button is clicked by Space and by Enter.
// proves: html.elements.button, react-dom.event.onClick
import { trace } from "parity";
export const steps = [{ key: "Tab" }, { key: " " }, { key: "Enter" }];
export default function Case() {
  return (
    <div data-testid="root" className="h-full w-full bg-white p-4 text-slate-900">
      <button data-testid="b" className="bg-slate-200 px-3 py-1 focus:bg-sky-300" onClick={(e) => trace("click", "b", e.detail)}>press</button>
    </div>
  );
}
