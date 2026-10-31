// filter: blur() on glows (a coloured, rounded box with nothing in it, as the brand films draw them), from a style and
// from Tailwind's blur-* classes, over a dark page and over a light one.
// proves: css.properties.filter.blur
export const steps = [];
export default function Case() {
  return (
    <div data-testid="root" className="relative h-full w-full" style={{ background: "#0b0d14" }}>
      <div data-testid="glow" className="absolute" style={{ left: 40, top: 40, width: 300, height: 180, borderRadius: 180, background: "rgba(90, 160, 255, 0.5)", filter: "blur(30px)" }} />
      <div data-testid="soft" className="absolute rounded-3xl bg-pink-500 blur-md" style={{ left: 420, top: 60, width: 200, height: 140 }} />
      <div data-testid="light" className="absolute bg-white" style={{ left: 0, top: 320, width: 800, height: 280 }}>
        <div data-testid="spot" className="absolute rounded-full bg-amber-400 blur-2xl" style={{ left: 80, top: 60, width: 220, height: 160 }} />
        <div data-testid="sharp" className="absolute rounded-lg bg-emerald-500" style={{ left: 420, top: 80, width: 160, height: 100, filter: "blur(0px)" }} />
      </div>
    </div>
  );
}
