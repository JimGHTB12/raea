// text-shadow as the films write it: a hard offset copy, a soft drop shadow, a glow (two blurred shadows, the first on
// top), one in the text's own colour, inherited from a parent, and `none` under a parent that sets one.
// proves: css.properties.text-shadow
export const steps = [];
export default function Case() {
  return (
    <div data-testid="root" className="relative h-full w-full" style={{ background: "#14161f", fontFamily: "Inter", color: "#fff" }}>
      <div data-testid="hard" className="absolute" style={{ left: 40, top: 30, fontSize: 40, fontWeight: 700, textShadow: "3px 3px 0 #0f1089" }}>Hard copy</div>
      <div data-testid="drop" className="absolute" style={{ left: 420, top: 30, fontSize: 40, textShadow: "0 4px 12px rgba(0, 0, 0, 0.9)" }}>Soft drop</div>
      <div data-testid="glow" className="absolute" style={{ left: 40, top: 130, fontSize: 64, fontWeight: 800, color: "#ffe066", textShadow: "0 0 22px rgba(255, 140, 0, 0.9), 0 0 6px rgba(255, 60, 0, 0.75)" }}>GLOW</div>
      <div data-testid="own" className="absolute" style={{ left: 420, top: 140, fontSize: 48, color: "#5eead4", textShadow: "0 0 18px" }}>own ink</div>
      <div data-testid="parent" className="absolute" style={{ left: 40, top: 270, width: 700, textShadow: "2px 2px 4px #e11d48" }}>
        <div data-testid="child" style={{ fontSize: 32 }}>Inherited from the parent</div>
        <div data-testid="off" style={{ fontSize: 32, textShadow: "none" }}>Turned off below it</div>
      </div>
      <div data-testid="light" className="absolute bg-white" style={{ left: 0, top: 420, width: 800, height: 180 }}>
        <div data-testid="ink" className="absolute" style={{ left: 40, top: 40, fontSize: 56, fontWeight: 700, color: "#111827", textShadow: "0 2px 0 #cbd5e1, 0 6px 16px rgba(17, 24, 39, 0.35)" }}>On paper</div>
      </div>
    </div>
  );
}
