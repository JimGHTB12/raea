// requestAnimationFrame on the host's frame: callbacks asked for in one frame run together with one timestamp, a
// callback asked for inside one runs a frame later, cancelAnimationFrame drops one, and a loop counts its frames.
// proves: api.Window.requestAnimationFrame, api.Window.cancelAnimationFrame, api.Performance.now
import { useEffect, useState } from "react";
import { trace } from "parity";
export const steps = [{ click: "go" }, { wait: 20 }];
export default function Case() {
  const [frames, setFrames] = useState(0);
  const go = () => {
    let a = -1;
    requestAnimationFrame((t) => {
      a = t;
      requestAnimationFrame((t2) => trace("raf", "nested", [t2 > a, typeof t2]));
    });
    requestAnimationFrame((t) => trace("raf", "same-frame", t === a));
    const dead = requestAnimationFrame(() => trace("raf", "cancelled"));
    cancelAnimationFrame(dead);
    let n = 0;
    const loop = () => {
      n++;
      setFrames(n);
      if (n < 8) requestAnimationFrame(loop);
      else trace("raf", "loop-done", n);
    };
    requestAnimationFrame(loop);
    const t0 = performance.now();
    trace("now", "monotonic", performance.now() >= t0 && t0 > 0);
  };
  useEffect(() => trace("raf", "type", typeof requestAnimationFrame), []);
  return (
    <div data-testid="root" className="flex h-full w-full gap-2 bg-white p-4">
      <div data-testid="go" className="h-8 w-16 bg-slate-300" onClick={go} />
      <div data-testid="bar" data-value={String(frames)} className="h-8 bg-sky-400" style={{ width: 10 + frames * 20 }} />
    </div>
  );
}
