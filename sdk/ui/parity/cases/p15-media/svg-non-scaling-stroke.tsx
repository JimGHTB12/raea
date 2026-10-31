// P15: vector-effect="non-scaling-stroke", as watch's histogram draws its lines: polylines in a 48 x 24 viewBox
// stretched to a wide box (preserveAspectRatio="none") keep their 0.5 and 2 px widths, as does a path in a scaled
// group; a plain stroke beside them scales with the viewBox, and one written in the style attribute counts too.
// proves: svg.global_attributes.vector-effect
export const steps = [];

const wave = Array.from({ length: 49 }, (_, i) => `${i},${12 - 9 * Math.sin(i / 3)}`).join(" ");

export default function Case() {
  return (
    <div data-testid="root" className="flex h-full w-full flex-col gap-4 bg-white p-4">
      <svg data-testid="stretched" viewBox="0 0 48 24" preserveAspectRatio="none" className="h-20 w-[480px] bg-slate-100">
        <polyline points={wave} fill="none" stroke="#e11d48" strokeWidth="0.5" vectorEffect="non-scaling-stroke" />
        <polyline points={wave} fill="none" stroke="#2563eb" strokeWidth="2" vectorEffect="non-scaling-stroke" transform="translate(0 4)" />
        <polyline points="0,22 48,2" fill="none" stroke="#16a34a" strokeWidth="0.5" />
      </svg>
      <svg data-testid="scaled" viewBox="0 0 100 50" className="h-[100px] w-[200px] bg-slate-100">
        <g transform="scale(4 1.5)">
          <rect x="2" y="4" width="16" height="20" fill="#fde68a" stroke="#111827" strokeWidth="1" style={{ vectorEffect: "non-scaling-stroke" }} />
        </g>
        <rect x="76" y="6" width="16" height="30" fill="none" stroke="#111827" strokeWidth="1" />
      </svg>
    </div>
  );
}
