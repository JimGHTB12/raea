// Stacking: overlapping absolute boxes, the higher z-index drawn on top regardless of order.
// proves: css.properties.z-index, tw.utility.z-*
export const steps = [];
export default function Case() {
  return (
    <div data-testid="root" className="h-full w-full bg-white p-4">
      <div className="relative h-40 w-64">
        <div data-testid="top" className="absolute top-4 left-4 z-20 h-20 w-20 bg-rose-500" />
        <div data-testid="bottom" className="absolute top-10 left-10 z-10 h-20 w-20 bg-sky-500" />
      </div>
    </div>
  );
}
