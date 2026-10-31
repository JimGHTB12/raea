// CSS masks: a gradient mask-image (alpha and luminance), a radial vignette, an SVG mask image tiled with
// mask-repeat and mask-size, positioned, two masks composited (intersect, exclude, subtract), and mask: none.
// proves: css.properties.mask, css.properties.mask-image, css.properties.mask-image.none, css.properties.mask-image.multiple_mask_images, css.properties.mask-image.svg_masks, css.properties.mask-mode, css.properties.mask-mode.alpha, css.properties.mask-mode.luminance, css.properties.mask-repeat, css.properties.mask-repeat.no-repeat, css.properties.mask-repeat.repeat, css.properties.mask-repeat.repeat-x, css.properties.mask-size, css.properties.mask-size.contain, css.properties.mask-size.cover, css.properties.mask-position, css.properties.mask-position.center, css.properties.mask-position.right, css.properties.mask-composite, css.properties.mask-composite.add, css.properties.mask-composite.intersect, css.properties.mask-composite.exclude, css.properties.mask-composite.subtract, css.properties.mask.none
export const steps = [];
const STAR = "data:image/svg+xml," + encodeURIComponent('<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 40 40"><path d="M20 2l5 13h13l-10.5 8 4 13L20 28 8.5 36l4-13L2 15h13z"/></svg>');
const FILL = "linear-gradient(135deg, #f43f5e, #8b5cf6 50%, #06b6d4)";
const BOXES: [string, React.CSSProperties][] = [
  ["fade", { maskImage: "linear-gradient(to right, black, transparent)" }],
  ["vignette", { maskImage: "radial-gradient(circle, black 40%, transparent 70%)" }],
  ["lum", { maskImage: "linear-gradient(to bottom, white, black)", maskMode: "luminance" }],
  ["alpha", { maskImage: "linear-gradient(to bottom, white, black)", maskMode: "alpha" }],
  ["star", { maskImage: `url("${STAR}")`, maskSize: "contain", maskRepeat: "no-repeat", maskPosition: "center" }],
  ["stars", { maskImage: `url("${STAR}")`, maskSize: "32px 32px", maskRepeat: "repeat" }],
  ["stars-x", { maskImage: `url("${STAR}")`, maskSize: "28px 28px", maskRepeat: "repeat-x", maskPosition: "right" }],
  ["cover", { maskImage: `url("${STAR}")`, maskSize: "cover", maskRepeat: "no-repeat" }],
  ["add", { maskImage: "linear-gradient(black, black), radial-gradient(circle, black 30%, transparent 31%)", maskSize: "50% 100%, 100% 100%", maskRepeat: "no-repeat", maskComposite: "add" }],
  ["intersect", { maskImage: "linear-gradient(to right, black, transparent), radial-gradient(circle, black 45%, transparent 46%)", maskComposite: "intersect" }],
  ["exclude", { maskImage: "linear-gradient(black, black), radial-gradient(circle, black 35%, transparent 36%)", maskComposite: "exclude" }],
  ["subtract", { maskImage: "linear-gradient(black, black), radial-gradient(circle, black 35%, transparent 36%)", maskComposite: "subtract" }],
  ["none", { maskImage: "none" }],
  ["shorthand", { mask: "radial-gradient(circle at 30% 30%, black 30%, transparent 60%) no-repeat" }],
];
export default function Case() {
  return (
    <div data-testid="root" className="relative h-full w-full" style={{ background: "repeating-linear-gradient(90deg, #f8fafc 0 16px, #e2e8f0 16px 32px)" }}>
      {BOXES.map(([id, m], i) => (
        <div key={id} data-testid={id} className="absolute" style={{ left: 16 + (i % 6) * 130, top: 16 + Math.floor(i / 6) * 130, width: 118, height: 118, background: FILL, ...m }} />
      ))}
    </div>
  );
}
