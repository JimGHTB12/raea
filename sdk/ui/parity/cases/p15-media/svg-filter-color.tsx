// Colour filters: feColorMatrix saturate 0 (grey), hueRotate 90, a matrix that swaps red and blue, and luminanceToAlpha.
// proves: svg.elements.feColorMatrix, svg.elements.filter
export const steps = [];
const Swatch = ({ x, f }: { x: number; f: string }) => (
  <g filter={f}>
    <rect x={x} y={20} width={40} height={60} fill="#e11d48" />
    <rect x={x + 40} y={20} width={40} height={60} fill="#16a34a" />
    <rect x={x} y={80} width={80} height={40} fill="#2563eb" />
  </g>
);
export default function Case() {
  return (
    <div data-testid="root" className="h-full w-full bg-white p-4">
      <svg data-testid="svg" width={420} height={150}>
        <defs>
          <filter id="k-a"><feColorMatrix type="saturate" values="0" /></filter>
          <filter id="k-b"><feColorMatrix type="hueRotate" values="90" /></filter>
          <filter id="k-c"><feColorMatrix type="matrix" values="0 0 1 0 0  0 1 0 0 0  1 0 0 0 0  0 0 0 1 0" /></filter>
          <filter id="k-d"><feColorMatrix type="luminanceToAlpha" /></filter>
        </defs>
        <Swatch x={10} f="url(#k-a)" />
        <Swatch x={110} f="url(#k-b)" />
        <Swatch x={210} f="url(#k-c)" />
        <Swatch x={310} f="url(#k-d)" />
      </svg>
    </div>
  );
}
