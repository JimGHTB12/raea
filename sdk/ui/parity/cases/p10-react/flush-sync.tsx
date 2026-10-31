// flushSync: state committed synchronously inside a handler, so a later read in the same handler sees the new DOM's
// state (here: a second setState based on the first, traced in order).
// proves: react-dom.flushSync
import { useState } from "react";
import { flushSync } from "react-dom";
import { trace } from "parity";
export const steps = [{ click: "go" }];
export default function Case() {
  const [n, setN] = useState(0);
  return (
    <div data-testid="root" className="h-full w-full bg-white p-4 text-slate-900">
      <button data-testid="go" className="bg-slate-200 px-2" onClick={() => { flushSync(() => setN(1)); trace("after-flush", "go"); setN((x) => x + 1); }}>go</button>
      <span data-testid="n" data-value={String(n)} className="ml-2">{n}</span>
    </div>
  );
}
