// Blob, File, FileReader and FormData: a Blob from mixed parts, slice, text/arrayBuffer, a File's name and type,
// FileReader's events and results (text, data URL, array buffer), and FormData from a form and by hand.
// proves: api.Blob, api.Blob.Blob, api.Blob.size, api.Blob.type, api.Blob.slice, api.Blob.text, api.Blob.arrayBuffer, api.Blob.bytes, api.File, api.File.File, api.File.name, api.File.lastModified, api.FileReader, api.FileReader.FileReader, api.FileReader.readAsText, api.FileReader.readAsDataURL, api.FileReader.readAsArrayBuffer, api.FileReader.result, api.FileReader.readyState, wintertc.Blob, wintertc.File, wintertc.FormData
import { useRef, useState } from "react";
import { trace } from "parity";
export const steps = [{ click: "go" }, { wait: 20 }];
export default function Case() {
  const form = useRef<HTMLFormElement>(null);
  const [done, setDone] = useState(false);
  const go = async () => {
    const b = new Blob(["ab", new Uint8Array([99, 100]), new Blob(["é"])], { type: "Text/Plain" });
    trace("blob", "made", [b.size, b.type, await b.text(), await b.slice(1, 3).text(), await b.slice(-2).text(), b.slice(0, 1, "x/y").type]);
    trace("blob", "bytes", [[...new Uint8Array(await b.arrayBuffer())], [...(await b.bytes())].length]);
    const f = new File(["file body"], "dir/notes.txt", { type: "text/plain", lastModified: 1234 });
    trace("file", "made", [f.name, f.type, f.size, f.lastModified, f instanceof Blob, Object.prototype.toString.call(f)]);
    const read = (how: "readAsText" | "readAsDataURL" | "readAsArrayBuffer") =>
      new Promise<void>((ok) => {
        const r = new FileReader();
        const evs: string[] = [];
        r.onloadstart = () => evs.push("loadstart:" + r.readyState);
        r.onprogress = () => evs.push("progress");
        r.onload = () => evs.push("load:" + r.readyState);
        r.onloadend = () => {
          evs.push("loadend");
          const res = r.result;
          trace("reader", how, [evs, res instanceof ArrayBuffer ? [...new Uint8Array(res)] : res]);
          ok();
        };
        r[how](f.slice(0, 4, "text/plain"));
        evs.push("sync:" + r.readyState);
      });
    await read("readAsText");
    await read("readAsDataURL");
    await read("readAsArrayBuffer");
    const fd = new FormData(form.current!);
    trace("formdata", "form", [...fd.entries()]);
    fd.append("extra", "1");
    fd.set("name", "bo");
    fd.append("upload", new Blob(["x"], { type: "a/b" }), "up.bin");
    const up = fd.get("upload") as File;
    trace("formdata", "edit", [fd.get("name"), fd.getAll("tags"), fd.has("extra"), up.name, up.type, up instanceof File]);
    fd.delete("tags");
    trace("formdata", "keys", [...fd.keys()]);
    setDone(true);
  };
  return (
    <div data-testid="root" className="flex h-full w-full gap-2 bg-white p-4">
      <div data-testid="go" className="h-8 w-16 bg-slate-300" onClick={go} />
      <form ref={form} className="hidden">
        <input name="name" defaultValue="ann" />
        <input name="tags" type="checkbox" value="x" defaultChecked />
        <input name="tags" type="checkbox" value="y" />
        <input name="off" disabled defaultValue="no" />
        <select name="pick" defaultValue="2"><option value="1">1</option><option value="2">2</option></select>
        <textarea name="note" defaultValue={"l1\nl2"} />
        <button name="btn" value="v">b</button>
      </form>
      <div data-testid="done" data-value={String(done)} className={done ? "h-8 w-40 bg-lime-400" : "h-8 w-10 bg-slate-200"} />
    </div>
  );
}
