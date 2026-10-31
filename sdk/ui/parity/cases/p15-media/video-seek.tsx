// A video seeked by a click (currentTime = 0.5 s): its picture turns from the first frame (rose) to the one at 0.5 s
// (green), and `seeked` is traced with the time it landed on.
// proves: api.HTMLMediaElement.currentTime, html.elements.video
import { useRef } from "react";
import { trace } from "parity";
import { VIDEO } from "./fixtures";
export const steps = [{ wait: 10 }, { click: "seek" }, { wait: 10 }];
export default function Case() {
  const v = useRef<HTMLVideoElement>(null);
  return (
    <div data-testid="root" className="flex h-full w-full items-start gap-4 bg-white p-4">
      <video ref={v} data-testid="video" src={VIDEO} muted preload="auto" className="h-36 w-48" onSeeked={(e) => trace("seeked", "video", Math.round((e.target as HTMLVideoElement).currentTime * 10) / 10)} />
      <button data-testid="seek" className="h-8 w-20 bg-slate-800" onClick={() => v.current && (v.current.currentTime = 0.5)} />
    </div>
  );
}
