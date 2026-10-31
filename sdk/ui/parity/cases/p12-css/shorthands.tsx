// What bundlers and hand-written styles emit: the font shorthand, named colours (a minifier turns rgb(255,0,0)
// into red), hsl() in gradient stops, the border shorthand with an rgba colour, and a blurred box-shadow.
// proves: css.properties.font, css.properties.border, css.properties.box-shadow, css.types.color
export const steps = [];
export default function Case() {
  return (
    <div data-testid="root" className="relative h-full w-full" style={{ background: "white" }}>
      <div data-testid="font" className="absolute" style={{ left: 20, top: 10, font: "800 44px 'Parity Mono', monospace", color: "rebeccapurple" }}>Bold 44</div>
      <div data-testid="named" className="absolute" style={{ left: 20, top: 80, width: 80, height: 40, background: "red" }} />
      <div data-testid="named-hex" className="absolute" style={{ left: 120, top: 80, width: 80, height: 40, background: "#ff0" }} />
      <div data-testid="hsl" className="absolute" style={{ left: 220, top: 80, width: 160, height: 80, background: "linear-gradient(135deg, hsl(200, 70%, 55%), hsl(260, 60%, 35%))" }} />
      <div data-testid="bordered" className="absolute" style={{ left: 20, top: 200, width: 200, height: 100, background: "#1b2440", border: "10px solid rgba(244,234,214,0.6)" }} />
      <div data-testid="shadowed" className="absolute" style={{ left: 300, top: 220, width: 160, height: 100, background: "white", borderRadius: 12, boxShadow: "0 12px 24px rgba(0, 0, 0, 0.35)" }} />
    </div>
  );
}
