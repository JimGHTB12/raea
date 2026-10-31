// A press that moves the focus, every event in Chrome's order: pointerdown, mousedown, then blur and focusout on the
// field it leaves, focus and focusin on the one it lands on, then pointerup, mouseup, click.
// proves: api.Element.focusout_event, api.Element.focusin_event, react-dom.event.onMouseDown, react-dom.event.onClick
import { trace } from "parity";
export const steps = [{ click: "a" }, { click: "b" }];
export default function Case() {
  const h = (id: string) => ({
    onPointerDown: () => trace("pointerdown", id), onMouseDown: () => trace("mousedown", id),
    onFocus: () => trace("focus", id), onBlur: () => trace("blur", id),
    onPointerUp: () => trace("pointerup", id), onMouseUp: () => trace("mouseup", id), onClick: () => trace("click", id),
  });
  return (
    <div data-testid="root" className="h-full w-full bg-white p-4 text-slate-900">
      <input data-testid="a" className="block h-9 w-56 border border-slate-400 px-2 text-sm" {...h("a")} />
      <input data-testid="b" className="mt-2 block h-9 w-56 border border-slate-400 px-2 text-sm" {...h("b")} />
    </div>
  );
}
