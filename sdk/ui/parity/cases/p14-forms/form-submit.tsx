// Enter in a form's field submits it (onSubmit, prevented), with the field's value.
// proves: html.elements.form, api.HTMLFormElement.submit_event, react-dom.event.onSubmit
import { useState } from "react";
import { trace } from "parity";
export const steps = [{ click: "field" }, { text: "go" }, { key: "Enter" }];
export default function Case() {
  const [v, setV] = useState("");
  const [sent, setSent] = useState("");
  return (
    <div data-testid="root" className="h-full w-full bg-white p-4 text-slate-900">
      <form data-testid="form" onSubmit={(e) => { e.preventDefault(); trace("submit", "form", v); setSent(v); }}>
        <input data-testid="field" className="h-9 w-56 border border-slate-400 px-2 text-sm" value={v} onChange={(e) => setV(e.target.value)} />
      </form>
      <p data-testid="sent" data-value={sent}>{sent}</p>
    </div>
  );
}
