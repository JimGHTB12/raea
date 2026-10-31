// A grid: three equal columns with gaps, rows as they fill.
// proves: css.properties.grid-template-columns, tw.utility.grid, tw.utility.grid-cols-*
export const steps = [];
export default function Case() {
  return (
    <div data-testid="root" className="h-full w-full bg-white p-4">
      <div data-testid="grid" className="grid w-96 grid-cols-3 gap-3 bg-slate-100 p-2">
        {[1, 2, 3, 4, 5].map((i) => <div key={i} data-testid={"g" + i} className="h-12 bg-violet-400" />)}
      </div>
    </div>
  );
}
