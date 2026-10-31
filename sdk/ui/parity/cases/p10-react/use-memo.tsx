// useMemo: a filtered list recomputed only when its input changes.
// proves: react.hooks.useMemo, react-dom.element.li
import { useMemo, useState } from "react";
import { trace } from "parity";
export const steps = [{ click: "toggle" }, { click: "toggle" }];
const ALL = [1, 2, 3, 4, 5, 6];
export default function Case() {
  const [even, setEven] = useState(false);
  const shown = useMemo(() => { trace("memo", "list", even); return even ? ALL.filter((x) => x % 2 === 0) : ALL; }, [even]);
  return (
    <div data-testid="root" className="h-full w-full bg-white p-4 text-slate-900">
      <button data-testid="toggle" className="bg-slate-200 px-2" onClick={() => setEven(!even)}>toggle</button>
      <ul data-testid="list" className="mt-2">
        {shown.map((x) => <li key={x} data-testid={"item-" + x} className="h-6">{x}</li>)}
      </ul>
    </div>
  );
}
