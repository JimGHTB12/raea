// A canvas redrawn on a click (clearRect, then the next colour's square further right), and its size changed by state
// (setting width clears the bitmap).
// proves: api.CanvasRenderingContext2D.clearRect, api.CanvasRenderingContext2D.fillRect, html.elements.canvas
import { useEffect, useRef, useState } from "react";
export const steps = [{ click: "go" }, { click: "go" }];
const COLORS = ["#e11d48", "#16a34a", "#2563eb"];
export default function Case() {
  const ref = useRef<HTMLCanvasElement>(null);
  const [n, setN] = useState(0);
  useEffect(() => {
    const el = ref.current!;
    const c = el.getContext("2d")!;
    c.clearRect(0, 0, el.width, el.height);
    c.fillStyle = COLORS[n % 3];
    c.fillRect(20 + n * 60, 20, 50, 50);
  }, [n]);
  return (
    <div data-testid="root" className="flex h-full w-full items-start gap-4 bg-white p-4">
      <canvas data-testid="canvas" ref={ref} width={n === 2 ? 300 : 240} height={100} className="bg-slate-200" />
      <button data-testid="go" className="h-8 w-20 bg-slate-800" onClick={() => setN(n + 1)} />
    </div>
  );
}
