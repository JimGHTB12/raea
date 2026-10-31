// A field's placeholder on the row its value takes, whatever the field's line height and box: Chrome's
// ::placeholder is line-height normal (its html.css), centred on the inner editor's line.
// proves: html.elements.input.placeholder, css.properties.line-height
const rows: [string, string][] = [
  ["a", "text-sm"],
  ["b", "h-9 border border-slate-400 px-2 py-1 text-sm"],
  ["c", "h-9 border border-slate-400 px-2 text-sm"],
  ["d", "text-sm leading-6"],
  ["e", "h-10 border border-slate-400 px-2 text-sm leading-7"],
  ["f", "text-[13px] leading-5"],
  ["g", "h-11 border border-slate-400 px-2 text-[13px] leading-5"],
  ["h", "text-base"],
];
export default function Case() {
  return (
    <div data-testid="root" className="flex h-full w-full flex-col gap-2 bg-white p-4 text-slate-900">
      {rows.map(([id, cls]) => (
        <div key={id} className="flex items-center gap-4">
          <input data-testid={`${id}-ph`} className={`w-48 bg-transparent outline-none ${cls}`} placeholder="Hx Hx ph" />
          <input data-testid={`${id}-val`} className={`w-48 bg-transparent outline-none ${cls}`} defaultValue="Hx Hx ph" />
        </div>
      ))}
    </div>
  );
}
