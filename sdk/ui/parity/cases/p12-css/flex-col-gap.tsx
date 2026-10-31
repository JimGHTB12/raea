// A column: flex-col with a gap, children stretched across.
// proves: css.properties.flex-direction, css.properties.gap, tw.utility.flex-col, tw.utility.gap-*
export const steps = [];
export default function Case() {
  return (
    <div data-testid="root" className="h-full w-full bg-white p-4">
      <div data-testid="col" className="flex w-64 flex-col gap-2 bg-slate-100 p-2">
        <div data-testid="a" className="h-8 bg-rose-400" />
        <div data-testid="b" className="h-12 bg-sky-400" />
        <div data-testid="c" className="h-6 bg-emerald-400" />
      </div>
    </div>
  );
}
