// clipPath: a picture of stripes clipped to a circle, a group clipped to a star (evenodd clip-rule), and a clip in
// objectBoundingBox units.
// proves: svg.elements.clipPath, svg.elements.pattern
export const steps = [];
export default function Case() {
  return (
    <div data-testid="root" className="h-full w-full bg-white p-4">
      <svg data-testid="svg" width={420} height={200}>
        <defs>
          <clipPath id="c-a"><circle cx={90} cy={90} r={70} /></clipPath>
          <clipPath id="c-b" clipRule="evenodd"><path clipRule="evenodd" d="M290 10 L320 170 L190 70 L390 70 L260 170 Z" /></clipPath>
          <clipPath id="c-c" clipPathUnits="objectBoundingBox"><rect x={0.25} y={0.25} width={0.5} height={0.5} /></clipPath>
          <pattern id="c-p" width={16} height={16} patternUnits="userSpaceOnUse"><rect width={16} height={8} fill="#e11d48" /><rect y={8} width={16} height={8} fill="#facc15" /></pattern>
        </defs>
        <rect x={0} y={0} width={180} height={180} fill="url(#c-p)" clipPath="url(#c-a)" />
        <g clipPath="url(#c-b)"><rect x={180} y={0} width={220} height={180} fill="#2563eb" /><circle cx={290} cy={90} r={40} fill="#16a34a" /></g>
        <rect x={340} y={130} width={60} height={60} fill="#7c3aed" clipPath="url(#c-c)" />
      </svg>
    </div>
  );
}
