// viewBox and preserveAspectRatio: the same 100x50 drawing in 150x150 boxes, meet (centred, xMinYMin), slice
// (xMidYMid, xMaxYMax) and none (stretched); each svg has a slate background from its class.
// proves: svg.elements.svg
export const steps = [];
const Art = () => (
  <>
    <rect width={100} height={50} fill="#2563eb" />
    <circle cx={25} cy={25} r={20} fill="#facc15" />
    <rect x={60} y={10} width={30} height={30} fill="#e11d48" />
  </>
);
export default function Case() {
  const box = { width: 150, height: 150, viewBox: "0 0 100 50" };
  return (
    <div data-testid="root" className="flex h-full w-full flex-wrap content-start gap-3 bg-white p-4">
      <svg data-testid="meet" {...box} className="shrink-0 bg-slate-300"><Art /></svg>
      <svg data-testid="meet-min" {...box} preserveAspectRatio="xMinYMin meet" className="shrink-0 bg-slate-300"><Art /></svg>
      <svg data-testid="slice" {...box} preserveAspectRatio="xMidYMid slice" className="shrink-0 bg-slate-300"><Art /></svg>
      <svg data-testid="slice-max" {...box} preserveAspectRatio="xMaxYMax slice" className="shrink-0 bg-slate-300"><Art /></svg>
      <svg data-testid="none" {...box} preserveAspectRatio="none" className="shrink-0 bg-slate-300"><Art /></svg>
      <svg data-testid="css-size" viewBox="0 0 100 50" className="h-[60px] w-[240px] shrink-0 bg-slate-300"><Art /></svg>
    </div>
  );
}
