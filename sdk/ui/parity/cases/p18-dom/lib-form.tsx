// P18, a real library: react-hook-form (7.90.0) with uncontrolled inputs: register's ref reads the element (instanceof
// HTMLElement through ownerDocument.defaultView, isConnected, name, value), a submit with errors focuses the first
// one, typing changes the values, and the second submit hands over the values.
// proves: api.HTMLInputElement.value, api.HTMLElement.focus, api.Node.isConnected, api.HTMLFormElement.submit_event, api.Event.preventDefault, api.Document.defaultView
import { useForm } from "react-hook-form";
import { trace } from "parity";

export const steps = [{ click: "submit" }, { wait: 3 }, { text: "Ada" }, { click: "email" }, { text: "ada@x.io" }, { click: "submit" }, { wait: 3 }];

type F = { name: string; email: string };
// typed text and the caret in the field's colour, no focus ring: how a field draws is P14's
const FIELD = { width: 200, height: 28, backgroundColor: "rgb(241, 245, 249)", color: "rgb(241, 245, 249)", caretColor: "transparent", outline: "none" };

export default function Case() {
  const { register, handleSubmit } = useForm<F>({ defaultValues: { name: "", email: "" } });
  const ok = (v: F) => trace("submit", "form", v);
  const bad = (e: Record<string, unknown>) => trace("invalid", "form", [Object.keys(e), document.activeElement?.getAttribute("data-testid") ?? "body"]);
  return (
    <div data-testid="root" className="flex h-full w-full flex-col bg-white" style={{ padding: 16, gap: 8 }}>
      <form data-testid="form" className="flex flex-col" style={{ gap: 8, flexShrink: 0 }} onSubmit={handleSubmit(ok, bad)}>
        <input data-testid="name" {...register("name", { required: true })} style={FIELD} />
        <input data-testid="email" {...register("email", { required: true, pattern: /@/ })} style={FIELD} />
        <button data-testid="submit" type="submit" style={{ width: 80, height: 28, backgroundColor: "rgb(30, 41, 59)", outline: "none" }} />
      </form>
    </div>
  );
}
