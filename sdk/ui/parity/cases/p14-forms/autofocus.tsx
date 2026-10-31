// autoFocus: the field has the focus after the mount, and typing goes straight in.
// proves: html.global_attributes.autofocus
export const steps = [{ text: "hi" }];
export default function Case() {
  return (
    <div data-testid="root" className="h-full w-full bg-white p-4 text-slate-900">
      <input data-testid="other" className="h-9 w-56 border border-slate-400 px-2 text-sm" />
      <input data-testid="f" className="h-9 w-56 border border-slate-400 px-2 text-sm" autoFocus />
    </div>
  );
}
