// The async clipboard from a click: writeText, then readText gives it back; ClipboardItem's shape.
// proves: api.Clipboard, api.Clipboard.writeText, api.Clipboard.readText, api.Navigator.clipboard, api.ClipboardItem, api.ClipboardItem.ClipboardItem, api.ClipboardItem.types, api.ClipboardItem.getType
import { useState } from "react";
import { trace } from "parity";
export const steps = [{ click: "copy" }, { wait: 6 }, { click: "paste" }, { wait: 6 }];
export default function Case() {
  const [got, setGot] = useState("");
  const copy = () =>
    navigator.clipboard.writeText("copied text").then(
      () => trace("clipboard", "write", "ok"),
      (e) => trace("clipboard", "write", (e as DOMException).name),
    );
  const paste = async () => {
    try {
      const t = await navigator.clipboard.readText();
      trace("clipboard", "read", t);
      setGot(t);
    } catch (e) {
      trace("clipboard", "read", (e as DOMException).name);
    }
    const item = new ClipboardItem({ "text/plain": new Blob(["i"], { type: "text/plain" }) });
    trace("clipboard", "item", [item.types, await (await item.getType("text/plain")).text()]);
  };
  return (
    <div data-testid="root" className="flex h-full w-full gap-2 bg-white p-4">
      <div data-testid="copy" className="h-8 w-16 bg-slate-300" onClick={copy} />
      <div data-testid="paste" className="h-8 w-16 bg-slate-400" onClick={paste} />
      <div data-testid="got" data-value={got} className={got ? "h-8 w-40 bg-lime-400" : "h-8 w-10 bg-slate-200"} />
    </div>
  );
}
