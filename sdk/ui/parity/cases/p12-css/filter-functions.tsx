// Every filter function on its own over the same picture (a gradient with a white and a dark mark), a chain of
// several, drop-shadow() following a rounded shape's alpha, and filter: none.
// proves: css.properties.filter, css.properties.filter.brightness, css.properties.filter.contrast, css.properties.filter.drop-shadow, css.properties.filter.grayscale, css.properties.filter.hue-rotate, css.properties.filter.invert, css.properties.filter.opacity, css.properties.filter.saturate, css.properties.filter.sepia, css.properties.filter.none, css.types.filter-function, css.types.filter-function.brightness, css.types.filter-function.contrast, css.types.filter-function.drop-shadow, css.types.filter-function.grayscale, css.types.filter-function.hue-rotate, css.types.filter-function.invert, css.types.filter-function.opacity, css.types.filter-function.saturate, css.types.filter-function.sepia, css.types.filter-function.blur
export const steps = [];
const FILTERS = ["none", "brightness(1.6)", "brightness(0.4)", "contrast(2.2)", "contrast(0.3)", "grayscale(1)", "grayscale(0.5)", "hue-rotate(120deg)", "hue-rotate(-75deg)", "invert(1)", "invert(0.35)", "opacity(0.4)", "saturate(3)", "saturate(0.2)", "sepia(1)", "sepia(0.4) contrast(1.4) brightness(1.1) blur(1.5px)", "drop-shadow(6px 8px 0 #0f172a)", "drop-shadow(0 0 10px rgba(236, 72, 153, 0.9))"];
function Picture() {
  return (
    <div className="absolute inset-0 rounded-xl" style={{ background: "linear-gradient(135deg, #ef4444, #f59e0b 35%, #22c55e 60%, #3b82f6)" }}>
      <div className="absolute rounded-full bg-white" style={{ left: 18, top: 18, width: 34, height: 34 }} />
      <div className="absolute rounded-md" style={{ right: 16, bottom: 16, width: 44, height: 22, background: "#111827" }} />
    </div>
  );
}
export default function Case() {
  return (
    <div data-testid="root" className="relative h-full w-full" style={{ background: "#e5e7eb" }}>
      {FILTERS.map((f, i) => (
        <div key={f} data-testid={`f-${i}`} className="absolute" style={{ left: 20 + (i % 6) * 128, top: 20 + Math.floor(i / 6) * 128, width: 108, height: 108, filter: f }}>
          <Picture />
        </div>
      ))}
      <div data-testid="shape-shadow" className="absolute" style={{ left: 20, top: 420, width: 300, height: 140, filter: "drop-shadow(10px 12px 6px rgba(0, 0, 0, 0.45))" }}>
        <div className="absolute rounded-full" style={{ left: 0, top: 10, width: 120, height: 120, background: "#14b8a6" }} />
        <div className="absolute" style={{ left: 150, top: 30, width: 120, height: 80, background: "#a855f7", clipPath: "polygon(50% 0, 100% 100%, 0 100%)" }} />
      </div>
    </div>
  );
}
