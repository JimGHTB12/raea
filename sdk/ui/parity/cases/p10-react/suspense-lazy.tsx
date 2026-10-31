// Suspense and lazy: the fallback first, then the lazy component once its promise resolves. The promise resolves on
// a click, not a timer: React holds a reveal until 300 ms after the fallback showed (its fallback throttle), so with a
// timer, whether the first read sees the fallback depends on how fast the page loaded. The step's 30 frames are past
// the throttle on both sides.
// proves: react.components.Suspense, react.api.lazy
import { lazy, Suspense } from "react";
let release: () => void = () => {};
const Late = lazy(() => new Promise<{ default: () => JSX.Element }>((r) => (release = () => r({ default: () => <div data-testid="late" className="h-12 w-48 bg-green-400" /> }))));
export const steps = [{ click: "finish", wait: 30 }];
export default function Case() {
  return (
    <div data-testid="root" className="flex h-full w-full flex-col gap-2 bg-white p-4 text-slate-900">
      <div data-testid="finish" className="h-8 w-16 bg-slate-500" onClick={() => release()} />
      <Suspense fallback={<div data-testid="fallback" className="h-12 w-24 bg-slate-300" />}><Late /></Suspense>
    </div>
  );
}
