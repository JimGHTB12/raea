// A textarea: several lines typed, Enter breaking the line.
// proves: html.elements.textarea, react-dom.controlled.textarea.value
import { useState } from "react";
export const steps = [{ click: "area" }, { text: "one" }, { key: "Enter" }, { text: "two" }];
export default function Case() {
  const [v, setV] = useState("");
  return (
    <div data-testid="root" className="h-full w-full bg-white p-4 text-slate-900">
      <textarea data-testid="area" rows={4} className="w-64 border border-slate-400 p-1 text-sm" value={v} onChange={(e) => setV(e.target.value)} />
    </div>
  );
}
