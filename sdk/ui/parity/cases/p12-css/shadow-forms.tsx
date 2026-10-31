// box-shadow past the plain one: inset, several at once, spread (positive and negative), on rounded and pill
// boxes, a coloured glow, and box-shadow: none; text-shadow: none beside a stack of them.
// proves: css.properties.box-shadow.inset, css.properties.box-shadow.multiple_shadows, css.properties.box-shadow.spread_radius, css.properties.box-shadow.none, css.properties.text-shadow.none
export const steps = [];
const SHADOWS: [string, string][] = [
  ["inset", "inset 0 6px 14px rgba(0, 0, 0, 0.45)"],
  ["inset-spread", "inset 0 0 0 10px #f59e0b"],
  ["multi", "0 1px 2px rgba(0,0,0,0.2), 0 8px 16px rgba(0,0,0,0.2), 0 24px 48px rgba(0,0,0,0.25)"],
  ["spread", "0 0 0 8px #10b981"],
  ["neg-spread", "0 18px 12px -10px rgba(0, 0, 0, 0.6)"],
  ["glow", "0 0 24px 6px rgba(236, 72, 153, 0.75)"],
  ["both", "inset 0 0 12px #1d4ed8, 0 10px 0 #1e3a8a"],
  ["none", "none"],
];
export default function Case() {
  return (
    <div data-testid="root" className="relative h-full w-full" style={{ background: "#f1f5f9" }}>
      {SHADOWS.map(([id, s], i) => (
        <div key={id} data-testid={id} className="absolute bg-white" style={{ left: 40 + (i % 4) * 190, top: 50 + Math.floor(i / 4) * 170, width: 130, height: 100, borderRadius: i % 3 === 0 ? 50 : i % 3 === 1 ? 18 : 0, boxShadow: s }} />
      ))}
      <div data-testid="texts" className="absolute" style={{ left: 40, top: 420, fontSize: 44, fontWeight: 800, color: "#0f172a" }}>
        <span style={{ textShadow: "2px 2px 0 #f472b6, 4px 4px 0 #60a5fa, 0 0 12px rgba(0,0,0,0.4)" }}>stacked</span>
        <span style={{ marginLeft: 40, textShadow: "none" }}>none</span>
      </div>
    </div>
  );
}
