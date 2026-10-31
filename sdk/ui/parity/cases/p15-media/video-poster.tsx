// A video with a poster: paused, it shows the poster (amber), at the video's own size (96x72) and stretched to
// 192x144 by its classes.
// proves: html.elements.video, html.elements.video.poster, html.elements.video.src, react-dom.element.video
import { POSTER, VIDEO } from "./fixtures";
export const steps = [{ wait: 10 }];
export default function Case() {
  return (
    <div data-testid="root" className="flex h-full w-full items-start gap-4 bg-white p-4">
      <video data-testid="natural" src={VIDEO} poster={POSTER} muted preload="auto" />
      <video data-testid="sized" src={VIDEO} poster={POSTER} muted preload="auto" className="h-36 w-48" />
    </div>
  );
}
