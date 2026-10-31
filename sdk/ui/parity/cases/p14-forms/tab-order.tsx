// Tab moves the focus through the fields in document order; Shift+Tab goes back.
// proves: html.global_attributes.tabindex, css.selectors.focus, tw.variant.focus
// tier: quick
export const steps = [{ key: "Tab" }, { key: "Tab" }, { key: "Tab" }, { key: "Tab", mods: ["shift"] }];
export default function Case() {
  const c = "mb-2 block h-9 w-56 border border-slate-400 px-2 text-sm focus:border-sky-500 focus:bg-sky-50";
  return (
    <div data-testid="root" className="h-full w-full bg-white p-4 text-slate-900">
      <input data-testid="f1" className={c} />
      <input data-testid="f2" className={c} />
      <button data-testid="b" className="bg-slate-200 px-2 focus:bg-sky-300">ok</button>
    </div>
  );
}
