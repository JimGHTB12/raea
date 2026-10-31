// P18, a real library: Basecoat (basecoat-css 1.1.0), shadcn/ui's look as plain HTML and classes, its compiled
// Tailwind v4 sheet in a <style>: @layer, CSS nesting, :has(), :is(), oklch and color-mix, container queries and
// @property. Buttons in each variant and size, badges, a card with a header, content and footer, fields (input,
// textarea, checkbox, radio, switch), an alert, tabs, a table and a progress bar. React holds the state Basecoat's
// own JS would (the selected tab, the switch); typing, a click on the switch and on a tab restyle through :checked,
// aria-selected and :focus-visible. Text is drawn in the corpus's face, so what's compared is the sheet, not fonts.
// proves: css.at-rules.layer, css.selectors.has, css.selectors.is, css.selectors.nesting, css.types.color.oklch, css.types.color.color-mix
import { useState } from "react";
// the compiled sheet (the package's exports name its Tailwind sources, not the CDN build)
import basecoat from "../../node_modules/basecoat-css/dist/basecoat.cdn.css" with { type: "text" };

export const steps = [{ click: "name" }, { text: "Ada" }, { click: "notify" }, { click: "tab-2" }, { wait: 4 }];

const face = ':root { --font-sans: "Parity Mono"; --font-mono: "Parity Mono"; }';

export default function Case() {
  const [tab, setTab] = useState(1);
  const [notify, setNotify] = useState(false);
  return (
    <div data-testid="root" className="h-full w-full bg-white p-4" style={{ color: "rgb(10, 10, 10)", display: "flex", gap: 16, alignItems: "flex-start" }}>
      <style>{basecoat}</style>
      <style>{face}</style>
      <div style={{ display: "flex", flexDirection: "column", gap: 12, width: 380 }}>
        <div data-testid="buttons" style={{ display: "flex", flexWrap: "wrap", gap: 8 }}>
          <button data-testid="b-primary" className="btn">Primary</button>
          <button data-testid="b-secondary" className="btn" data-variant="secondary">Secondary</button>
          <button data-testid="b-outline" className="btn" data-variant="outline">Outline</button>
          <button data-testid="b-ghost" className="btn" data-variant="ghost">Ghost</button>
          <button data-testid="b-destructive" className="btn" data-variant="destructive">Delete</button>
          <button data-testid="b-link" className="btn" data-variant="link">Link</button>
          <button data-testid="b-sm" className="btn" data-size="sm">Small</button>
          <button data-testid="b-lg" className="btn" data-size="lg" data-variant="outline">Large</button>
          <button data-testid="b-disabled" className="btn" disabled>Off</button>
        </div>
        <div data-testid="badges" style={{ display: "flex", gap: 8 }}>
          <span data-testid="g-primary" className="badge">New</span>
          <span data-testid="g-secondary" className="badge" data-variant="secondary">Beta</span>
          <span data-testid="g-outline" className="badge" data-variant="outline">Draft</span>
          <span data-testid="g-destructive" className="badge" data-variant="destructive">Error</span>
        </div>
        <div data-testid="alert" className="alert" role="alert">
          <h2>Heads up</h2>
          <section>Basecoat draws shadcn's look from classes.</section>
        </div>
        <div data-testid="tabs" className="tabs">
          <nav role="tablist" aria-orientation="horizontal">
            {[1, 2, 3].map((i) => (
              <button key={i} data-testid={`tab-${i}`} type="button" role="tab" aria-selected={tab === i} onClick={() => setTab(i)}>
                {["Account", "Password", "Team"][i - 1]}
              </button>
            ))}
          </nav>
          <div data-testid="panel" role="tabpanel">Panel {tab}</div>
        </div>
        <div data-testid="progress" className="progress" role="progressbar" aria-valuenow={60}>
          <div style={{ width: "60%" }} />
        </div>
      </div>
      <div style={{ display: "flex", flexDirection: "column", gap: 12, width: 340 }}>
        <div data-testid="card" className="card">
          <header>
            <h2>Create project</h2>
            <p>Deploy in one click.</p>
          </header>
          <section>
            <form className="form" style={{ display: "grid", gap: 12 }} onSubmit={(e) => e.preventDefault()}>
              <div className="field">
                <label htmlFor="name">Name</label>
                <input data-testid="name" id="name" type="text" placeholder="Project name" />
              </div>
              <div className="field">
                <label htmlFor="about">About</label>
                <textarea data-testid="about" id="about" rows={2} placeholder="What it does" />
              </div>
              <div className="field" style={{ display: "flex", gap: 16 }}>
                <label><input data-testid="agree" type="checkbox" defaultChecked /> Agree</label>
                <label><input data-testid="plan" type="radio" name="plan" defaultChecked /> Pro</label>
                <label><input data-testid="notify" type="checkbox" role="switch" checked={notify} onChange={(e) => setNotify(e.target.checked)} /> Notify</label>
              </div>
            </form>
          </section>
          <footer style={{ display: "flex", justifyContent: "space-between" }}>
            <button data-testid="cancel" className="btn" data-variant="outline">Cancel</button>
            <button data-testid="deploy" className="btn">Deploy</button>
          </footer>
        </div>
        <div className="table-container">
          <table data-testid="table" className="table">
            <thead>
              <tr><th>Invoice</th><th>Status</th><th>Amount</th></tr>
            </thead>
            <tbody>
              <tr><td>INV001</td><td>Paid</td><td>$250</td></tr>
              <tr><td>INV002</td><td>Pending</td><td>$150</td></tr>
            </tbody>
          </table>
        </div>
      </div>
    </div>
  );
}
