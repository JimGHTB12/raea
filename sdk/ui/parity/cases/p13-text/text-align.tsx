// Alignment: centre and right inside a fixed width.
// proves: css.properties.text-align, tw.utility.text-center, tw.utility.text-right
export const steps = [];
export default function Case() {
  return (
    <div data-testid="root" className="h-full w-full bg-white p-4 text-slate-900">
      <p data-testid="c" className="mb-2 w-64 bg-slate-100 text-center">centred</p>
      <p data-testid="r" className="w-64 bg-slate-100 text-right">right</p>
    </div>
  );
}
