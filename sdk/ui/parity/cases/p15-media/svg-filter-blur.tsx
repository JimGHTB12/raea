// Filters: a Gaussian blur (stdDeviation 4) on a rect, an anisotropic one (6 0) on a circle, and a blur under the
// unblurred source (feMerge).
// proves: svg.elements.filter, svg.elements.feGaussianBlur, svg.elements.feMerge, svg.elements.feMergeNode
export const steps = [];
export default function Case() {
  return (
    <div data-testid="root" className="h-full w-full bg-white p-4">
      <svg data-testid="svg" width={420} height={200}>
        <defs>
          <filter id="f-a" x="-30%" y="-30%" width="160%" height="160%"><feGaussianBlur stdDeviation={4} /></filter>
          <filter id="f-b" x="-30%" y="-30%" width="160%" height="160%"><feGaussianBlur stdDeviation="6 0" /></filter>
          <filter id="f-c" x="-30%" y="-30%" width="160%" height="160%">
            <feGaussianBlur in="SourceGraphic" stdDeviation={5} result="b" />
            <feMerge><feMergeNode in="b" /><feMergeNode in="SourceGraphic" /></feMerge>
          </filter>
        </defs>
        <rect x={20} y={30} width={100} height={100} fill="#e11d48" filter="url(#f-a)" />
        <circle cx={210} cy={80} r={50} fill="#2563eb" filter="url(#f-b)" />
        <rect x={300} y={30} width={90} height={90} fill="#16a34a" filter="url(#f-c)" />
      </svg>
    </div>
  );
}
