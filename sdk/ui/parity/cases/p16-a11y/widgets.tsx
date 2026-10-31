// Composite widgets' roles and names: a menu of menuitems (one a menuitemcheckbox), a tablist of tabs and its tabpanel,
// a listbox of options, a radiogroup, a toolbar, a modal dialog named by its heading (aria-labelledby), an alert, a
// status, a tooltip, a separator and a group.
// proves: aria.role.menu, aria.role.menuitem, aria.role.menuitemcheckbox, aria.role.tablist, aria.role.tab, aria.role.tabpanel, aria.role.listbox, aria.role.option, aria.role.radiogroup, aria.role.toolbar, aria.role.dialog, aria.role.alert, aria.role.status, aria.role.tooltip, aria.role.separator, aria.role.group, aria.property.aria-modal
export const steps = [];
const f = "block h-6 w-32";
export default function Case() {
  return (
    <div data-testid="root" className="h-full w-full bg-white p-4">
      <div className="grid grid-cols-3 gap-2 opacity-0">
        <div data-testid="menu" role="menu" aria-label="File" className="block w-32">
          <div data-testid="menuitem" role="menuitem" className={f}>Open</div>
          <div data-testid="menucheck" role="menuitemcheckbox" aria-checked="true" className={f}>Wrap</div>
        </div>
        <div data-testid="tablist" role="tablist" aria-label="Views" className="block w-32">
          <button data-testid="tab-a" role="tab" aria-selected="true" className={f}>One</button>
          <button data-testid="tab-b" role="tab" aria-selected="false" className={f}>Two</button>
        </div>
        <div data-testid="tabpanel" role="tabpanel" aria-label="One" className={f}>panel</div>
        <div data-testid="listbox" role="listbox" aria-label="Fruit" className="block w-32">
          <div data-testid="opt-a" role="option" aria-selected="true" className={f}>Apple</div>
          <div data-testid="opt-b" role="option" aria-selected="false" className={f}>Pear</div>
        </div>
        <div data-testid="radiogroup" role="radiogroup" aria-label="Speed" className="block w-32">
          <div data-testid="radio" role="radio" aria-checked="false" tabIndex={0} className={f}>Fast</div>
        </div>
        <div data-testid="toolbar" role="toolbar" aria-label="Format" className={f} />
        <div data-testid="dialog" role="dialog" aria-modal="true" aria-labelledby="wd-title" className="block w-32">
          <h2 id="wd-title" data-testid="dialog-title" className={f}>Confirm</h2>
        </div>
        <div data-testid="alert" role="alert" className={f}>Oops</div>
        <div data-testid="status" role="status" className={f}>Saved</div>
        <div data-testid="tooltip" role="tooltip" className={f}>Hint</div>
        <div data-testid="separator" role="separator" className={f} />
        <div data-testid="group" role="group" aria-label="Pair" className={f} />
      </div>
    </div>
  );
}
