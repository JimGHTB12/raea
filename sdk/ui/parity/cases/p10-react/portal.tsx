// A portal: a child rendered into another element (a sibling container), its events bubbling through React's tree.
// proves: react-dom.createPortal
import { useState } from "react";
import { createPortal } from "react-dom";
import { trace } from "parity";
function Into({ host }: { host: HTMLElement | null }) {
  if (!host) return null;
  return createPortal(<button data-testid="ported" className="bg-amber-300 px-2" onClick={() => trace("click", "ported")}>in the portal</button>, host);
}
export const steps = [{ click: "ported" }];
export default function Case() {
  const [host, setHost] = useState<HTMLElement | null>(null);
  return (
    <div data-testid="root" className="h-full w-full bg-white p-4 text-slate-900" onClick={() => trace("click", "root")}>
      <div data-testid="source" className="mb-2 h-10 w-64 bg-slate-100"><Into host={host} /></div>
      <div data-testid="target" ref={setHost} className="h-10 w-64 bg-emerald-100" />
    </div>
  );
}
