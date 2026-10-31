// P18: an <iframe> with a src (a data: URL, loaded through the page's net provider) and a rotated srcdoc one
// (docs/blitz-bevy.feature B5c).
// proves: html.elements.iframe, html.elements.iframe.src, html.elements.iframe.srcdoc
export const steps = [];
const html = `<!doctype html><body style="margin:0;background:#bbf7d0"><div style="margin:8px;width:80px;height:40px;background:#dc2626;border-radius:8px"></div><ul style="font:14px sans-serif;color:#14532d"><li>one</li><li>two</li></ul></body>`;
export default function Case() {
  return (
    <div data-testid="root" className="h-full w-full bg-white p-4">
      <iframe data-testid="src" src={"data:text/html," + encodeURIComponent(html)} title="src" className="block h-32 w-72 border-2 border-emerald-800" />
      <iframe data-testid="rotated" srcDoc={html} title="rotated" className="mt-10 ml-10 block h-32 w-72 rotate-6 border-0" />
    </div>
  );
}
