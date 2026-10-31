// Opacity: an element and everything in it faded.
// proves: css.properties.opacity, tw.utility.opacity-*
export const steps = [];
export default function Case() {
  return (
    <div data-testid="root" className="h-full w-full bg-white p-4">
      <div data-testid="faded" className="h-24 w-48 bg-rose-600 p-4 opacity-50">
        <div data-testid="kid" className="h-8 w-8 bg-slate-900" />
      </div>
    </div>
  );
}
