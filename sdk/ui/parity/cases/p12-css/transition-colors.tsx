// A colour transition: hovered, then given time to finish (its end state compared).
// proves: css.properties.transition, tw.utility.transition-*, tw.utility.duration-*
export const steps = [{ move: "card", wait: 30 }];
export default function Case() {
  return (
    <div data-testid="root" className="h-full w-full bg-white p-4">
      <div data-testid="card" className="h-24 w-48 bg-slate-200 transition-colors duration-150 hover:bg-emerald-500" />
    </div>
  );
}
