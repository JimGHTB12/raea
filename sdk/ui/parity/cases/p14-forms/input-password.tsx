// type=password: typed characters shown as bullets, the value kept as typed.
// proves: html.elements.input.type_password
import { useState } from "react";
export const steps = [{ click: "pw" }, { text: "secret" }];
export default function Case() {
  const [v, setV] = useState("");
  return (
    <div data-testid="root" className="h-full w-full bg-white p-4 text-slate-900">
      <input data-testid="pw" type="password" className="h-9 w-56 border border-slate-400 px-2 text-sm" value={v} onChange={(e) => setV(e.target.value)} />
    </div>
  );
}
