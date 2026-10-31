// A video with no poster, paused at its start: it shows its first frame (rose) once loaded, at its own size (96x72)
// and stretched. onLoadedData is traced with its size and duration, both videos' together in document order once both
// have loaded: neither browser orders loadeddata across two elements (each decodes on its own), so the trace doesn't.
// proves: html.elements.video, html.elements.video.src, html.elements.video.muted, api.HTMLVideoElement.videoWidth, api.HTMLVideoElement.videoHeight, api.HTMLMediaElement.duration
import { trace } from "parity";
import { VIDEO } from "./fixtures";
export const steps = [{ wait: 10 }];
const ids = ["natural", "sized"];
const seen = new Map<string, number[]>();
export default function Case() {
  const loaded = (id: string) => (e: { target: EventTarget }) => {
    const v = e.target as HTMLVideoElement;
    seen.set(id, [v.videoWidth, v.videoHeight, Math.round(v.duration * 10) / 10]);
    if (seen.size === ids.length) for (const k of ids) trace("loadeddata", k, seen.get(k));
  };
  return (
    <div data-testid="root" className="flex h-full w-full items-start gap-4 bg-white p-4">
      <video data-testid="natural" src={VIDEO} muted preload="auto" onLoadedData={loaded("natural")} />
      <video data-testid="sized" src={VIDEO} muted preload="auto" className="h-36 w-48" onLoadedData={loaded("sized")} />
    </div>
  );
}
