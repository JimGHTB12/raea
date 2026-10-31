// flex-1 and grow: the free space shared by the growing items, a fixed one keeping its width.
// proves: css.properties.flex-grow, tw.utility.flex-*, tw.utility.grow-*
export const steps = [];
export default function Case() {
  return (
    <div data-testid="root" className="h-full w-full bg-white p-4">
      <div data-testid="row" className="flex h-12 w-96 gap-2 bg-slate-100">
        <div data-testid="fixed" className="w-20 bg-rose-400" />
        <div data-testid="one" className="flex-1 bg-sky-400" />
        <div data-testid="two" className="grow bg-emerald-400" />
      </div>
    </div>
  );
}
