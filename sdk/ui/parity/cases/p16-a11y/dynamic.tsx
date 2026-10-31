// The tree as React changes it: a disclosure button's aria-expanded and the region it shows, a toggle's aria-pressed, a
// checkbox clicked to checked, a tab clicked to selected, a heading's text renamed.
// proves: aria.state.aria-expanded, aria.state.aria-pressed, aria.state.aria-checked, aria.state.aria-selected, aria.html-aam.att-checked
import { useState } from "react";
export const steps = [{ click: "disclose" }, { click: "toggle" }, { click: "check" }, { click: "tab-b" }, { click: "rename" }];
const f = "block h-6 w-32";
export default function Case() {
  const [open, setOpen] = useState(false);
  const [on, setOn] = useState(false);
  const [tab, setTab] = useState("a");
  const [name, setName] = useState("Draft");
  return (
    <div data-testid="root" className="h-full w-full bg-white p-4">
      <div className="grid grid-cols-3 gap-2 opacity-0">
        <button data-testid="disclose" aria-expanded={open} onClick={() => setOpen(!open)} className={f}>Details</button>
        {open ? <section data-testid="details" aria-label="Details" className={f}>more</section> : <span className={f} />}
        <button data-testid="toggle" aria-pressed={on} onClick={() => setOn(!on)} className={f}>Mute</button>
        <input data-testid="check" type="checkbox" aria-label="Done" className="h-4 w-4" />
        <div role="tablist" aria-label="T" className="block w-32">
          <button data-testid="tab-a" role="tab" aria-selected={tab === "a"} onClick={() => setTab("a")} className={f}>A</button>
          <button data-testid="tab-b" role="tab" aria-selected={tab === "b"} onClick={() => setTab("b")} className={f}>B</button>
        </div>
        <h2 data-testid="heading" className={f}>{name}</h2>
        <button data-testid="rename" onClick={() => setName("Final")} className={f}>Rename</button>
      </div>
    </div>
  );
}
