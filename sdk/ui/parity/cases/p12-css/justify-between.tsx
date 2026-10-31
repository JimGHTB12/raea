// justify-between: the free space between the items, none at the ends.
// proves: css.properties.justify-content, tw.utility.justify-between
export const steps = [];
export default function Case() {
  return (
    <div data-testid="root" className="h-full w-full bg-white p-4">
      <div data-testid="row" className="flex w-96 justify-between bg-slate-100">
        <div data-testid="a" className="h-10 w-10 bg-rose-400" />
        <div data-testid="b" className="h-10 w-16 bg-sky-400" />
        <div data-testid="c" className="h-10 w-12 bg-emerald-400" />
      </div>
    </div>
  );
}
