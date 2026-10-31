// Focus and blur as Tab moves between two buttons.
// proves: react-dom.event.onFocus, react-dom.event.onBlur, api.Element.focus_event, api.Element.blur_event
import { trace } from "parity";
export const steps = [{ key: "Tab" }, { key: "Tab" }, { key: "Tab", mods: ["shift"] }];
export default function Case() {
  const h = (id: string) => ({ onFocus: () => trace("focus", id), onBlur: () => trace("blur", id) });
  return (
    <div data-testid="root" className="h-full w-full bg-white p-4 text-slate-900">
      <button data-testid="one" className="mr-2 bg-slate-200 px-2 focus:bg-sky-300" {...h("one")}>one</button>
      <button data-testid="two" className="bg-slate-200 px-2 focus:bg-sky-300" {...h("two")}>two</button>
    </div>
  );
}
