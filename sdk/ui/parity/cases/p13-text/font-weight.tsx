// Bold: the face has only a regular weight, so Chrome synthesises bold; the same must happen here.
// proves: css.properties.font-weight, tw.utility.font-*
export const steps = [];
export default function Case() {
  return (
    <div data-testid="root" className="h-full w-full bg-white p-4 text-slate-900">
      <p data-testid="n">regular words</p>
      <p data-testid="b" className="font-bold">bold words</p>
    </div>
  );
}
