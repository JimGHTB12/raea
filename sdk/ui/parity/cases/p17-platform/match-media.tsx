// matchMedia: what the queries match at 800x600, a change event (and the old addListener) as the viewport narrows past
// a breakpoint, and a component that follows it with useSyncExternalStore.
// proves: api.Window.matchMedia, api.MediaQueryList, api.MediaQueryList.matches, api.MediaQueryList.media, api.MediaQueryList.EventTarget_inheritance
import { useEffect, useSyncExternalStore } from "react";
import { trace } from "parity";
let mql: MediaQueryList | null = null;
const wide = () => (mql ??= matchMedia("(min-width: 700px)"));
const sub = (f: () => void) => {
  wide().addEventListener("change", f);
  return () => wide().removeEventListener("change", f);
};
export const steps = [{ resize: [600, 500] }, { wait: 3 }, { resize: [800, 600] }, { wait: 3 }];
export default function Case() {
  const isWide = useSyncExternalStore(sub, () => wide().matches);
  useEffect(() => {
    for (const q of ["(min-width: 700px)", "(max-width: 700px)", "screen and (orientation: landscape)", "(prefers-color-scheme: dark)", "(prefers-reduced-motion: reduce)", "(hover: hover)", "(pointer: fine)", "print", "not print", "(min-width: 30em)"])
      trace("query", q, matchMedia(q).matches);
    trace("mql", "shape", [wide().media, wide() instanceof EventTarget, typeof wide().addListener]);
    const old = (e: MediaQueryListEvent) => trace("listener", "old", [e.matches, e.media, innerWidth]);
    wide().addListener(old);
    return () => wide().removeListener(old);
  }, []);
  return (
    <div data-testid="root" className="h-full w-full bg-white p-4">
      <div data-testid="box" data-value={String(isWide)} className={isWide ? "h-10 w-96 bg-sky-400" : "h-10 w-40 bg-orange-400"} />
    </div>
  );
}
