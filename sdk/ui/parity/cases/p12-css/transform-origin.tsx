// Transforms about an origin, and per-corner radii: a scale from the top-left corner, a rotation and a squash about
// the bottom centre, and corners written as a four-value border-radius.
// proves: css.properties.transform, css.properties.transform-origin, css.properties.border-radius, css.properties.border-top-left-radius
export const steps = [];
export default function Case() {
  return (
    <div data-testid="root" className="relative h-full w-full bg-white">
      <div data-testid="stage" className="absolute bg-slate-200" style={{ left: 40, top: 40, width: 200, height: 100, transform: "scale(1.5)", transformOrigin: "0 0" }}>
        <div data-testid="inner" className="absolute bg-sky-500" style={{ left: 20, top: 20, width: 40, height: 40 }} />
      </div>
      <div data-testid="tipped" className="absolute bg-rose-500" style={{ left: 400, top: 200, width: 80, height: 120, transformOrigin: "50% 100%", transform: "rotate(20deg) scaleY(0.75)" }} />
      <div data-testid="hill" className="absolute bg-emerald-600" style={{ left: 40, top: 360, width: 300, height: 150, borderRadius: "150px 150px 0 0" }} />
      <div data-testid="walker" className="absolute bg-amber-500" style={{ left: 400, top: 400, width: 70, height: 70, borderRadius: "35px 35px 10px 10px" }} />
    </div>
  );
}
