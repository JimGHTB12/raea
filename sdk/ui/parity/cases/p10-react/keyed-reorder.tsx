// A keyed list reversed: rows keep their identity (their own state) as they move.
// proves: react.components.Fragment, react-dom.element.li
// tier: quick
import { useState } from "react";
import { trace } from "parity";
function Row({ id }: { id: string }) {
  const [on, setOn] = useState(false);
  return <li data-testid={"row-" + id} data-value={on ? "on" : "off"} onClick={() => { trace("click", "row-" + id); setOn(!on); }} className={on ? "mb-1 h-8 bg-amber-300 px-2" : "mb-1 h-8 bg-slate-100 px-2"}>{id}</li>;
}
export const steps = [{ click: "row-b" }, { click: "reverse" }];
export default function Case() {
  const [ids, setIds] = useState(["a", "b", "c", "d"]);
  return (
    <div data-testid="root" className="h-full w-full bg-white p-4 text-slate-900">
      <button data-testid="reverse" className="mb-2 bg-slate-200 px-2" onClick={() => setIds([...ids].reverse())}>reverse</button>
      <ul data-testid="list">{ids.map((id) => <Row key={id} id={id} />)}</ul>
    </div>
  );
}
