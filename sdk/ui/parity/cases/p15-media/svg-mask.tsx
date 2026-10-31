// Masks: a luminance mask of a white circle and a gradient over a striped rect, and a mask with maskUnits in user
// space cutting a hole (a black rect in a white one).
// proves: svg.elements.mask, svg.elements.linearGradient
export const steps = [];
export default function Case() {
  return (
    <div data-testid="root" className="h-full w-full bg-white p-4">
      <svg data-testid="svg" width={400} height={200}>
        <defs>
          <linearGradient id="m-g"><stop offset="0" stopColor="#ffffff" /><stop offset="1" stopColor="#000000" /></linearGradient>
          <mask id="m-a">
            <circle cx={90} cy={90} r={70} fill="#ffffff" />
            <rect x={0} y={150} width={180} height={30} fill="url(#m-g)" />
          </mask>
          <mask id="m-b" maskUnits="userSpaceOnUse" x={200} y={0} width={200} height={200}>
            <rect x={200} y={0} width={180} height={180} fill="#ffffff" />
            <rect x={250} y={50} width={80} height={80} fill="#000000" />
          </mask>
          <pattern id="m-p" width={20} height={20} patternUnits="userSpaceOnUse"><rect width={10} height={20} fill="#e11d48" /><rect x={10} width={10} height={20} fill="#2563eb" /></pattern>
        </defs>
        <rect x={0} y={0} width={180} height={180} fill="url(#m-p)" mask="url(#m-a)" />
        <rect x={200} y={0} width={180} height={180} fill="#16a34a" mask="url(#m-b)" />
      </svg>
    </div>
  );
}
