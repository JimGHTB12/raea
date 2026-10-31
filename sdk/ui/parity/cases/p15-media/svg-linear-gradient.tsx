// Linear gradients in an inline svg: two stops across a rect, three stops with stop-opacity down another, one in user
// space with a gradientTransform, and spreadMethod reflect on a short vector.
// proves: svg.elements.linearGradient, svg.elements.stop, svg.elements.defs
export const steps = [];
export default function Case() {
  return (
    <div data-testid="root" className="h-full w-full bg-white p-4">
      <svg data-testid="svg" width={400} height={200}>
        <defs>
          <linearGradient id="lg-a"><stop offset="0" stopColor="#e11d48" /><stop offset="1" stopColor="#2563eb" /></linearGradient>
          <linearGradient id="lg-b" x1="0" y1="0" x2="0" y2="1">
            <stop offset="0" stopColor="#16a34a" /><stop offset="0.5" stopColor="#f59e0b" stopOpacity={0.5} /><stop offset="1" stopColor="#111827" />
          </linearGradient>
          <linearGradient id="lg-c" gradientUnits="userSpaceOnUse" x1="200" y1="0" x2="300" y2="0" gradientTransform="rotate(30 250 50)">
            <stop offset="0" stopColor="#7c3aed" /><stop offset="1" stopColor="#facc15" />
          </linearGradient>
          <linearGradient id="lg-d" x1="0.4" x2="0.6" spreadMethod="reflect"><stop offset="0" stopColor="#0891b2" /><stop offset="1" stopColor="#ffffff" /></linearGradient>
        </defs>
        <rect x={0} y={0} width={180} height={90} fill="url(#lg-a)" />
        <rect x={0} y={100} width={180} height={90} fill="url(#lg-b)" />
        <rect x={200} y={0} width={180} height={90} fill="url(#lg-c)" />
        <circle cx={290} cy={150} r={45} fill="url(#lg-d)" />
      </svg>
    </div>
  );
}
