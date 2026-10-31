// whitespace-pre: spaces and line breaks kept as written.
// proves: tw.utility.whitespace-pre, css.properties.white-space
export const steps = [];
export default function Case() {
  return (
    <div data-testid="root" className="h-full w-full bg-white p-4 text-slate-900">
      <p data-testid="p" className="bg-slate-100 text-sm whitespace-pre">{"a   b\n  c"}</p>
    </div>
  );
}
