// Canvas 2D gradients: a linear gradient with three stops, a radial gradient between two circles, a conic gradient,
// and a gradient as a stroke.
// proves: api.CanvasRenderingContext2D.createLinearGradient, api.CanvasRenderingContext2D.createRadialGradient, api.CanvasRenderingContext2D.createConicGradient
import { useEffect, useRef } from "react";
export const steps = [];
export default function Case() {
  const ref = useRef<HTMLCanvasElement>(null);
  useEffect(() => {
    const c = ref.current!.getContext("2d")!;
    const l = c.createLinearGradient(10, 0, 150, 0);
    l.addColorStop(0, "#e11d48");
    l.addColorStop(0.5, "#facc15");
    l.addColorStop(1, "#2563eb");
    c.fillStyle = l;
    c.fillRect(10, 10, 140, 80);
    const r = c.createRadialGradient(230, 50, 5, 240, 55, 50);
    r.addColorStop(0, "#ffffff");
    r.addColorStop(1, "#16a34a");
    c.fillStyle = r;
    c.fillRect(180, 0, 120, 110);
    const k = c.createConicGradient(0, 380, 55);
    k.addColorStop(0, "#7c3aed");
    k.addColorStop(0.5, "#0891b2");
    k.addColorStop(1, "#7c3aed");
    c.fillStyle = k;
    c.beginPath();
    c.arc(380, 55, 45, 0, Math.PI * 2);
    c.fill();
    const s = c.createLinearGradient(0, 130, 0, 190);
    s.addColorStop(0, "#111827");
    s.addColorStop(1, "#f59e0b");
    c.strokeStyle = s;
    c.lineWidth = 10;
    c.strokeRect(20, 130, 400, 50);
  }, []);
  return (
    <div data-testid="root" className="h-full w-full bg-white p-4">
      <canvas data-testid="canvas" ref={ref} width={480} height={200} className="bg-slate-100" />
    </div>
  );
}
