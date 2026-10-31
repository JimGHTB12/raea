// A second React root (react-dom/client's createRoot) on an element the first one rendered: it renders, takes
// clicks and keeps its own state, re-renders from outside, and unmounts.
// proves: react-dom.client.createRoot
import { useEffect, useRef, useState } from "react";
import { createRoot, type Root } from "react-dom/client";
import { trace } from "parity";
function Inner({ label }: { label: string }) {
  const [n, setN] = useState(0);
  return <button data-testid="inner" data-value={`${label} ${n}`} className="h-10 bg-pink-300 px-2" style={{ width: 100 + n * 20 }} onClick={() => { trace("click", "inner"); setN(n + 1); }}>{label}</button>;
}
export const steps = [{ click: "inner" }, { click: "relabel" }, { click: "inner" }, { click: "unmount" }];
export default function Case() {
  const host = useRef<HTMLDivElement>(null);
  const root = useRef<Root | null>(null);
  const [label, setLabel] = useState("one");
  useEffect(() => {
    root.current = createRoot(host.current!);
    return () => root.current?.unmount();
  }, []);
  useEffect(() => root.current?.render(<Inner label={label} />), [label]);
  return (
    <div data-testid="root" className="h-full w-full bg-white p-4 text-slate-900">
      <div className="flex gap-2">
        <button data-testid="relabel" className="bg-slate-200 px-2" onClick={() => setLabel("two")}>relabel</button>
        <button data-testid="unmount" className="bg-slate-200 px-2" onClick={() => { root.current?.unmount(); root.current = null; }}>unmount</button>
      </div>
      <div data-testid="host" ref={host} className="mt-2 min-h-12 w-64 bg-slate-100 p-1" />
    </div>
  );
}
