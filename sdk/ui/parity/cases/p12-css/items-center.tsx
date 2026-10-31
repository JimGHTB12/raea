// items-center: children of different heights centred on the cross axis.
// proves: css.properties.align-items, tw.utility.items-center
// tier: quick
export const steps = [];
export default function Case() {
  return (
    <div data-testid="root" className="h-full w-full bg-white p-4">
      <div data-testid="row" className="flex h-32 w-80 items-center gap-4 bg-slate-100">
        <div data-testid="a" className="h-8 w-8 bg-rose-400" />
        <div data-testid="b" className="h-20 w-8 bg-sky-400" />
        <div data-testid="c" className="h-12 w-8 bg-emerald-400" />
      </div>
    </div>
  );
}
