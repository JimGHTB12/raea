// Canvas 2D text in the page's font: fillText at two sizes, strokeText, textAlign left/center/right about a line, and
// textBaseline top/middle/alphabetic/bottom; measureText's width traced. Drawn once the page's font has loaded (a
// canvas draws in whatever face is ready when it's called, so Chrome needs document.fonts.load first).
// proves: api.CanvasRenderingContext2D.fillText, api.CanvasRenderingContext2D.strokeText, api.CanvasRenderingContext2D.font, api.CanvasRenderingContext2D.textAlign, api.CanvasRenderingContext2D.textBaseline, api.CanvasRenderingContext2D.measureText
import { useEffect, useRef } from "react";
import { trace } from "parity";
export const steps = [{ wait: 4 }];
export default function Case() {
  const ref = useRef<HTMLCanvasElement>(null);
  useEffect(() => {
    const fonts = (document as unknown as { fonts?: { load(f: string): Promise<unknown> } }).fonts;
    if (fonts?.load) fonts.load('24px "Parity Mono"').then(draw);
    else draw();
  }, []);
  const draw = () => {
    const c = ref.current!.getContext("2d")!;
    c.fillStyle = "#111827";
    c.font = '24px "Parity Mono"';
    c.fillText("Hello canvas", 10, 34);
    trace("measure", "canvas", Math.round(c.measureText("Hello canvas").width));
    c.font = '14px "Parity Mono"';
    c.fillStyle = "#e11d48";
    c.fillText("small text", 10, 60);
    c.font = '28px "Parity Mono"';
    c.strokeStyle = "#2563eb";
    c.lineWidth = 1;
    c.strokeText("stroked", 260, 34);
    c.strokeStyle = "#9ca3af";
    c.beginPath();
    c.moveTo(240, 70);
    c.lineTo(240, 190);
    c.moveTo(10, 150);
    c.lineTo(470, 150);
    c.stroke();
    c.font = '16px "Parity Mono"';
    c.fillStyle = "#16a34a";
    (["left", "center", "right"] as CanvasTextAlign[]).forEach((a, i) => {
      c.textAlign = a;
      c.fillText(a, 240, 90 + i * 20);
    });
    c.textAlign = "left";
    c.fillStyle = "#7c3aed";
    (["top", "middle", "alphabetic", "bottom"] as CanvasTextBaseline[]).forEach((b, i) => {
      c.textBaseline = b;
      c.fillText(b.slice(0, 3), 20 + i * 60, 150);
    });
  };
  return (
    <div data-testid="root" className="h-full w-full bg-white p-4">
      <canvas data-testid="canvas" ref={ref} width={480} height={200} className="bg-slate-100" />
    </div>
  );
}
