// A deliberately wrong case: the box is another colour on the side under test, so pixels must fail while layout and
// behaviour pass.
// expect: fail=pixels pass=layout,behaviour
// tier: quick
import { side } from "parity";
export const steps = [];
export default function Case() {
  return (
    <div data-testid="root" className="h-full w-full bg-white">
      <div data-testid="box" style={{ width: 120, height: 60, backgroundColor: side === "chrome" ? "rgb(225, 29, 72)" : "rgb(29, 72, 225)" }} />
    </div>
  );
}
