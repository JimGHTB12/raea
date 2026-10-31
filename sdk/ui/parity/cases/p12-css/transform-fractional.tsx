// Bars under a fractional translate and scale (P26 reel daw/song's camera): placed and anti-aliased as Chrome draws them,
// not snapped to whole pixels first.
// proves: css.properties.transform
export const steps = [];
export default function Case() {
  return (
    <div data-testid="root" className="h-full w-full" style={{ position: "relative", background: "#0b0b12" }}>
      <div data-testid="tilt" style={{ position: "absolute", left: 0, top: 0, width: 400, height: 200, transformOrigin: "200px 100px", transform: "rotate(-0.3deg)" }}>
      <div data-testid="cam" style={{ position: "absolute", left: 0, top: 0, width: 400, height: 200, transformOrigin: "0px 0px", transform: "translate(3.37px, 2.61px) scale(1.4)" }}>
        {Array.from({ length: 50 }, (_, i) => (
          <div key={i} style={{ position: "absolute", left: i * 6, top: 40 + (i % 7) * 4, width: 4, height: 100 - (i % 7) * 8, background: "#ff6a3d", borderRadius: 1 }} />
        ))}
      </div>
      </div>
    </div>
  );
}
