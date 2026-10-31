// ctx.filter: the same striped tile drawn under each filter function (blur, brightness, contrast, grayscale,
// hue-rotate, invert, opacity, saturate, sepia, drop-shadow) and a list of three, with a translate and a scale in
// force (a filter's lengths are the bitmap's pixels, untouched by them), an invalid value ignored (the last valid one
// kept) and save()/restore() carrying it; the label text is drawn unfiltered after the last restore().
// proves: api.CanvasRenderingContext2D.filter
import { useEffect, useRef } from "react";
export const steps = [];
const FILTERS = [
  "blur(3px)",
  "brightness(160%)",
  "contrast(0.4)",
  "grayscale(1)",
  "hue-rotate(120deg)",
  "invert(0.8)",
  "opacity(40%)",
  "saturate(3)",
  "sepia(100%)",
  "drop-shadow(6px 4px 4px rgba(15, 23, 42, 0.7))",
  "sepia(0.6) hue-rotate(0.5turn) blur(1px)",
];
export default function Case() {
  const ref = useRef<HTMLCanvasElement>(null);
  useEffect(() => {
    const c = ref.current!.getContext("2d")!;
    const tile = (x: number, y: number) => {
      c.fillStyle = "#e11d48";
      c.fillRect(x, y, 60, 60);
      c.fillStyle = "#facc15";
      c.fillRect(x + 10, y + 10, 40, 14);
      c.fillStyle = "#2563eb";
      c.beginPath();
      c.arc(x + 30, y + 42, 12, 0, Math.PI * 2);
      c.fill();
    };
    FILTERS.forEach((f, i) => {
      c.save();
      c.filter = f;
      c.filter = "blur(nope)";
      tile(16 + (i % 6) * 84, 16 + Math.floor(i / 6) * 96);
      c.restore();
    });
    c.save();
    c.translate(520, 112);
    c.scale(1.5, 1.5);
    c.filter = "blur(2px)";
    tile(0, 0);
    c.restore();
    c.fillStyle = "#0f172a";
    c.font = "14px sans-serif";
    c.fillText(`after restore: ${c.filter}`, 16, 236);
  }, []);
  return (
    <div data-testid="root" className="h-full w-full bg-white p-4">
      <canvas data-testid="canvas" ref={ref} width={640} height={250} className="bg-slate-100" />
    </div>
  );
}
