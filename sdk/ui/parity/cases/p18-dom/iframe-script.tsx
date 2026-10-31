// P18: a scripted <iframe> sandboxed to allow-scripts (docs/blitz-bevy.feature B5c), its page and script beside the
// case (iframe-script.frame.ts, frame.html). The frame says hello with its own origin, "null" for a sandbox without
// allow-same-origin; the parent hears it with that origin and the iframe's contentWindow as its source, and posts a
// ping there; the frame answers. Both sides' lines are drawn, in the parent and in the frame.
// proves: api.HTMLIFrameElement.contentWindow, api.Window.parent, api.Window.postMessage, api.MessageEvent, api.MessageEvent.data, api.MessageEvent.origin, api.MessageEvent.source, html.elements.iframe.sandbox, html.elements.iframe.sandbox.allow-scripts
import { useEffect, useRef, useState } from "react";
export const steps = [{ wait: 30 }];

// heard from the first module line, before React's first render, so the frame's hello can't come first
const heard: string[] = [];
let told = () => {};
let frame: HTMLIFrameElement | null = null;
addEventListener("message", (e) => {
  const d = e.data as { hello?: boolean; origin?: string; reply?: string };
  const from = e.source === frame?.contentWindow ? "the frame" : "elsewhere";
  if (d.hello) {
    heard.push(`hello from ${from}, its origin ${d.origin}, the event's ${e.origin}`);
    frame?.contentWindow?.postMessage("ping", "*");
  } else if (d.reply) heard.push(`${d.reply} from ${from}`);
  told();
});

export default function Case() {
  const ref = useRef<HTMLIFrameElement>(null);
  const [, setN] = useState(0);
  useEffect(() => {
    frame = ref.current;
    told = () => setN((n) => n + 1);
  }, []);
  return (
    <div data-testid="root" className="h-full w-full bg-white p-4 text-sm">
      <iframe ref={ref} data-testid="frame" src="frame.html" sandbox="allow-scripts" title="frame" className="block h-24 w-96 border-2 border-slate-700" />
      <ul data-testid="heard" className="mt-4">
        {heard.map((l, i) => (
          <li key={i}>{l}</li>
        ))}
      </ul>
    </div>
  );
}
