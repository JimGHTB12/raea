// use(): a promise read inside Suspense (the fallback until it resolves, on a click), and a context read
// conditionally, after an early return, which no other hook may do.
// proves: react.api.use, react.components.Suspense
import { createContext, Suspense, use, useState } from "react";
const Tone = createContext("plain");
let resolve: (v: string) => void = () => {};
const data = new Promise<string>((r) => (resolve = r));
function Body() {
  const v = use(data);
  return <div data-testid="body" data-value={v} className="h-10 w-48 bg-emerald-300">{v}</div>;
}
function Label({ show }: { show: boolean }) {
  if (!show) return <div data-testid="label" data-value="hidden" className="h-6 w-16 bg-slate-200" />;
  const t = use(Tone);
  return <div data-testid="label" data-value={t} className="h-6 w-40 bg-violet-300">{t}</div>;
}
export const steps = [{ click: "show" }, { click: "resolve" }];
export default function Case() {
  const [show, setShow] = useState(false);
  return (
    <Tone value="loud">
      <div data-testid="root" className="h-full w-full bg-white p-4 text-slate-900">
        <div className="flex gap-2">
          <button data-testid="show" className="bg-slate-200 px-2" onClick={() => setShow(true)}>show</button>
          <button data-testid="resolve" className="bg-slate-200 px-2" onClick={() => resolve("loaded")}>resolve</button>
        </div>
        <div className="my-2"><Label show={show} /></div>
        <Suspense fallback={<div data-testid="fallback" className="h-10 w-24 bg-slate-300" />}><Body /></Suspense>
      </div>
    </Tone>
  );
}
