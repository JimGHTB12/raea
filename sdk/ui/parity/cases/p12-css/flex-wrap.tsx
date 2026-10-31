// flex-wrap: items that don't fit go onto the next line.
// proves: css.properties.flex-wrap, tw.utility.flex-wrap
export const steps = [];
export default function Case() {
  return (
    <div data-testid="root" className="h-full w-full bg-white p-4">
      <div data-testid="box" className="flex w-60 flex-wrap gap-2 bg-slate-100 p-2">
        {[1, 2, 3, 4, 5, 6, 7].map((i) => <div key={i} data-testid={"i" + i} className="h-10 w-16 bg-indigo-400" />)}
      </div>
    </div>
  );
}
