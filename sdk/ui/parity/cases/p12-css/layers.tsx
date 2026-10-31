// CSS's offscreen groups: opacity on an element with children fades it as one picture (the pill's label doesn't show
// its own background through it), filter: blur() blurs a whole subtree (text and all), and backdrop-filter: blur()
// blurs what's behind a glass panel inside its rounded box.
// proves: css.properties.opacity, css.properties.filter.blur, css.properties.backdrop-filter
export const steps = [];
export default function Case() {
  return (
    <div data-testid="root" className="relative h-full w-full" style={{ background: "#0b0d14", fontFamily: "Inter", color: "#fff" }}>
      <div data-testid="pill" className="absolute rounded-full" style={{ left: 40, top: 40, padding: "14px 28px", background: "#2563eb", opacity: 0.5, fontSize: 28, fontWeight: 700 }}>
        <span style={{ background: "#f59e0b", padding: "2px 8px", borderRadius: 6 }}>Half faded</span>
      </div>
      <div data-testid="stack" className="absolute" style={{ left: 420, top: 30, width: 320, height: 120, opacity: 0.6 }}>
        <div className="absolute bg-rose-500" style={{ left: 0, top: 0, width: 200, height: 100 }} />
        <div className="absolute bg-emerald-400" style={{ left: 100, top: 20, width: 200, height: 100 }} />
      </div>
      <div data-testid="blurred" className="absolute" style={{ left: 40, top: 190, width: 340, height: 160, filter: "blur(6px)" }}>
        <div className="absolute bg-amber-400" style={{ left: 20, top: 20, width: 120, height: 120, borderRadius: 24 }} />
        <div className="absolute" style={{ left: 160, top: 50, fontSize: 40, fontWeight: 800 }}>Blur</div>
      </div>
      <div data-testid="behind" className="absolute" style={{ left: 420, top: 190, width: 340, height: 200 }}>
        <div className="absolute bg-fuchsia-500" style={{ left: 0, top: 0, width: 120, height: 200 }} />
        <div className="absolute bg-cyan-400" style={{ left: 120, top: 0, width: 110, height: 200 }} />
        <div className="absolute bg-lime-400" style={{ left: 230, top: 0, width: 110, height: 200 }} />
        <div data-testid="glass" className="absolute rounded-2xl backdrop-blur-md" style={{ left: 40, top: 40, width: 260, height: 120, background: "rgba(255, 255, 255, 0.15)" }} />
      </div>
      <div data-testid="light" className="absolute bg-white" style={{ left: 0, top: 420, width: 800, height: 180 }}>
        <div data-testid="card" className="absolute rounded-xl" style={{ left: 40, top: 30, width: 300, height: 120, background: "#111827", opacity: 0.4, color: "#fff", fontSize: 24, padding: 20 }}>
          Card at 40%
        </div>
        <div data-testid="soft" className="absolute" style={{ left: 420, top: 40, filter: "blur(3px)", fontSize: 44, fontWeight: 700, color: "#111827" }}>Soft heading</div>
      </div>
    </div>
  );
}
