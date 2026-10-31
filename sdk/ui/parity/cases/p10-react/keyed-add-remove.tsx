// A keyed list grown and shrunk: an item added at the front, one removed from the middle.
// proves: react.hooks.useState, react-dom.element.li
import { useState } from "react";
export const steps = [{ click: "add" }, { click: "x-2" }];
export default function Case() {
  const [items, setItems] = useState([1, 2, 3]);
  const [next, setNext] = useState(4);
  return (
    <div data-testid="root" className="h-full w-full bg-white p-4 text-slate-900">
      <button data-testid="add" className="mb-2 bg-slate-200 px-2" onClick={() => { setItems([next, ...items]); setNext(next + 1); }}>add</button>
      <ul data-testid="list">
        {items.map((i) => (
          <li key={i} data-testid={"item-" + i} className="mb-1 flex h-8 w-48 items-center justify-between bg-slate-100 px-2">
            <span>{i}</span>
            <button data-testid={"x-" + i} className="bg-rose-200 px-1" onClick={() => setItems(items.filter((x) => x !== i))}>x</button>
          </li>
        ))}
      </ul>
    </div>
  );
}
