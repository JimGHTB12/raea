// The element API: Children (map, count, toArray, only, forEach), cloneElement (props merged, key and ref kept),
// isValidElement and createElement, as a layout and a trace.
// proves: react.components.Children, react.api.cloneElement, react.api.isValidElement, react.api.createElement
import { Children, cloneElement, createElement, Fragment, isValidElement, useState, type ReactElement, type ReactNode } from "react";
import { trace } from "parity";
function Stack({ children }: { children: ReactNode }) {
  trace("children", "stack", { count: Children.count(children), keys: Children.toArray(children).map((c) => (isValidElement(c) ? c.key : typeof c)) });
  return <div data-testid="stack" className="flex gap-1">{Children.map(children, (c, i) => (isValidElement(c) ? cloneElement(c as ReactElement<{ className?: string; "data-testid"?: string }>, { "data-testid": "kid-" + i, className: ((c.props as { className?: string }).className ?? "") + (i % 2 ? " bg-sky-300" : " bg-orange-300") }) : c))}</div>;
}
function Only({ children }: { children: ReactNode }) {
  const one = Children.only(children);
  return cloneElement(one as ReactElement<{ "data-value"?: string }>, { "data-value": "only" });
}
export const steps = [{ click: "more" }];
export default function Case() {
  const [n, setN] = useState(2);
  const items = Array.from({ length: n }, (_, i) => <div key={"k" + i} className="h-8 w-10" />);
  let seen = 0;
  Children.forEach([items, "text", null, <Fragment key="f"><span /></Fragment>], () => seen++);
  trace("forEach", "case", seen);
  return (
    <div data-testid="root" className="h-full w-full bg-white p-4 text-slate-900">
      <button data-testid="more" className="mb-2 bg-slate-200 px-2" onClick={() => setN(n + 2)}>more</button>
      <Stack>{items}{false}{null}</Stack>
      <Only><div data-testid="one" className="mt-2 h-6 w-16 bg-slate-300" /></Only>
      {createElement("div", { "data-testid": "made", className: "mt-2 h-6 w-20 bg-lime-300" }, "made")}
    </div>
  );
}
