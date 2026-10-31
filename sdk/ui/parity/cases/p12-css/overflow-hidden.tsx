// overflow-hidden clips a child that's bigger than its parent.
// proves: tw.utility.overflow-hidden
export const steps = [];
export default function Case() {
  return (
    <div data-testid="root" className="h-full w-full bg-white p-4">
      <div data-testid="clip" className="h-20 w-40 overflow-hidden bg-slate-200">
        <div data-testid="big" className="h-40 w-80 bg-rose-400" />
      </div>
    </div>
  );
}
