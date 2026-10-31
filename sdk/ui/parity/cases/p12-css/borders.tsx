// Borders: widths per side, colour, and how they add to the box (border-box).
// proves: css.properties.border-width, tw.utility.border-*
// tier: quick
export const steps = [];
export default function Case() {
  return (
    <div data-testid="root" className="h-full w-full bg-white p-4">
      <div data-testid="a" className="mb-3 h-16 w-40 border-4 border-slate-900 bg-amber-200" />
      <div data-testid="b" className="mb-3 h-16 w-40 border border-rose-600 bg-white" />
      <div data-testid="c" className="h-16 w-40 border-t-8 border-l-2 border-sky-700 bg-sky-100" />
    </div>
  );
}
