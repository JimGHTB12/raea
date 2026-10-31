// Letter spacing: tracking-widest against normal.
// proves: css.properties.letter-spacing, tw.utility.tracking-*
export const steps = [];
export default function Case() {
  return (
    <div data-testid="root" className="h-full w-full bg-white p-4 text-slate-900">
      <p data-testid="n" className="inline-block bg-slate-100">spacing</p>
      <br />
      <p data-testid="w" className="inline-block bg-slate-100 tracking-widest">spacing</p>
    </div>
  );
}
