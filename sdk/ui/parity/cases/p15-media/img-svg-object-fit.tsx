// An SVG picture with a natural size (width and height, no viewBox: 160x90, a blue field and an amber disc in its
// middle) under object-fit in 120x120 boxes: fill, contain and cover scale the drawing from its natural size to the fit's
// rect, none draws it at its own size, centred and clipped. A natural-size SVG image without a viewBox drew unscaled and
// cropped to its top-left corner (blitz-paint draw_svg, docs/blitz-bevy.feature H5), so cover lost the disc.
// proves: html.elements.img, html.elements.img.src, css.properties.object-fit, css.properties.object-fit.cover, css.properties.object-fit.contain
export const steps = [];
const SVG = "data:image/svg+xml," + encodeURIComponent('<svg xmlns="http://www.w3.org/2000/svg" width="160" height="90"><rect width="160" height="90" fill="#2563eb"/><circle cx="80" cy="45" r="30" fill="#f59e0b"/></svg>');
const fits = ["fill", "contain", "cover", "none"] as const;
export default function Case() {
  return (
    <div data-testid="root" style={{ display: "flex", alignItems: "flex-start", gap: 12, padding: 12, width: "100%", height: "100%", boxSizing: "border-box", background: "white" }}>
      {fits.map((f) => (
        <img key={f} data-testid={f} src={SVG} alt="" style={{ width: 120, height: 120, objectFit: f, background: "#cbd5e1" }} />
      ))}
    </div>
  );
}
