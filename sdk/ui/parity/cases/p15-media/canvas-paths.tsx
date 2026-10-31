// Canvas 2D paths: lines, an arc and a full circle, quadratic and bezier curves, rect() and roundRect(), closePath,
// filled (nonzero and evenodd) and stroked.
// proves: api.CanvasRenderingContext2D.beginPath, api.CanvasRenderingContext2D.moveTo, api.CanvasRenderingContext2D.lineTo, api.CanvasRenderingContext2D.arc, api.CanvasRenderingContext2D.arcTo, api.CanvasRenderingContext2D.ellipse, api.CanvasRenderingContext2D.quadraticCurveTo, api.CanvasRenderingContext2D.bezierCurveTo, api.CanvasRenderingContext2D.rect, api.CanvasRenderingContext2D.roundRect, api.CanvasRenderingContext2D.closePath, api.CanvasRenderingContext2D.fill, api.CanvasRenderingContext2D.stroke
import { useEffect, useRef } from "react";
export const steps = [];
export default function Case() {
  const ref = useRef<HTMLCanvasElement>(null);
  useEffect(() => {
    const c = ref.current!.getContext("2d")!;
    c.lineWidth = 3;
    c.strokeStyle = "#111827";
    c.beginPath();
    c.moveTo(10, 10);
    c.lineTo(110, 10);
    c.lineTo(60, 80);
    c.closePath();
    c.fillStyle = "#e11d48";
    c.fill();
    c.stroke();
    c.beginPath();
    c.arc(170, 50, 40, 0, Math.PI * 2);
    c.fillStyle = "#16a34a";
    c.fill();
    c.beginPath();
    c.arc(260, 50, 35, Math.PI * 0.25, Math.PI * 1.5, false);
    c.stroke();
    c.beginPath();
    c.moveTo(320, 10);
    c.quadraticCurveTo(380, 90, 440, 10);
    c.stroke();
    c.beginPath();
    c.moveTo(10, 120);
    c.bezierCurveTo(60, 60, 110, 200, 160, 120);
    c.strokeStyle = "#2563eb";
    c.lineWidth = 5;
    c.stroke();
    c.beginPath();
    c.rect(180, 100, 80, 80);
    c.rect(200, 120, 40, 40);
    c.fillStyle = "#7c3aed";
    c.fill("evenodd");
    c.beginPath();
    c.roundRect(280, 100, 90, 70, 16);
    c.fillStyle = "#f59e0b";
    c.fill();
    c.beginPath();
    c.ellipse(420, 140, 30, 15, Math.PI / 6, 0, Math.PI * 2);
    c.moveTo(380, 100);
    c.arcTo(460, 100, 460, 180, 20);
    c.strokeStyle = "#0891b2";
    c.lineWidth = 3;
    c.stroke();
  }, []);
  return (
    <div data-testid="root" className="h-full w-full bg-white p-4">
      <canvas data-testid="canvas" ref={ref} width={480} height={200} className="bg-slate-100" />
    </div>
  );
}
