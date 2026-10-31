// Inherited right alignment on a shrink-to-fit box's rows: an absolutely placed box with text-align: right holds
// nowrap rows of `<span>K</span>&nbsp;&nbsp;<span>V</span>`, each row as wide as the widest and its line on the right.
// The two no-break spaces stay two (U+00A0 doesn't collapse).
// proves: css.properties.text-align, css.properties.white-space
export const steps = [];
const rows = [["SONG", "87.97s"], ["BEATS", "262"], ["BARS", "72"]];
export default function Case() {
  return (
    <div data-testid="root" className="relative h-full w-full bg-white">
      <div data-testid="box" style={{ position: "absolute", right: 40, top: 20, textAlign: "right", background: "#f1f5f9" }}>
        {rows.map(([k, v]) => (
          <div key={k} data-testid={`row-${k}`} style={{ fontSize: 20, lineHeight: 1.5, whiteSpace: "nowrap" }}>
            <span data-testid={`k-${k}`} style={{ color: "#b45309" }}>{k}</span>&nbsp;&nbsp;<span data-testid={`v-${k}`} style={{ fontWeight: 700 }}>{v}</span>
          </div>
        ))}
      </div>
    </div>
  );
}
