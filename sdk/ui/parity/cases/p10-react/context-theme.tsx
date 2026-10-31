// Context: a provider's value read by a nested consumer, changed by a click.
// proves: react.hooks.useContext, react.api.createContext
// tier: quick
import { createContext, useContext, useState } from "react";
const Theme = createContext("light");
function Panel() {
  const t = useContext(Theme);
  return <div data-testid="panel" data-value={t} className={t === "dark" ? "h-16 w-48 bg-slate-800" : "h-16 w-48 bg-slate-100"} />;
}
export const steps = [{ click: "flip" }];
export default function Case() {
  const [t, setT] = useState("light");
  return (
    <Theme.Provider value={t}>
      <div data-testid="root" className="h-full w-full bg-white p-4 text-slate-900">
        <button data-testid="flip" className="mb-2 bg-slate-200 px-2" onClick={() => setT(t === "light" ? "dark" : "light")}>flip</button>
        <Panel />
      </div>
    </Theme.Provider>
  );
}
