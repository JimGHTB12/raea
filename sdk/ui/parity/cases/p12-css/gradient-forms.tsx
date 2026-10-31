// The gradient forms raea's films reach for, past the ones gradient-tiles and conic-gradient cover: interpolation
// hints, double-position stops (hard bands), a single colour stop, `to` corners, a unitless 0 angle, colour spaces
// (oklch, in hsl longer hue), repeating radial `at`, repeating conic, transparent stops (premultiplied: no grey
// fringe), several backgrounds layered, and an SVG data image as a background.
// proves: css.types.gradient, css.types.gradient.linear-gradient, css.types.gradient.linear-gradient.interpolation_hints, css.types.gradient.linear-gradient.to, css.types.gradient.linear-gradient.unitless_0_angle, css.types.gradient.linear-gradient.single_color_stop, css.types.gradient.linear-gradient.premultiplied_gradients, css.types.gradient.linear-gradient.interpolation_color_space, css.types.gradient.linear-gradient.hue_interpolation_method, css.types.gradient.radial-gradient.doubleposition, css.types.gradient.radial-gradient.interpolation_hints, css.types.gradient.radial-gradient.premultiplied_gradients, css.types.gradient.repeating-radial-gradient.at, css.types.gradient.repeating-conic-gradient, css.types.gradient.conic-gradient.doubleposition, css.types.gradient.repeating-linear-gradient.doubleposition, css.properties.background-image.gradients, css.properties.background-image.multiple_backgrounds, css.properties.background-image.svg_images
export const steps = [];
const SVG = "data:image/svg+xml," + encodeURIComponent('<svg xmlns="http://www.w3.org/2000/svg" width="40" height="40"><circle cx="20" cy="20" r="12" fill="#0ea5e9"/></svg>');
const BOXES: [string, string][] = [
  ["hint", "linear-gradient(90deg, #0f172a, 20%, #f8fafc)"],
  ["bands", "linear-gradient(90deg, #ef4444 0 25%, #f59e0b 25% 50%, #22c55e 50% 75%, #3b82f6 75%)"],
  ["single", "linear-gradient(#8b5cf6)"],
  ["to-corner", "linear-gradient(to bottom right, #f43f5e, #0ea5e9)"],
  ["zero", "linear-gradient(0, #facc15, #0f766e)"],
  ["oklch", "linear-gradient(in oklch 90deg, #ef4444, #3b82f6)"],
  ["hsl-longer", "linear-gradient(in hsl longer hue 90deg, #ef4444, #ef4444 99%)"],
  ["fade", "linear-gradient(90deg, rgba(220, 38, 38, 1), transparent)"],
  ["radial-bands", "radial-gradient(circle, #fde047 0 30%, #fb923c 30% 60%, #be123c 60%)"],
  ["radial-hint", "radial-gradient(#ffffff, 15%, #1e293b)"],
  ["radial-fade", "radial-gradient(closest-side, rgba(14, 165, 233, 1), transparent)"],
  ["rep-radial-at", "repeating-radial-gradient(circle at 20% 30%, #0f172a 0 6px, #e2e8f0 6px 12px)"],
  ["rep-conic", "repeating-conic-gradient(#111827 0 15deg, #f9fafb 15deg 30deg)"],
  ["conic-bands", "conic-gradient(#22c55e 0 90deg, #eab308 90deg 180deg, #3b82f6 180deg 270deg, #ec4899 270deg)"],
  ["rep-linear-bands", "repeating-linear-gradient(-45deg, #1d4ed8 0 10px, #93c5fd 10px 20px)"],
  ["layered", `url("${SVG}") center / 40px 40px no-repeat, radial-gradient(circle at 70% 30%, rgba(255,255,255,0.8), transparent 40%), linear-gradient(160deg, #4c1d95, #db2777)`],
  ["svg-tiles", `url("${SVG}") 0 0 / 30px 30px`],
];
export default function Case() {
  return (
    <div data-testid="root" className="relative h-full w-full" style={{ background: "#ffffff" }}>
      {BOXES.map(([id, bg], i) => (
        <div key={id} data-testid={id} className="absolute" style={{ left: 16 + (i % 6) * 130, top: 16 + Math.floor(i / 6) * 130, width: 118, height: 118, background: bg }} />
      ))}
    </div>
  );
}
