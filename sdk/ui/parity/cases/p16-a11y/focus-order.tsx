// The focus order: Tab through a button, a link, an input, a tabIndex=0 div, a select and a tabIndex=2 div (a positive
// tabIndex comes first), skipping tabIndex=-1, a disabled button and a link without href; Shift+Tab back. After each
// key the focused node, as the accessibility tree has it (focused, focusable), and document focus agree.
// proves: aria.role.button, aria.role.link
export const steps = [
  { key: "Tab" }, { key: "Tab" }, { key: "Tab" }, { key: "Tab" }, { key: "Tab" }, { key: "Tab" }, { key: "Tab" },
  { key: "Tab", mods: ["shift"] }, { key: "Tab", mods: ["shift"] },
];
const f = "block h-6 w-32";
export default function Case() {
  return (
    <div data-testid="root" className="h-full w-full bg-white p-4">
      <div className="grid grid-cols-3 gap-2 opacity-0">
        <button data-testid="button" className={f}>One</button>
        <a data-testid="link" href="#x" className={f}>Two</a>
        <div data-testid="skip" tabIndex={-1} className={f}>skip</div>
        <input data-testid="input" aria-label="Three" className={f} />
        <button data-testid="disabled" disabled className={f}>Off</button>
        <a data-testid="no-href" className={f}>plain</a>
        <div data-testid="div" tabIndex={0} role="button" className={f}>Four</div>
        <select data-testid="select" aria-label="Five" className={f}><option>a</option></select>
        <div data-testid="first" tabIndex={2} role="button" className={f}>Zero</div>
      </div>
    </div>
  );
}
