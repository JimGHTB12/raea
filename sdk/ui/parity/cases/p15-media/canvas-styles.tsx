// Canvas 2D stroke styles and alpha: lineCap butt/round/square, lineJoin miter/round/bevel, setLineDash with an
// offset, a miter limit, globalAlpha.
// proves: api.CanvasRenderingContext2D.lineCap, api.CanvasRenderingContext2D.lineJoin, api.CanvasRenderingContext2D.setLineDash, api.CanvasRenderingContext2D.lineDashOffset, api.CanvasRenderingContext2D.miterLimit, api.CanvasRenderingContext2D.globalAlpha
import { useEffect, useRef } from "react";
export const steps = [];
export default function Case() {
  const ref = useRef<HTMLCanvasElement>(null);
  useEffect(() => {
    const c = ref.current!.getContext("2d")!;
    c.strokeStyle = "#111827";
    c.lineWidth = 12;
    (["butt", "round", "square"] as CanvasLineCap[]).forEach((cap, i) => {
      c.lineCap = cap;
      c.beginPath();
      c.moveTo(20, 20 + i * 30);
      c.lineTo(140, 20 + i * 30);
      c.stroke();
    });
    c.lineCap = "butt";
    (["miter", "round", "bevel"] as CanvasLineJoin[]).forEach((join, i) => {
      c.lineJoin = join;
      c.beginPath();
      c.moveTo(180 + i * 80, 90);
      c.lineTo(210 + i * 80, 20);
      c.lineTo(240 + i * 80, 90);
      c.stroke();
    });
    c.lineJoin = "miter";
    c.miterLimit = 2;
    c.beginPath();
    c.moveTo(420, 90);
    c.lineTo(440, 20);
    c.lineTo(460, 90);
    c.stroke();
    c.lineWidth = 4;
    c.setLineDash([12, 6]);
    c.lineDashOffset = 3;
    c.strokeStyle = "#2563eb";
    c.strokeRect(20, 120, 200, 60);
    c.setLineDash([]);
    c.globalAlpha = 0.4;
    c.fillStyle = "#e11d48";
    c.fillRect(260, 120, 120, 60);
    c.fillStyle = "#16a34a";
    c.fillRect(320, 140, 120, 50);
  }, []);
  return (
    <div data-testid="root" className="h-full w-full bg-white p-4">
      <canvas data-testid="canvas" ref={ref} width={480} height={200} className="bg-slate-100" />
    </div>
  );
}
