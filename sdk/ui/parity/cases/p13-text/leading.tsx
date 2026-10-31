// Line height: leading-loose against leading-none on wrapped text.
// proves: css.properties.line-height, tw.utility.leading-*
export const steps = [];
export default function Case() {
  return (
    <div data-testid="root" className="h-full w-full bg-white p-4 text-slate-900">
      <p data-testid="loose" className="mb-2 w-40 bg-slate-100 text-sm leading-loose">Some words that wrap onto lines</p>
      <p data-testid="none" className="w-40 bg-slate-100 text-sm leading-none">Some words that wrap onto lines</p>
    </div>
  );
}
