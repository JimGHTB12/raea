// A key resets a component (its state starts over), and a component whose type changes at the same place remounts;
// the same type at the same place keeps its state.
// proves: react.hooks.useState, react.components.Fragment
import { useState } from "react";
function Field({ who }: { who: string }) {
  const [n, setN] = useState(0);
  return <button data-testid={"field-" + who} data-value={String(n)} className="mb-1 block h-8 bg-yellow-300 px-2" style={{ width: 60 + n * 20 }} onClick={() => setN(n + 1)}>{who}</button>;
}
function Other({ who }: { who: string }) {
  const [n, setN] = useState(0);
  return <button data-testid={"field-" + who} data-value={"other " + n} className="mb-1 block h-8 bg-cyan-300 px-2" onClick={() => setN(n + 1)}>{who}</button>;
}
export const steps = [{ click: "field-keyed" }, { click: "field-kept" }, { click: "field-swapped" }, { click: "switch" }];
export default function Case() {
  const [user, setUser] = useState("ann");
  return (
    <div data-testid="root" className="h-full w-full bg-white p-4 text-slate-900">
      <button data-testid="switch" className="mb-2 bg-slate-200 px-2" onClick={() => setUser("bob")}>switch</button>
      <Field key={user} who="keyed" />
      <Field who="kept" />
      {user === "ann" ? <Field who="swapped" /> : <Other who="swapped" />}
    </div>
  );
}
