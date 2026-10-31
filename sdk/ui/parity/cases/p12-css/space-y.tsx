// space-y: margins between siblings (not before the first).
// proves: tw.utility.space-y-*
export const steps = [];
export default function Case() {
  return (
    <div data-testid="root" className="h-full w-full bg-white p-4">
      <div data-testid="stack" className="w-48 space-y-3 bg-slate-100">
        <div data-testid="a" className="h-8 bg-rose-400" />
        <div data-testid="b" className="h-8 bg-sky-400" />
        <div data-testid="c" className="h-8 bg-emerald-400" />
      </div>
    </div>
  );
}
