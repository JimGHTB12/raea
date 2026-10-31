// Form controls' roles, names, values and states: a text input named by <label for>, a checkbox inside its label, an
// input named only by its placeholder, a checked checkbox, a radio pair, a range (min, max, value), search, number,
// a required email, password, a textarea and a read-only one, a select (combobox: its value is the chosen option),
// a disabled button and input, a submit input named by its value.
// proves: aria.role.textbox, aria.role.checkbox, aria.role.radio, aria.role.slider, aria.role.searchbox, aria.role.spinbutton, aria.role.combobox, aria.html-aam.el-input-text, aria.html-aam.el-input-checkbox, aria.html-aam.el-input-radio, aria.html-aam.el-input-range, aria.html-aam.el-input-search, aria.html-aam.el-input-number, aria.html-aam.el-input-email, aria.html-aam.el-input-password, aria.html-aam.el-input-submit, aria.html-aam.att-for-label, aria.html-aam.att-placeholder, aria.html-aam.att-checked, aria.html-aam.att-disabled, aria.html-aam.att-required, aria.html-aam.att-readonly, aria.html-aam.att-min, aria.html-aam.att-max
export const steps = [];
const f = "block h-6 w-40";
export default function Case() {
  return (
    <div data-testid="root" className="h-full w-full bg-white p-4">
      <div className="grid grid-cols-3 gap-2 opacity-0">
        <label data-testid="name-label" htmlFor="fc-name" className={f}>Name</label>
        <input data-testid="name" id="fc-name" defaultValue="Ada" className={f} />
        <label data-testid="wrap" className={f}><input data-testid="remember" type="checkbox" className="h-4 w-4" />Remember</label>
        <input data-testid="placeholder" placeholder="Search here" className={f} />
        <input data-testid="agree" type="checkbox" aria-label="Agree" defaultChecked className="h-4 w-4" />
        <input data-testid="radio-a" type="radio" name="fc-r" aria-label="Small" defaultChecked className="h-4 w-4" />
        <input data-testid="radio-b" type="radio" name="fc-r" aria-label="Large" className="h-4 w-4" />
        <input data-testid="range" type="range" min={0} max={10} defaultValue={3} aria-label="Volume" className={f} />
        <input data-testid="search" type="search" aria-label="Query" className={f} />
        <input data-testid="number" type="number" aria-label="Count" defaultValue={4} className={f} />
        <input data-testid="email" type="email" aria-label="Email" required className={f} />
        <input data-testid="password" type="password" aria-label="Password" className={f} />
        <textarea data-testid="textarea" aria-label="Message" className={f} />
        <textarea data-testid="readonly" aria-label="Notes" readOnly defaultValue="fixed" className={f} />
        <select data-testid="select" aria-label="Size" defaultValue="m" className={f}>
          <option value="s">Small</option>
          <option value="m">Medium</option>
        </select>
        <button data-testid="disabled" disabled className={f}>Off</button>
        <input data-testid="disabled-input" disabled aria-label="Locked" className={f} />
        <input data-testid="submit" type="submit" value="Send" className={f} />
      </div>
    </div>
  );
}
