// hidden (display: none) toggled by state: the element takes no space while hidden.
// proves: tw.utility.hidden
import { useState } from "react";
export const steps = [{ click: "toggle" }];
export default function Case() {
  const [show, setShow] = useState(false);
  return (
    <div data-testid="root" className="h-full w-full bg-white p-4">
      <button data-testid="toggle" className="mb-2 bg-slate-200 px-2" onClick={() => setShow(!show)}>toggle</button>
      <div data-testid="maybe" className={(show ? "" : "hidden ") + "h-16 w-32 bg-amber-400"} />
      <div data-testid="after" className="h-8 w-32 bg-slate-500" />
    </div>
  );
}
