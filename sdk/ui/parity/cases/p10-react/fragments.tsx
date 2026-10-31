// Fragments: children returned as a fragment lay out as their parent's own children.
// proves: react.components.Fragment
import { Fragment } from "react";
function Pair({ n }: { n: number }) {
  return (<><div data-testid={"a" + n} className="h-8 w-8 bg-rose-400" /><div data-testid={"b" + n} className="h-8 w-8 bg-sky-400" /></>);
}
export const steps = [];
export default function Case() {
  return (
    <div data-testid="root" className="h-full w-full bg-white p-4 text-slate-900">
      <div data-testid="row" className="flex gap-2">
        <Pair n={1} />
        <Fragment key="x"><Pair n={2} /></Fragment>
      </div>
    </div>
  );
}
