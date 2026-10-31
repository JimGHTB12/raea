// useImperativeHandle with a ref: the parent calls the child's own method.
// proves: react.hooks.useImperativeHandle, react.api.forwardRef
import { forwardRef, useImperativeHandle, useRef, useState } from "react";
type H = { grow(): void };
const Box = forwardRef<H>(function Box(_, ref) {
  const [w, setW] = useState(40);
  useImperativeHandle(ref, () => ({ grow: () => setW((x) => x + 40) }), []);
  return <div data-testid="box" data-value={String(w)} className="mt-2 h-8 bg-fuchsia-400" style={{ width: w }} />;
});
export const steps = [{ click: "grow" }, { click: "grow" }];
export default function Case() {
  const r = useRef<H>(null);
  return (
    <div data-testid="root" className="h-full w-full bg-white p-4 text-slate-900">
      <button data-testid="grow" className="bg-slate-200 px-2" onClick={() => r.current?.grow()}>grow</button>
      <Box ref={r} />
    </div>
  );
}
