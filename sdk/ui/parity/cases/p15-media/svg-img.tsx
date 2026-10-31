// A whole svg document as an img's source (data:image/svg+xml): drawn at its own size (its width and height), and
// scaled to 240x120 by the img's classes (drawn at that size, not a bitmap scaled up).
// proves: html.elements.img, svg.elements.svg, svg.elements.linearGradient
export const steps = [];
const doc = `<svg xmlns="http://www.w3.org/2000/svg" width="120" height="60" viewBox="0 0 120 60"><defs><linearGradient id="g"><stop offset="0" stop-color="#e11d48"/><stop offset="1" stop-color="#2563eb"/></linearGradient></defs><rect width="120" height="60" rx="10" fill="url(#g)"/><circle cx="30" cy="30" r="18" fill="#facc15" stroke="#111827" stroke-width="3"/></svg>`;
const src = `data:image/svg+xml,${encodeURIComponent(doc)}`;
export default function Case() {
  return (
    <div data-testid="root" className="flex h-full w-full items-start gap-4 bg-white p-4">
      <img data-testid="natural" src={src} alt="" />
      <img data-testid="sized" src={src} alt="" className="h-[120px] w-[240px]" />
    </div>
  );
}
