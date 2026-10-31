// A disabled input: a click doesn't focus it and typing doesn't reach it; disabled: styles apply.
// proves: tw.variant.disabled, react-dom.prop.disabled
export const steps = [{ click: "f" }, { text: "no" }];
export default function Case() {
  return (
    <div data-testid="root" className="h-full w-full bg-white p-4 text-slate-900">
      <input data-testid="f" disabled className="h-9 w-56 border border-slate-400 px-2 text-sm disabled:bg-slate-200" defaultValue="locked" />
    </div>
  );
}
