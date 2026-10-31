// memo (skipping a render when props are equal, or as a custom compare says), forwardRef and ref as a plain prop
// (React 19) both reaching the host element, createRef on a class, and a ref callback's cleanup (React 19).
// proves: react.api.memo, react.api.forwardRef, react.api.createRef, react.hooks.useRef
import { Component, createRef, forwardRef, memo, useRef, useState, type Ref } from "react";
import { trace } from "parity";
const Plain = memo(function Plain({ v }: { v: number }) {
  trace("render", "plain", v);
  return <div data-testid="plain" className="h-6 bg-sky-300" style={{ width: 40 + v * 10 }} />;
});
const Even = memo(
  function Even({ v }: { v: number }) {
    trace("render", "even", v);
    return <div data-testid="even" data-value={String(v)} className="h-6 w-24 bg-emerald-300" />;
  },
  (a, b) => a.v % 2 === b.v % 2,
);
const Fwd = forwardRef<HTMLDivElement>(function Fwd(_, ref) {
  return <div data-testid="fwd" ref={ref} className="h-6 w-24 bg-amber-300" />;
});
function AsProp({ ref }: { ref: Ref<HTMLDivElement> }) {
  return <div data-testid="asprop" ref={ref} className="h-6 w-24 bg-rose-300" />;
}
class Klass extends Component {
  box = createRef<HTMLDivElement>();
  componentDidMount() { trace("createRef", "klass", this.box.current?.getAttribute("data-testid")); }
  render() { return <div data-testid="klass" ref={this.box} className="h-6 w-24 bg-violet-300" />; }
}
export const steps = [{ click: "same" }, { click: "bump" }, { click: "bump" }, { click: "read" }, { click: "toggle" }, { click: "toggle" }];
export default function Case() {
  const [v, setV] = useState(0);
  const [, force] = useState(0);
  const [cb, setCb] = useState(true);
  const fwd = useRef<HTMLDivElement>(null);
  const prop = useRef<HTMLDivElement>(null);
  return (
    <div data-testid="root" className="flex h-full w-full flex-col items-start gap-1 bg-white p-4 text-slate-900">
      <div className="flex gap-2">
        <button data-testid="same" className="bg-slate-200 px-2" onClick={() => force((x) => x + 1)}>same</button>
        <button data-testid="bump" className="bg-slate-200 px-2" onClick={() => setV(v + 1)}>bump</button>
        <button data-testid="read" className="bg-slate-200 px-2" onClick={() => trace("refs", "case", [fwd.current?.getAttribute("data-testid"), prop.current?.getAttribute("data-testid"), fwd.current?.tagName])}>read</button>
        <button data-testid="toggle" className="bg-slate-200 px-2" onClick={() => setCb(!cb)}>toggle</button>
      </div>
      <Plain v={v} />
      <Even v={v} />
      <Fwd ref={fwd} />
      <AsProp ref={prop} />
      <Klass />
      {cb && (
        <div
          data-testid="cb"
          className="h-6 w-24 bg-teal-300"
          ref={(el) => {
            trace("ref", "cb", el?.getAttribute("data-testid"));
            return () => trace("ref-cleanup", "cb");
          }}
        />
      )}
    </div>
  );
}
