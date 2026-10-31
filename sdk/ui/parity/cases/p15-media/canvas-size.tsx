// A canvas's two sizes: its bitmap (the width and height attributes) and its box. Default (300x150), a 100x50 bitmap
// shown at 200x100 (each pixel twice as big, smoothed), and a 200x100 bitmap shown at 100x50.
// proves: html.elements.canvas, api.HTMLCanvasElement.getContext
import { useEffect, useRef } from "react";
export const steps = [];
function draw(el: HTMLCanvasElement | null) {
  const c = el?.getContext("2d");
  if (!c || !el) return;
  c.fillStyle = "#2563eb";
  c.fillRect(0, 0, el.width / 2, el.height / 2);
  c.fillStyle = "#e11d48";
  c.beginPath();
  c.arc(el.width * 0.7, el.height * 0.6, el.height * 0.3, 0, Math.PI * 2);
  c.fill();
}
export default function Case() {
  const a = useRef<HTMLCanvasElement>(null);
  const b = useRef<HTMLCanvasElement>(null);
  const c = useRef<HTMLCanvasElement>(null);
  useEffect(() => [a, b, c].forEach((r) => draw(r.current)), []);
  return (
    <div data-testid="root" className="flex h-full w-full flex-wrap items-start gap-4 bg-white p-4">
      <canvas data-testid="default" ref={a} className="bg-slate-200" />
      <canvas data-testid="up" ref={b} width={100} height={50} className="h-[100px] w-[200px] bg-slate-200" />
      <canvas data-testid="down" ref={c} width={200} height={100} className="h-[50px] w-[100px] bg-slate-200" />
    </div>
  );
}
