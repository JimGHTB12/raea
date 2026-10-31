// A deliberately wrong case: a press on the box traces a different value on the side under test, so behaviour must
// fail while layout and pixels pass.
// expect: fail=behaviour pass=layout,pixels
// tier: quick
import { side, trace } from "parity";
export const steps = [{ down: "box" }, { up: "box" }];
export default function Case() {
  return (
    <div data-testid="root" className="h-full w-full bg-white">
      <div data-testid="box" onPointerDown={() => trace("pointerdown", "box", side === "chrome" ? 1 : 2)} style={{ width: 120, height: 60, backgroundColor: "rgb(225, 29, 72)" }} />
    </div>
  );
}
