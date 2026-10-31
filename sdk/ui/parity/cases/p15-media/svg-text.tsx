// Text in an inline svg: sizes, fills, a stroke, text-anchor start/middle/end, a tspan with its own fill and a dy, and
// letter-spacing; in the page's font (the svg inherits it).
// proves: svg.elements.text, svg.elements.tspan
export const steps = [];
export default function Case() {
  return (
    <div data-testid="root" className="h-full w-full bg-white p-4">
      <svg data-testid="svg" width={400} height={220}>
        <text x={10} y={30} fontSize={24} fill="#111827">Hello svg</text>
        <text x={10} y={70} fontSize={18} fill="#e11d48" stroke="#111827" strokeWidth={0.8}>stroked</text>
        <text x={200} y={110} fontSize={16} textAnchor="start" fill="#2563eb">start</text>
        <text x={200} y={135} fontSize={16} textAnchor="middle" fill="#16a34a">middle</text>
        <text x={200} y={160} fontSize={16} textAnchor="end" fill="#7c3aed">end</text>
        <line x1={200} y1={95} x2={200} y2={165} stroke="#9ca3af" />
        <text x={10} y={200} fontSize={20} fill="#111827">a <tspan fill="#f59e0b" dy={-6}>tspan</tspan> b</text>
        <text x={230} y={30} fontSize={14} letterSpacing={4} fill="#0891b2">spaced</text>
      </svg>
    </div>
  );
}
