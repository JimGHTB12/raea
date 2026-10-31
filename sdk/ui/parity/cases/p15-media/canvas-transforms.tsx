// Canvas 2D transforms: translate, rotate and scale, nested with save/restore, setTransform and resetTransform, and
// the transform a path was built under (not the one it's filled under) placing it.
// proves: api.CanvasRenderingContext2D.translate, api.CanvasRenderingContext2D.rotate, api.CanvasRenderingContext2D.scale, api.CanvasRenderingContext2D.save, api.CanvasRenderingContext2D.restore, api.CanvasRenderingContext2D.setTransform, api.CanvasRenderingContext2D.resetTransform, api.CanvasRenderingContext2D.transform
import { useEffect, useRef } from "react";
export const steps = [];
export default function Case() {
  const ref = useRef<HTMLCanvasElement>(null);
  useEffect(() => {
    const c = ref.current!.getContext("2d")!;
    c.save();
    c.translate(80, 80);
    for (let i = 0; i < 6; i++) {
      c.fillStyle = `hsl(${i * 60}, 70%, 50%)`;
      c.fillRect(0, -6, 60, 12);
      c.rotate(Math.PI / 3);
    }
    c.restore();
    c.save();
    c.translate(200, 30);
    c.scale(2, 1);
    c.fillStyle = "#111827";
    c.fillRect(0, 0, 40, 40);
    c.restore();
    c.setTransform(1, 0.3, 0, 1, 300, 20);
    c.fillStyle = "#2563eb";
    c.fillRect(0, 0, 60, 60);
    c.resetTransform();
    c.transform(1, 0, 0, 1, 400, 40);
    c.fillStyle = "#16a34a";
    c.fillRect(0, 0, 40, 40);
    c.resetTransform();
    c.beginPath();
    c.translate(60, 150);
    c.rect(0, 0, 50, 30);
    c.resetTransform();
    c.fillStyle = "#e11d48";
    c.fill();
  }, []);
  return (
    <div data-testid="root" className="h-full w-full bg-white p-4">
      <canvas data-testid="canvas" ref={ref} width={480} height={200} className="bg-slate-100" />
    </div>
  );
}
