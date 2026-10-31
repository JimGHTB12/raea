// Radial gradients: centred, with a moved focal point, in user space, and as a stroke's paint.
// proves: svg.elements.radialGradient, svg.elements.stop
export const steps = [];
export default function Case() {
  return (
    <div data-testid="root" className="h-full w-full bg-white p-4">
      <svg data-testid="svg" width={400} height={200}>
        <defs>
          <radialGradient id="rg-a"><stop offset="0" stopColor="#facc15" /><stop offset="1" stopColor="#e11d48" /></radialGradient>
          <radialGradient id="rg-b" cx="0.5" cy="0.5" r="0.5" fx="0.3" fy="0.3"><stop offset="0" stopColor="#ffffff" /><stop offset="0.6" stopColor="#2563eb" /><stop offset="1" stopColor="#111827" /></radialGradient>
          <radialGradient id="rg-c" gradientUnits="userSpaceOnUse" cx="300" cy="50" r="60"><stop offset="0" stopColor="#16a34a" /><stop offset="1" stopColor="#16a34a" stopOpacity={0} /></radialGradient>
          <radialGradient id="rg-d"><stop offset="0" stopColor="#7c3aed" /><stop offset="1" stopColor="#0891b2" /></radialGradient>
        </defs>
        <rect x={0} y={0} width={180} height={90} fill="url(#rg-a)" />
        <circle cx={90} cy={145} r={45} fill="url(#rg-b)" />
        <rect x={200} y={0} width={180} height={100} fill="url(#rg-c)" />
        <rect x={215} y={120} width={150} height={60} fill="none" stroke="url(#rg-d)" strokeWidth={14} />
      </svg>
    </div>
  );
}
