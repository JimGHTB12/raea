// SVG <text> drawn as Chrome draws text: the page's one-face mono (its only face 500) at 400 to 900, where Chrome
// synthesizes bold from 600 up; Helvetica, whose bold is a face of its own, at 400 and 800; and light text on a dark
// band, which CoreText's smoothing thickens more.
// proves: svg.elements.text, css.properties.font-weight
export const steps = [];
const WEIGHTS = [400, 500, 600, 700, 900];
export default function Case() {
  return (
    <div data-testid="root" className="h-full w-full bg-white">
      <svg width={760} height={520}>
        {WEIGHTS.map((w, i) => (
          <text key={w} x={20} y={56 + i * 64} fontSize={44} fontWeight={w} fill="#0f172a">mono {w}</text>
        ))}
        <text x={380} y={56} fontSize={44} fontFamily="Helvetica" fill="#0f172a">Helvetica 400</text>
        <text x={380} y={120} fontSize={44} fontFamily="Helvetica" fontWeight={800} fill="#0f172a">Helvetica 800</text>
        <text x={380} y={170} fontSize={16} fontWeight={700} fill="#0f172a">small mono 700, 16 px</text>
        <rect x={0} y={360} width={760} height={140} fill="#0f172a" />
        <text x={20} y={420} fontSize={40} fill="#f8fafc">light on dark 400</text>
        <text x={20} y={476} fontSize={40} fontWeight={700} fill="#f8fafc">light on dark 700</text>
      </svg>
    </div>
  );
}
