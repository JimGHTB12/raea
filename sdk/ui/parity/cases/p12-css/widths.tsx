// Widths: a fraction, a max-width cap, an arbitrary value, min-height.
// proves: css.properties.width, tw.utility.w-*, tw.utility.max-w-*, tw.utility.min-h-*
export const steps = [];
export default function Case() {
  return (
    <div data-testid="root" className="h-full w-full bg-white p-4">
      <div data-testid="half" className="mb-2 h-6 w-1/2 bg-rose-400" />
      <div data-testid="capped" className="mb-2 h-6 w-full max-w-xs bg-sky-400" />
      <div data-testid="arb" className="mb-2 h-6 w-[137px] bg-emerald-400" />
      <div data-testid="minh" className="min-h-16 w-24 bg-amber-400" />
    </div>
  );
}
