// An uncontrolled input: defaultValue, then typing appends at the caret (the end after a click at its right).
// proves: react-dom.controlled.input.defaultValue, html.elements.input
export const steps = [{ click: "field", at: [0.95, 0.5] }, { text: "!" }];
export default function Case() {
  return (
    <div data-testid="root" className="h-full w-full bg-white p-4 text-slate-900">
      <input data-testid="field" className="h-9 w-56 border border-slate-400 px-2 text-sm" defaultValue="start" />
    </div>
  );
}
