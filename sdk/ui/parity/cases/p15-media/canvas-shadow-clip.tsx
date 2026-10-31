// Shadows (shadowColor, shadowBlur, shadowOffset) under a filled rect, a clip() to a circle with stripes drawn through
// it, and globalCompositeOperation destination-out punching a hole.
// proves: api.CanvasRenderingContext2D.shadowBlur, api.CanvasRenderingContext2D.shadowColor, api.CanvasRenderingContext2D.shadowOffsetX, api.CanvasRenderingContext2D.shadowOffsetY, api.CanvasRenderingContext2D.clip, api.CanvasRenderingContext2D.globalCompositeOperation
import { useEffect, useRef } from "react";
export const steps = [];
export default function Case() {
  const ref = useRef<HTMLCanvasElement>(null);
  useEffect(() => {
    const c = ref.current!.getContext("2d")!;
    c.save();
    c.shadowColor = "rgba(17, 24, 39, 0.6)";
    c.shadowBlur = 10;
    c.shadowOffsetX = 6;
    c.shadowOffsetY = 8;
    c.fillStyle = "#facc15";
    c.fillRect(20, 20, 120, 80);
    c.restore();
    c.save();
    c.beginPath();
    c.arc(250, 70, 55, 0, Math.PI * 2);
    c.clip();
    for (let i = 0; i < 12; i++) {
      c.fillStyle = i % 2 ? "#e11d48" : "#2563eb";
      c.fillRect(190 + i * 10, 0, 10, 200);
    }
    c.restore();
    c.fillStyle = "#16a34a";
    c.fillRect(340, 20, 120, 120);
    c.globalCompositeOperation = "destination-out";
    c.beginPath();
    c.arc(400, 80, 30, 0, Math.PI * 2);
    c.fill();
    c.globalCompositeOperation = "source-over";
  }, []);
  return (
    <div data-testid="root" className="h-full w-full bg-white p-4">
      <canvas data-testid="canvas" ref={ref} width={480} height={200} className="bg-slate-100" />
    </div>
  );
}
