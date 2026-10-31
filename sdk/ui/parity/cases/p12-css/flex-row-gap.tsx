// proves: css.properties.display.flex, css.properties.gap, css.properties.padding, tw.utility.flex, tw.utility.gap-*, tw.utility.p-*, tw.utility.bg-*, api.Element.getBoundingClientRect
// tier: quick
export const steps = [];
export default function Case() {
  return (
    <div data-testid="root" className="h-full w-full bg-white p-4">
      <div data-testid="row" className="flex gap-3 bg-slate-100 p-2">
        <div data-testid="a" className="h-10 w-16 bg-rose-500" />
        <div data-testid="b" className="h-12 w-20 bg-sky-500" />
        <div data-testid="c" className="h-8 w-10 bg-emerald-500" />
      </div>
    </div>
  );
}
