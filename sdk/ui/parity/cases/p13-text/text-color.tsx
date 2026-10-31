// Text colour set on a parent and inherited by its text.
// proves: css.properties.color, tw.utility.text-*
export const steps = [];
export default function Case() {
  return (
    <div data-testid="root" className="h-full w-full bg-white p-4 text-slate-900">
      <div data-testid="box" className="w-48 bg-slate-900 p-2 text-amber-300">
        <p data-testid="p">light on dark</p>
      </div>
    </div>
  );
}
