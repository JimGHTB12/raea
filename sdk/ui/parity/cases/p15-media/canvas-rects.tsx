// Canvas 2D rectangles: fillRect in three colours, strokeRect, and clearRect cutting a hole; drawn in an effect on
// mount, in a 300x150 canvas (its default size) shown at that size.
// proves: html.elements.canvas, api.HTMLCanvasElement.getContext, api.HTMLCanvasElement.getContext.2d_context, api.CanvasRenderingContext2D.fillRect, api.CanvasRenderingContext2D.strokeRect, api.CanvasRenderingContext2D.clearRect, api.CanvasRenderingContext2D.fillStyle, api.CanvasRenderingContext2D.strokeStyle, api.CanvasRenderingContext2D.lineWidth
import { useEffect, useRef } from "react";
export const steps = [];
export default function Case() {
  const ref = useRef<HTMLCanvasElement>(null);
  useEffect(() => {
    const c = ref.current!.getContext("2d")!;
    c.fillStyle = "#e11d48";
    c.fillRect(10, 10, 100, 60);
    c.fillStyle = "rgb(22, 163, 74)";
    c.fillRect(60, 40, 100, 60);
    c.fillStyle = "rgba(37, 99, 235, 0.5)";
    c.fillRect(110, 70, 100, 60);
    c.strokeStyle = "#111827";
    c.lineWidth = 4;
    c.strokeRect(220, 20, 60, 60);
    c.clearRect(30, 20, 30, 30);
  }, []);
  return (
    <div data-testid="root" className="h-full w-full bg-white p-4">
      <canvas data-testid="canvas" ref={ref} className="bg-slate-200" />
    </div>
  );
}
