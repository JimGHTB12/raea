// useCallback and memo: a memoized child doesn't re-render when its parent does with the same callback.
// proves: react.hooks.useCallback, react.api.memo
import { memo, useCallback, useState } from "react";
import { trace } from "parity";
export const steps = [{ click: "other" }, { click: "child" }];
const Child = memo(function Child({ onHit }: { onHit: () => void }) {
  trace("render", "child");
  return <button data-testid="child" className="bg-sky-200 px-2" onClick={onHit}>child</button>;
});
export default function Case() {
  const [n, setN] = useState(0);
  const [hits, setHits] = useState(0);
  const onHit = useCallback(() => setHits((h) => h + 1), []);
  return (
    <div data-testid="root" className="h-full w-full bg-white p-4 text-slate-900">
      <button data-testid="other" className="mr-2 bg-slate-200 px-2" onClick={() => setN(n + 1)}>other {n}</button>
      <Child onHit={onHit} />
      <span data-testid="hits" data-value={String(hits)} className="ml-2">{hits}</span>
    </div>
  );
}
