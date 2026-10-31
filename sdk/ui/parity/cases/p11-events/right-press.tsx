// A right press, each event with its button: pointerdown and mousedown, contextmenu (prevented, so no menu), then
// pointerup, mouseup and auxclick, and no click.
// proves: react-dom.event.onAuxClick, api.Element.auxclick_event, react-dom.event.onMouseDown
import { trace } from "parity";
export const steps = [{ click: "card", button: "right" }];
export default function Case() {
  const t = (ev: string) => (e: { button: number }) => trace(ev, "card", e.button);
  return (
    <div data-testid="root" className="h-full w-full bg-white p-4 text-slate-900">
      <div data-testid="card" className="h-24 w-48 bg-amber-200"
        onPointerDown={t("pointerdown")} onMouseDown={t("mousedown")}
        onContextMenu={(e) => { e.preventDefault(); trace("contextmenu", "card", e.button); }}
        onPointerUp={t("pointerup")} onMouseUp={t("mouseup")} onAuxClick={t("auxclick")} onClick={t("click")} />
    </div>
  );
}
