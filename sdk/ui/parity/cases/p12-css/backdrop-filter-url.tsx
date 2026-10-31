// backdrop-filter: url(#id) naming SVG <filter>s in the page, over a busy ground: a blur, a red-blue swap
// (feColorMatrix, in linearRGB), a url() followed by a function (grayscale), a function before a url() (sepia), and a
// url() to nothing, which leaves the backdrop unfiltered. Each panel's own fill is translucent, so what's under it
// shows through filtered.
// proves: css.properties.backdrop-filter
export const steps = [];
const GROUND = "conic-gradient(from 30deg at 30% 40%, #f43f5e, #f59e0b, #84cc16, #06b6d4, #8b5cf6, #f43f5e)";
const PANELS = ["url(#b-blur)", "url(#b-swap)", "url(#b-swap) grayscale(1)", "sepia(1) url(#b-swap)", "url(#b-none)"];
export default function Case() {
  return (
    <div data-testid="root" className="relative h-full w-full" style={{ background: GROUND }}>
      <svg width={0} height={0} className="absolute">
        <defs>
          <filter id="b-blur">
            <feGaussianBlur stdDeviation="6" />
          </filter>
          <filter id="b-swap">
            <feColorMatrix type="matrix" values="0 0 1 0 0  0 1 0 0 0  1 0 0 0 0  0 0 0 1 0" />
          </filter>
        </defs>
      </svg>
      <div className="absolute" style={{ left: 0, top: 0, right: 0, bottom: 0, backgroundImage: "repeating-linear-gradient(45deg, rgba(255,255,255,0.7) 0 6px, transparent 6px 24px)" }} />
      <div className="absolute" style={{ left: 60, top: 200, fontSize: 64, fontWeight: 800, color: "#0f172a" }}>raea glass</div>
      {PANELS.map((b, i) => (
        <div key={b} data-testid={`p-${i}`} className="absolute" style={{ left: 24 + i * 152, top: 24, width: 136, height: 136, borderRadius: i % 2 ? 24 : 0, background: "rgba(255, 255, 255, 0.12)", backdropFilter: b, border: "1px solid rgba(255,255,255,0.5)" }} />
      ))}
    </div>
  );
}
