// line-clamp-2: at most two lines, the second ending in an ellipsis.
// proves: css.properties.line-clamp, tw.utility.line-clamp-*
export const steps = [];
export default function Case() {
  return (
    <div data-testid="root" className="h-full w-full bg-white p-4 text-slate-900">
      <p data-testid="p" className="line-clamp-2 w-48 bg-slate-100 text-sm">One two three four five six seven eight nine ten eleven twelve thirteen fourteen fifteen sixteen.</p>
    </div>
  );
}
