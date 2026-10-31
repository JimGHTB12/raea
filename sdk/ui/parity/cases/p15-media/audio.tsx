// An audio element without controls has no box (it's display: none), and is still an element with its media API:
// paused at the start, muted as its prop says, its duration once the metadata has loaded (traced).
// proves: html.elements.audio, html.elements.audio.src, react-dom.element.audio, api.HTMLMediaElement.paused, api.HTMLMediaElement.muted, api.HTMLMediaElement.duration
import { useRef } from "react";
import { trace } from "parity";
import { AUDIO } from "./fixtures";
export const steps = [{ wait: 10 }, { click: "read" }];
export default function Case() {
  const a = useRef<HTMLAudioElement>(null);
  return (
    <div data-testid="root" className="flex h-full w-full items-start gap-4 bg-white p-4">
      <audio ref={a} data-testid="audio" src={AUDIO} muted preload="auto" onLoadedMetadata={(e) => trace("loadedmetadata", "audio", Math.round((e.target as HTMLAudioElement).duration * 10) / 10)} />
      <button data-testid="read" className="h-8 w-20 bg-slate-800" onClick={() => trace("state", "audio", [a.current?.paused, a.current?.muted, a.current?.currentTime])} />
    </div>
  );
}
