// Drop shadows: feDropShadow on a rect, and the long form (blur of SourceAlpha, offset, flood, composite, merge) on a
// circle.
// proves: svg.elements.feDropShadow, svg.elements.feOffset, svg.elements.feFlood, svg.elements.feComposite, svg.elements.feMerge
export const steps = [];
export default function Case() {
  return (
    <div data-testid="root" className="h-full w-full bg-white p-4">
      <svg data-testid="svg" width={400} height={200}>
        <defs>
          <filter id="s-a" x="-30%" y="-30%" width="160%" height="160%"><feDropShadow dx={6} dy={8} stdDeviation={4} floodColor="#111827" floodOpacity={0.5} /></filter>
          <filter id="s-b" x="-30%" y="-30%" width="160%" height="160%">
            <feGaussianBlur in="SourceAlpha" stdDeviation={3} />
            <feOffset dx={-5} dy={5} result="o" />
            <feFlood floodColor="#7c3aed" />
            <feComposite in2="o" operator="in" result="s" />
            <feMerge><feMergeNode in="s" /><feMergeNode in="SourceGraphic" /></feMerge>
          </filter>
        </defs>
        <rect x={30} y={30} width={120} height={90} rx={10} fill="#facc15" filter="url(#s-a)" />
        <circle cx={280} cy={90} r={55} fill="#0891b2" filter="url(#s-b)" />
      </svg>
    </div>
  );
}
