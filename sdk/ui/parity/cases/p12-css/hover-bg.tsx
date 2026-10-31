// hover: a background that changes while the pointer is over the element, and back when it leaves.
// proves: css.selectors.hover, tw.variant.hover
// tier: quick
export const steps = [{ move: "card" }, { move: [700, 500] }];
export default function Case() {
  return (
    <div data-testid="root" className="h-full w-full bg-white p-4">
      <div data-testid="card" className="h-24 w-48 bg-slate-200 hover:bg-sky-400" />
    </div>
  );
}
