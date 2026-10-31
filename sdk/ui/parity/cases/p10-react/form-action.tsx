// A form's action (React 19): a function as <form action>, called with the form's FormData on submit, in a transition;
// the uncontrolled fields reset to their defaults after it, and a button's formAction overrides the form's.
// proves: react.hooks.useActionState, html.elements.form.action
import { useState } from "react";
import { trace } from "parity";
export const steps = [{ click: "name" }, { key: "End" }, { text: "x" }, { click: "save" }, { click: "other" }];
export default function Case() {
  const [saved, setSaved] = useState("none");
  return (
    <div data-testid="root" className="h-full w-full bg-white p-4 text-slate-900">
      <form
        data-testid="form"
        action={(fd: FormData) => {
          trace("action", "form", { name: fd.get("name"), kind: fd.get("kind") });
          setSaved(String(fd.get("name")));
        }}
      >
        <input data-testid="name" name="name" defaultValue="ada" className="mb-2 block h-8 w-48 border border-slate-400 px-1" />
        <input type="hidden" name="kind" value="person" />
        <div className="flex gap-2">
        <button data-testid="save" type="submit" className="bg-slate-200 px-2">save</button>
        <button data-testid="other" type="submit" className="bg-slate-200 px-2" formAction={(fd: FormData) => { trace("action", "other", { name: fd.get("name") }); setSaved("other"); }}>other</button>
        </div>
      </form>
      <div data-testid="saved" data-value={saved} className={saved === "none" ? "mt-2 h-6 w-24 bg-slate-200" : "mt-2 h-6 w-64 bg-emerald-300"} />
    </div>
  );
}
