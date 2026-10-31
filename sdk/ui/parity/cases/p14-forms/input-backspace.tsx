// Editing keys: typed, then Backspace deletes, ArrowLeft moves the caret and typing inserts there.
// proves: html.elements.input.type_text, react-dom.controlled.input.value
import { useState } from "react";
export const steps = [{ click: "field" }, { text: "abcd" }, { key: "Backspace" }, { key: "ArrowLeft" }, { text: "X" }];
export default function Case() {
  const [v, setV] = useState("");
  return (
    <div data-testid="root" className="h-full w-full bg-white p-4 text-slate-900">
      <input data-testid="field" className="h-9 w-56 border border-slate-400 px-2 text-sm" value={v} onChange={(e) => setV(e.target.value)} />
    </div>
  );
}
