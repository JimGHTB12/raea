// Gradients sized and placed: a 2-layer checker from 4px tiles offset 2px (the dither fill raea's samples draw), a dot
// grid of radial tiles, stripes repeated on x only, a centred unrepeated tile, and the shorthand's `0 0 / 24px 24px`.
// proves: css.properties.background-size, css.properties.background-position, css.properties.background-repeat
export const steps = [];
const c = "rgb(15,23,42)";
const checker = `linear-gradient(45deg, ${c} 25%, transparent 25%, transparent 75%, ${c} 75%), linear-gradient(45deg, ${c} 25%, transparent 25%, transparent 75%, ${c} 75%)`;
export default function Case() {
  return (
    <div data-testid="root" className="grid h-full w-full grid-cols-3 gap-6 bg-white p-6">
      <div data-testid="checker" className="h-32" style={{ backgroundImage: checker, backgroundSize: "4px 4px", backgroundPosition: "0 0, 2px 2px" }} />
      <div data-testid="dots" className="h-32 bg-slate-100" style={{ backgroundImage: "radial-gradient(circle, rgb(99,102,241) 2px, transparent 2.5px)", backgroundSize: "16px 16px", backgroundPosition: "8px 8px" }} />
      <div data-testid="stripes" className="h-32 bg-amber-50" style={{ backgroundImage: "linear-gradient(90deg, rgb(245,158,11) 50%, transparent 50%)", backgroundSize: "20px 40%", backgroundRepeat: "repeat-x", backgroundPosition: "0 50%" }} />
      <div data-testid="once" className="h-32 border-4 border-slate-300 bg-slate-50" style={{ backgroundImage: "linear-gradient(135deg, rgb(244,63,94), rgb(14,165,233))", backgroundSize: "60% 50%", backgroundRepeat: "no-repeat", backgroundPosition: "center" }} />
      <div data-testid="shorthand" className="h-32" style={{ background: "linear-gradient(rgb(16,185,129) 2px, transparent 2px) 0 0 / 24px 24px, rgb(236,253,245)" }} />
      <div data-testid="conic" className="h-32" style={{ backgroundImage: "conic-gradient(rgb(30,41,59) 25%, rgb(226,232,240) 0 50%, rgb(30,41,59) 0 75%, rgb(226,232,240) 0)", backgroundSize: "32px 32px" }} />
    </div>
  );
}
