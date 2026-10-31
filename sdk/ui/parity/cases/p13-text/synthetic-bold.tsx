// Synthetic bold as Chrome draws it on macOS: the page's one-face mono (its only face 500) at 500 and at 600 or more,
// at 16, 24, 32 and 44 px (Chrome thickens it from 24 px up, not at 16), and Helvetica, whose bold is a face of its
// own and so never synthesized, at 400 and 700.
// proves: css.properties.font-weight, css.properties.font-synthesis
export const steps = [];
const ROWS: [number, number][] = [[16, 500], [16, 600], [24, 500], [24, 600], [32, 500], [32, 700], [44, 500], [44, 900]];
export default function Case() {
  return (
    <div data-testid="root" className="relative h-full w-full bg-white">
      {ROWS.map(([size, weight], i) => (
        <div key={i} style={{ position: "absolute", left: 20, top: 8 + i * 60, fontSize: size, fontWeight: weight, color: "#0f172a", lineHeight: "56px", whiteSpace: "nowrap" }}>
          mono {weight} at {size}
        </div>
      ))}
      <div style={{ position: "absolute", left: 460, top: 8, fontFamily: "Helvetica", fontSize: 32, color: "#0f172a", lineHeight: "56px" }}>Helvetica 400</div>
      <div style={{ position: "absolute", left: 460, top: 68, fontFamily: "Helvetica", fontSize: 32, fontWeight: 700, color: "#0f172a", lineHeight: "56px" }}>Helvetica 700</div>
    </div>
  );
}
