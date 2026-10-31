// keyup after keydown at the focused element; Tab's keydown at the element it leaves, its keyup at the one it focuses.
// proves: react-dom.event.onKeyUp, api.Element.keyup_event
import { trace } from "parity";
export const steps = [{ click: "a" }, { key: "x" }, { key: "Tab" }];
export default function Case() {
  const h = (id: string) => ({ onKeyDown: (e: { key: string }) => trace("keydown", id, e.key), onKeyUp: (e: { key: string }) => trace("keyup", id, e.key) });
  return (
    <div data-testid="root" className="h-full w-full bg-white p-4 text-slate-900">
      <div data-testid="a" tabIndex={0} className="h-12 w-48 bg-slate-200 outline-none focus:bg-sky-200" {...h("a")} />
      <div data-testid="b" tabIndex={0} className="mt-2 h-12 w-48 bg-slate-200 outline-none focus:bg-sky-200" {...h("b")} />
    </div>
  );
}
