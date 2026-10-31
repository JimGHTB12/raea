// drawImage: an Image loaded from a PNG data URL drawn at its size, scaled, and a source rectangle of it scaled into a
// destination; drawn once the image has loaded.
// proves: api.CanvasRenderingContext2D.drawImage, api.CanvasRenderingContext2D.imageSmoothingEnabled
import { useEffect, useRef } from "react";
import { trace } from "parity";
import { PNG } from "./fixtures";
export const steps = [{ wait: 6 }];
export default function Case() {
  const ref = useRef<HTMLCanvasElement>(null);
  useEffect(() => {
    const img = new Image();
    img.onload = () => {
      trace("load", "img", [img.naturalWidth, img.naturalHeight]);
      const c = ref.current!.getContext("2d")!;
      c.drawImage(img, 10, 10);
      c.drawImage(img, 120, 10, 192, 144);
      c.drawImage(img, 0, 0, 48, 36, 330, 10, 96, 72);
    };
    img.src = PNG;
  }, []);
  return (
    <div data-testid="root" className="h-full w-full bg-white p-4">
      <canvas data-testid="canvas" ref={ref} width={480} height={200} className="bg-slate-100" />
    </div>
  );
}
