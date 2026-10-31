// P18: static <iframe>s with srcdoc: the UA's default box (300x150, a 2px inset border), one styled, and one scaled
// to half inside a clipping box (docs/blitz-bevy.feature B5c). Each frame's document is laid out and drawn at its
// iframe's content box, under the element's whole transform.
// proves: html.elements.iframe, html.elements.iframe.srcdoc
export const steps = [];
const doc = `<!doctype html><html><body style="margin:0;background:#fde68a;font:16px sans-serif"><div style="margin:12px;width:120px;height:60px;background:#2563eb"></div><p style="margin:12px;color:#111">Inside the frame</p></body></html>`;
export default function Case() {
  return (
    <div data-testid="root" className="h-full w-full bg-white p-4">
      <iframe data-testid="plain" srcDoc={doc} title="plain" />
      <iframe data-testid="styled" srcDoc={doc} title="styled" className="mt-4 block h-40 w-96 rounded-lg border-4 border-slate-700" />
      <div data-testid="scaled-box" className="relative mt-4 h-24 w-96 overflow-hidden border border-slate-300">
        <iframe data-testid="scaled" srcDoc={doc} title="scaled" className="absolute top-0 left-0 h-[200%] w-[200%] origin-top-left scale-50 border-0" />
      </div>
    </div>
  );
}
