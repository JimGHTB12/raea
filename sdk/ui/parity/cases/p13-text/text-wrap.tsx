// Wrapping: a long sentence in a narrow box breaks at spaces onto several lines; the box grows with it.
// proves: css.properties.white-space, css.properties.width
// tier: quick
export const steps = [];
export default function Case() {
  return (
    <div data-testid="root" className="h-full w-full bg-white p-4 text-slate-900">
      <p data-testid="p" className="w-48 bg-slate-100 text-sm">The quick brown fox jumps over the lazy dog and keeps on running far away.</p>
    </div>
  );
}
