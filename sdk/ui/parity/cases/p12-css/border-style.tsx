// border-style dashed and dotted, from the border shorthand, borderStyle and Tailwind's border-dashed/border-dotted, at
// several widths, square and rounded, as the editors draw drop zones and the games draw courts.
// proves: css.properties.border-style
export const steps = [];
export default function Case() {
  return (
    <div data-testid="root" className="relative h-full w-full" style={{ background: "#0b0d14" }}>
      <div data-testid="dash1" className="absolute" style={{ left: 40, top: 40, width: 200, height: 120, border: "1px dashed #8aa4ff" }} />
      <div data-testid="dash4" className="absolute" style={{ left: 280, top: 40, width: 200, height: 120, border: "4px dashed rgba(255, 255, 255, 0.6)" }} />
      <div data-testid="drop" className="absolute rounded-xl border-2 border-dashed border-sky-400" style={{ left: 520, top: 40, width: 240, height: 120 }} />
      <div data-testid="dot2" className="absolute border-2 border-dotted border-amber-300" style={{ left: 40, top: 200, width: 200, height: 120 }} />
      <div data-testid="dot6" className="absolute" style={{ left: 280, top: 200, width: 200, height: 120, borderWidth: 6, borderStyle: "dotted", borderColor: "#f472b6" }} />
      <div data-testid="pill" className="absolute rounded-full border-4 border-dashed border-emerald-400 bg-white/10" style={{ left: 520, top: 220, width: 240, height: 80 }} />
      <div data-testid="light" className="absolute bg-white" style={{ left: 0, top: 360, width: 800, height: 240 }}>
        <div data-testid="court" className="absolute" style={{ left: 60, top: 40, width: 300, height: 160, border: "3px dashed #111827" }} />
        <div data-testid="side" className="absolute" style={{ left: 420, top: 40, width: 300, height: 160, borderLeft: "2px dotted #2563eb", borderBottom: "2px dotted #2563eb" }} />
      </div>
    </div>
  );
}
