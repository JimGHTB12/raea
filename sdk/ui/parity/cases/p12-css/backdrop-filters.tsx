// backdrop-filter over a busy ground: blur, a chain, each colour function, a rounded and a clipped panel, nested
// glass, and backdrop-filter: none. Each panel's own fill is translucent, so what's under it shows through filtered.
// proves: css.properties.backdrop-filter, css.properties.backdrop-filter.none
export const steps = [];
const GROUND = "conic-gradient(from 30deg at 30% 40%, #f43f5e, #f59e0b, #84cc16, #06b6d4, #8b5cf6, #f43f5e)";
const PANELS = ["blur(8px)", "blur(24px)", "grayscale(1)", "invert(1)", "sepia(1) saturate(2)", "brightness(0.5) blur(4px)", "hue-rotate(180deg)", "contrast(0.4)", "none"];
export default function Case() {
  return (
    <div data-testid="root" className="relative h-full w-full" style={{ background: GROUND }}>
      <div className="absolute" style={{ left: 0, top: 0, right: 0, bottom: 0, backgroundImage: "repeating-linear-gradient(45deg, rgba(255,255,255,0.7) 0 6px, transparent 6px 24px)" }} />
      <div className="absolute" style={{ left: 60, top: 260, fontSize: 64, fontWeight: 800, color: "#0f172a" }}>raea glass</div>
      {PANELS.map((b, i) => (
        <div key={b} data-testid={`p-${i}`} className="absolute" style={{ left: 24 + (i % 5) * 152, top: 24 + Math.floor(i / 5) * 152, width: 136, height: 136, borderRadius: i % 2 ? 24 : 0, background: "rgba(255, 255, 255, 0.12)", backdropFilter: b, border: "1px solid rgba(255,255,255,0.5)" }} />
      ))}
      <div data-testid="outer" className="absolute" style={{ left: 420, top: 340, width: 340, height: 220, borderRadius: 28, background: "rgba(15, 23, 42, 0.25)", backdropFilter: "blur(12px)" }}>
        <div data-testid="inner" className="absolute" style={{ left: 40, top: 40, width: 180, height: 120, borderRadius: 16, background: "rgba(255, 255, 255, 0.2)", backdropFilter: "grayscale(1) blur(2px)" }} />
      </div>
      <div data-testid="clipped" className="absolute" style={{ left: 40, top: 360, width: 260, height: 200, background: "rgba(255,255,255,0.1)", backdropFilter: "blur(10px) invert(0.8)", clipPath: "circle(45% at 50% 50%)" }} />
    </div>
  );
}
