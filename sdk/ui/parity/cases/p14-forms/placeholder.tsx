// A placeholder shown while the field is empty, gone once something is typed.
// proves: html.elements.input.placeholder, tw.variant.placeholder
export const steps = [{ click: "f" }, { text: "x" }];
export default function Case() {
  return (
    <div data-testid="root" className="h-full w-full bg-white p-4 text-slate-900">
      <input data-testid="f" className="h-9 w-56 border border-slate-400 px-2 text-sm" placeholder="type here" />
    </div>
  );
}
