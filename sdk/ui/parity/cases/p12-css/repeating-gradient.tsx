// repeating-linear-gradient and repeating-radial-gradient: a pattern of stops repeated across the box, hard stops
// written with two positions (`COLOUR 0 38px`), as a game's dashed centre line.
// proves: css.types.gradient.repeating-linear-gradient, css.types.gradient.repeating-radial-gradient, css.types.gradient.linear-gradient.doubleposition
export const steps = [];
export default function Case() {
  return (
    <div data-testid="root" className="flex h-full w-full flex-col gap-4 bg-slate-800 p-4">
      <div data-testid="dashes" className="h-2 w-[600px]" style={{ background: "repeating-linear-gradient(90deg, rgba(244,234,214,0.6) 0 38px, rgba(244,234,214,0) 38px 68px)" }} />
      <div data-testid="stripes" className="h-24 w-72" style={{ background: "repeating-linear-gradient(45deg, rgb(250,204,21) 0 10px, rgb(15,23,42) 10px 20px)" }} />
      <div data-testid="rings" className="h-40 w-40" style={{ background: "repeating-radial-gradient(circle at center, rgb(14,165,233) 0 6px, rgb(255,255,255) 6px 12px)" }} />
    </div>
  );
}
