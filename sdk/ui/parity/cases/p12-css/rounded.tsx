// Rounded corners: one radius, a full pill, and a radius per corner.
// proves: css.properties.border-radius, tw.utility.rounded-*
export const steps = [];
export default function Case() {
  return (
    <div data-testid="root" className="h-full w-full bg-white p-4">
      <div className="flex gap-4">
        <div data-testid="a" className="h-16 w-24 rounded-lg bg-rose-500" />
        <div data-testid="b" className="h-10 w-32 rounded-full bg-sky-500" />
        <div data-testid="c" className="h-16 w-24 rounded-tl-2xl rounded-br-[20px] bg-emerald-500" />
        <div data-testid="d" className="h-16 w-24 rounded border-2 border-slate-800 bg-white" />
      </div>
    </div>
  );
}
