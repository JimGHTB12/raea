// A phone-edit caption and title card as the films write them: heavy type outlined by sixteen hard text-shadows in a
// ring plus a soft drop (phone-edits/lib.tsx stroke(6)), scaled, and a white card turned -3deg with a deep box-shadow
// (unboxing-310's "TRY IT ALL KIT"). Edges of rotated and scaled glyphs, and many stacked shadows.
// proves: css.properties.text-shadow, css.properties.transform, css.properties.box-shadow
export const steps = [];
const STROKE = "6.0px 0.0px 0 #000, 5.5px 2.3px 0 #000, 4.2px 4.2px 0 #000, 2.3px 5.5px 0 #000, 0.0px 6.0px 0 #000, -2.3px 5.5px 0 #000, -4.2px 4.2px 0 #000, -5.5px 2.3px 0 #000, -6.0px 0.0px 0 #000, -5.5px -2.3px 0 #000, -4.2px -4.2px 0 #000, -2.3px -5.5px 0 #000, -0.0px -6.0px 0 #000, 2.3px -5.5px 0 #000, 4.2px -4.2px 0 #000, 5.5px -2.3px 0 #000, 0 8px 24px rgba(0,0,0,0.45)";
export default function Case() {
  return (
    <div data-testid="root" className="relative h-full w-full" style={{ background: "#7a5a3a", fontFamily: "Arial Black, Helvetica", fontWeight: 900 }}>
      <div data-testid="card" className="absolute" style={{ left: 40, top: 40, backgroundColor: "#fff", padding: "14px 34px", borderRadius: 12, transform: "rotate(-3deg) scale(1.02)", boxShadow: "0 16px 40px rgba(0,0,0,0.35)" }}>
        <div data-testid="title" style={{ fontSize: 64, color: "#e8202a", whiteSpace: "nowrap" }}>TRY IT ALL KIT</div>
      </div>
      <div data-testid="caption" className="absolute" style={{ left: 60, top: 300, width: 680, fontSize: 60, lineHeight: 1.05, color: "#fff", textAlign: "center", textTransform: "uppercase", textShadow: STROKE, transform: "scale(1.03)" }}>
        try it all <span style={{ color: "#ffd21f" }}>kit</span>
      </div>
    </div>
  );
}
