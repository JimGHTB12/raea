// Inline svg shapes with fills, strokes, dashes, caps and joins, a transformed group and a half-opaque group, drawn in
// a 200x150 viewBox scaled to 300x225.
// proves: svg.elements.svg, svg.elements.rect, svg.elements.circle, svg.elements.ellipse, svg.elements.line, svg.elements.polyline, svg.elements.polygon, svg.elements.path, svg.elements.g, react-dom.element.svg
export const steps = [];
export default function Case() {
  return (
    <div data-testid="root" className="h-full w-full bg-white p-4">
      <svg data-testid="svg" width={300} height={225} viewBox="0 0 200 150">
        <rect x={10} y={10} width={60} height={40} rx={8} fill="#e11d48" stroke="#111827" strokeWidth={3} />
        <circle cx={110} cy={30} r={20} fill="#16a34a" />
        <ellipse cx={165} cy={30} rx={25} ry={14} fill="none" stroke="#2563eb" strokeWidth={4} strokeDasharray="6 3" />
        <line x1={10} y1={70} x2={190} y2={70} stroke="#f59e0b" strokeWidth={6} strokeLinecap="round" />
        <polyline points="10,120 40,85 70,120 100,85" fill="none" stroke="#7c3aed" strokeWidth={5} strokeLinejoin="round" />
        <polygon points="130,85 160,140 100,140" fill="#0891b2" fillRule="evenodd" />
        <g transform="translate(150 95) rotate(20)" opacity={0.5}>
          <rect width={40} height={30} fill="#111827" />
          <path d="M0 0 L40 30 M40 0 L0 30" stroke="#ffffff" strokeWidth={3} />
        </g>
      </svg>
    </div>
  );
}
