// StrictMode: a tree under it renders and updates as without it (production builds don't double-invoke; both sides
// run React's production build), effects included.
// proves: react.components.StrictMode
import { StrictMode, useEffect, useState } from "react";
import { trace } from "parity";
function Counter() {
  const [n, setN] = useState(0);
  useEffect(() => {
    trace("effect", "counter", n);
    return () => trace("cleanup", "counter", n);
  }, [n]);
  return <button data-testid="inc" data-value={String(n)} className="bg-slate-200 px-2" style={{ width: 60 + n * 30 }} onClick={() => setN(n + 1)}>{n}</button>;
}
export const steps = [{ click: "inc" }, { click: "inc" }];
export default function Case() {
  return (
    <StrictMode>
      <div data-testid="root" className="h-full w-full bg-white p-4 text-slate-900"><Counter /></div>
    </StrictMode>
  );
}
