// conic-gradient: stops in degrees and percentages, a start angle (`from`) and a centre (`at`).
// proves: css.types.gradient.conic-gradient
export const steps = [];
export default function Case() {
  return (
    <div data-testid="root" className="flex h-full w-full gap-4 bg-white p-4">
      <div data-testid="wheel" className="h-40 w-40" style={{ background: "conic-gradient(rgb(239,68,68), rgb(234,179,8), rgb(34,197,94), rgb(59,130,246), rgb(239,68,68))" }} />
      <div data-testid="pie" className="h-40 w-40 rounded-full" style={{ background: "conic-gradient(from 90deg, rgb(15,23,42) 0deg 120deg, rgb(148,163,184) 120deg 75%, rgb(226,232,240) 75%)" }} />
      <div data-testid="off" className="h-40 w-56" style={{ background: "conic-gradient(at 25% 75%, rgb(168,85,247), rgb(236,72,153), rgb(168,85,247))" }} />
    </div>
  );
}
