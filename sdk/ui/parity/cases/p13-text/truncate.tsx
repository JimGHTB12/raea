// truncate: one line, cut with an ellipsis.
// proves: css.properties.text-overflow, tw.utility.truncate
export const steps = [];
export default function Case() {
  return (
    <div data-testid="root" className="h-full w-full bg-white p-4 text-slate-900">
      <p data-testid="p" className="w-40 truncate bg-slate-100 text-sm">A line far too long to fit in its box at all</p>
    </div>
  );
}
