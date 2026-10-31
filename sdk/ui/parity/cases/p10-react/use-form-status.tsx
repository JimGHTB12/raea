// useFormStatus: a submit button inside a form sees it pending while the form's async action runs, with its data.
// proves: react-dom.useFormStatus
import { useFormStatus } from "react-dom";
import { trace } from "parity";
let release: (() => void) | null = null;
function Submit() {
  const s = useFormStatus();
  return <button data-testid="submit" type="submit" data-value={s.pending ? "pending " + s.data?.get("q") : "idle"} className={s.pending ? "bg-amber-300 px-2" : "bg-slate-200 px-2"}>send</button>;
}
export const steps = [{ click: "submit" }, { click: "finish" }];
export default function Case() {
  return (
    <div data-testid="root" className="h-full w-full bg-white p-4 text-slate-900">
      <form
        action={async (fd: FormData) => {
          trace("action", "start", { q: fd.get("q") });
          await new Promise<void>((r) => (release = r));
          trace("action", "end");
        }}
      >
        <input name="q" defaultValue="hello" className="mb-2 block h-8 w-48 border border-slate-400 px-1" />
        <Submit />
      </form>
      <button data-testid="finish" className="mt-2 bg-slate-200 px-2" onClick={() => release?.()}>finish</button>
    </div>
  );
}
