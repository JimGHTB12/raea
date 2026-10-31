// A deliberately wrong case: the box is 2 pt wider on the side under test, so layout must fail (and nothing else is
// different in behaviour).
// expect: fail=layout pass=behaviour
// tier: quick
import { side } from "parity";
export const steps = [];
export default function Case() {
  return (
    <div data-testid="root" className="h-full w-full bg-white">
      <div data-testid="box" style={{ width: side === "chrome" ? 120 : 122, height: 60, backgroundColor: "rgb(225, 29, 72)" }} />
    </div>
  );
}
