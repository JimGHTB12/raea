// CSS skews: skewX, skewY and skew() in a transform, with a rotate before them, about a transform-origin, and
// Tailwind's skew-x-* and -skew-y-* classes; a skewed box's text and children lean with it.
// proves: css.types.transform-function.skew, css.types.transform-function.skewX, css.types.transform-function.skewY, css.properties.transform
export const steps = [];
export default function Case() {
  return (
    <div data-testid="root" className="relative h-full w-full" style={{ background: "#0f172a", fontFamily: "Inter", color: "#fff" }}>
      <div data-testid="x" className="absolute bg-rose-500" style={{ left: 60, top: 50, width: 220, height: 100, transform: "skewX(-20deg)", fontSize: 32, fontWeight: 800, padding: 20 }}>
        SALE
      </div>
      <div data-testid="y" className="absolute bg-sky-500" style={{ left: 360, top: 40, width: 160, height: 120, transform: "skewY(12deg)" }}>
        <div className="bg-white" style={{ margin: 20, height: 30, borderRadius: 8 }} />
      </div>
      <div data-testid="both" className="absolute bg-emerald-400" style={{ left: 600, top: 50, width: 140, height: 100, transform: "rotate(10deg) skew(15deg, 5deg)", borderRadius: 16 }} />
      <div data-testid="origin" className="absolute bg-amber-400" style={{ left: 60, top: 240, width: 200, height: 120, transform: "skewX(25deg)", transformOrigin: "left bottom" }} />
      <div data-testid="tw" className="absolute bg-violet-500 skew-x-12 rounded-xl" style={{ left: 340, top: 240, width: 180, height: 110, color: "#fff", fontSize: 24, fontWeight: 700, padding: 16 }}>
        Tailwind
      </div>
      <div data-testid="neg" className="absolute bg-pink-400 -skew-y-6" style={{ left: 590, top: 250, width: 160, height: 100 }} />
    </div>
  );
}
