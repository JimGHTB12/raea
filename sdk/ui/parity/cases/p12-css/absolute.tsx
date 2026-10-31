// Absolute positioning: children placed by insets inside a relative parent, out of the flow.
// proves: css.properties.position, tw.utility.absolute, tw.utility.relative, tw.utility.inset-*, tw.utility.top-*, tw.utility.left-*
export const steps = [];
export default function Case() {
  return (
    <div data-testid="root" className="h-full w-full bg-white p-4">
      <div data-testid="frame" className="relative h-48 w-80 bg-slate-100">
        <div data-testid="tl" className="absolute top-2 left-3 h-8 w-8 bg-rose-500" />
        <div data-testid="br" className="absolute right-4 bottom-1 h-10 w-16 bg-sky-500" />
        <div data-testid="fill" className="absolute inset-x-10 top-16 h-6 bg-emerald-500" />
        <div data-testid="flow" className="h-4 w-20 bg-amber-400" />
      </div>
    </div>
  );
}
