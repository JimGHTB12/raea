// Text changed by state: the box resizes with its new string.
// proves: react.hooks.useState, react-dom.element.span
import { useState } from "react";
export const steps = [{ click: "go" }];
export default function Case() {
  const [t, setT] = useState("short");
  return (
    <div data-testid="root" className="h-full w-full bg-white p-4 text-slate-900">
      <button data-testid="go" className="mb-2 bg-slate-200 px-2" onClick={() => setT("a much longer string now")}>go</button>
      <div><span data-testid="label" className="bg-yellow-200">{t}</span></div>
    </div>
  );
}
