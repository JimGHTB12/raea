// P15: a canvas read back with getImageData, as watch's histogram reads the frame on screen: rects filled, then read
// exactly (inside, across the edge, where nothing's drawn); a video's frame drawn on it before and after a seek
// (rose at its start, green at 0.5 s), its centre read; the errors a zero size or a short buffer throws. Each line is
// traced and drawn.
// proves: api.CanvasRenderingContext2D.getImageData, api.CanvasRenderingContext2D.createImageData, api.ImageData, api.ImageData.ImageData, api.ImageData.data, api.ImageData.width, api.ImageData.height
import { useEffect, useRef, useState } from "react";
import { trace } from "parity";
import { VIDEO } from "./fixtures";
export const steps = [{ wait: 30 }];

const px = (d: ImageData, x: number, y: number) => Array.from(d.data.slice((y * d.width + x) * 4, (y * d.width + x) * 4 + 4)).join(",");
// a video frame's colour, coarse: the codec's rounding differs by a step or two between decoders
const coarse = (d: ImageData, x: number, y: number) => Array.from(d.data.slice((y * d.width + x) * 4, (y * d.width + x) * 4 + 3), (v) => Math.round(v / 32) * 32).join(",");

export default function Case() {
  const c = useRef<HTMLCanvasElement>(null);
  const v = useRef<HTMLVideoElement>(null);
  const [lines, setLines] = useState<string[]>([]);
  useEffect(() => {
    const said: string[] = [];
    const say = (l: string) => {
      said.push(l);
      trace("line", "read", l);
      setLines([...said]);
    };
    const g = c.current!.getContext("2d")!;
    g.fillStyle = "rgb(200, 40, 90)";
    g.fillRect(0, 0, 20, 10);
    g.fillStyle = "rgba(0, 0, 255, 0.5)";
    g.fillRect(10, 0, 10, 10);
    const a = g.getImageData(0, 0, 4, 2);
    say(`fill: ${a.width}x${a.height}, ${a.data.length} bytes, ${px(a, 0, 0)}`);
    say(`half blue over it: ${px(g.getImageData(12, 4, 1, 1), 0, 0)}`);
    const edge = g.getImageData(-1, -1, 2, 2);
    say(`across the edge: ${px(edge, 0, 0)} | ${px(edge, 1, 1)}`);
    say(`nothing drawn: ${px(g.getImageData(30, 20, 1, 1), 0, 0)}`);
    const neg = g.getImageData(4, 4, -2, -2);
    say(`negative size: ${neg.width}x${neg.height}`);
    try {
      g.getImageData(0, 0, 0, 1);
    } catch (e) {
      say(`zero width: ${(e as Error).name}`);
    }
    const made = g.createImageData(3, 2);
    say(`made: ${made.width}x${made.height}, ${made.data.length} bytes, ${made.data.every((x) => x === 0)}`);
    try {
      new ImageData(new Uint8ClampedArray(10), 2);
    } catch (e) {
      say(`short buffer: ${(e as Error).name}`);
    }
    const video = v.current!;
    const frame = (what: string) => {
      g.drawImage(video, 0, 12, 32, 24);
      say(`${what}: ${coarse(g.getImageData(16, 24, 1, 1), 0, 0)}`);
    };
    const first = () => {
      frame("video at 0");
      video.addEventListener("seeked", () => frame("video at 0.5"), { once: true });
      video.currentTime = 0.5;
    };
    if (video.readyState >= 2) first();
    else video.addEventListener("loadeddata", first, { once: true });
  }, []);
  return (
    <div data-testid="root" className="flex h-full w-full gap-4 bg-white p-4 text-xs">
      <canvas ref={c} data-testid="canvas" width={32} height={36} className="h-36 w-32" />
      <video ref={v} src={VIDEO} muted preload="auto" className="hidden" />
      <ul data-testid="lines">
        {lines.map((l, i) => (
          <li key={i} className="mb-1">
            {l}
          </li>
        ))}
      </ul>
    </div>
  );
}
