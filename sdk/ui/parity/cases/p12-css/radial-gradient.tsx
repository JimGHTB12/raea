// radial-gradient: an ellipse sized in percentages and placed (`118% 60% at 50% 46%`, as a game's table light), a
// circle of a length, a keyword size at a corner, and the default (ellipse, farthest corner, centred). Translucent
// stops blend premultiplied, as CSS's do (a clear stop doesn't grey its neighbour).
// proves: css.types.gradient.radial-gradient, css.types.gradient.radial-gradient.at
export const steps = [];
const box = "h-32 w-48";
export default function Case() {
  return (
    <div data-testid="root" className="grid h-full w-full grid-cols-3 gap-4 bg-white p-4">
      <div data-testid="pool" className={box} style={{ background: "radial-gradient(118% 60% at 50% 46%, rgb(244,234,214), rgb(120,140,110) 58%, rgb(3,20,15) 100%)" }} />
      <div data-testid="circle" className={box} style={{ background: "radial-gradient(circle 40px at 30% 50%, rgb(250,204,21), rgb(37,99,235))" }} />
      <div data-testid="corner" className={box} style={{ background: "radial-gradient(closest-side at left top, rgb(220,38,38), rgb(255,255,255))" }} />
      <div data-testid="plain" className={box} style={{ backgroundImage: "radial-gradient(rgb(16,185,129), rgb(30,41,59))" }} />
      <div data-testid="glow" className={box} style={{ backgroundColor: "rgb(6,78,59)", backgroundImage: "radial-gradient(118% 60% at 50% 46%, rgba(244,234,214,0.5), rgba(244,234,214,0.05) 58%, rgba(3,20,15,0.6) 100%)" }} />
      <div data-testid="fade" className={box} style={{ backgroundColor: "rgb(255,255,255)", backgroundImage: "linear-gradient(90deg, rgba(0,0,0,0), rgb(37,99,235))" }} />
    </div>
  );
}
