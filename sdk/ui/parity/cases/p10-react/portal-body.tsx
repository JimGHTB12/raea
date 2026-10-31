// Portals out of the root: a modal into document.body (fixed, over the page) and a layer into an element made with
// document.createElement and appended to body (as Radix and React Aria do); React events from inside bubble through
// the React tree to the component that rendered the portal, not through the DOM's.
// proves: react-dom.createPortal
import { useEffect, useState } from "react";
import { createPortal } from "react-dom";
import { trace } from "parity";
export const steps = [{ click: "open" }, { click: "inside" }, { click: "layer-btn" }, { click: "close" }];
export default function Case() {
  const [open, setOpen] = useState(false);
  const [layer, setLayer] = useState<HTMLElement | null>(null);
  useEffect(() => {
    const el = document.createElement("div");
    el.setAttribute("data-layer", "");
    document.body.appendChild(el);
    setLayer(el);
    return () => void el.remove();
  }, []);
  return (
    <div data-testid="root" className="h-full w-full bg-white p-4 text-slate-900" onClick={(e) => trace("click", "root", (e.target as HTMLElement).getAttribute("data-testid"))}>
      <button data-testid="open" className="bg-slate-200 px-2" onClick={() => setOpen(true)}>open</button>
      {open &&
        createPortal(
          <div data-testid="modal" className="fixed left-20 top-24 flex h-40 w-80 items-start gap-2 bg-slate-800 p-4">
            <button data-testid="inside" className="bg-white px-2" onClick={() => trace("click", "inside")}>inside</button>
            <button data-testid="close" className="bg-rose-300 px-2" onClick={(e) => { e.stopPropagation(); setOpen(false); }}>close</button>
          </div>,
          document.body,
        )}
      {layer && createPortal(<button data-testid="layer-btn" className="fixed bottom-4 right-4 h-10 w-32 bg-amber-300" onClick={() => trace("click", "layer")}>layer</button>, layer)}
    </div>
  );
}
