// fetch without a server: data: URLs (text, base64, JSON), a blob: URL from URL.createObjectURL (and revoked), a
// Response and Request built by hand, Headers, a body read twice, an aborted fetch and one to a closed port, with what
// the page shows once each settles.
// proves: api.fetch, api.fetch.blob_data_support, api.fetch.options_parameter.signal, api.Response, api.Response.Response, api.Response.json, api.Response.text, api.Response.arrayBuffer, api.Response.blob, api.Response.ok, api.Response.status, api.Response.statusText, api.Response.headers, api.Response.bodyUsed, api.Response.clone, api.Response.json_static, api.Response.error_static, api.Response.type, api.Request, api.Request.Request, api.Request.method, api.Request.url, api.Request.headers, api.Request.text, api.Request.clone, api.Headers, api.Headers.Headers, api.Headers.append, api.Headers.get, api.Headers.has, api.Headers.set, api.Headers.delete, api.Headers.entries, api.URL.createObjectURL_static, api.URL.revokeObjectURL_static, wintertc.fetch, wintertc.Headers, wintertc.Request, wintertc.Response
import { useState } from "react";
import { trace } from "parity";
export const steps = [{ click: "go" }, { wait: 20 }];
export default function Case() {
  const [done, setDone] = useState(0);
  const go = async () => {
    const log = (on: string, d: unknown) => trace("fetch", on, d);
    const r1 = await fetch("data:text/plain;charset=utf-8,hello%20world");
    log("data-text", [r1.ok, r1.status, r1.statusText, r1.type, r1.headers.get("content-type"), await r1.text(), r1.bodyUsed]);
    const r2 = await fetch("data:application/json;base64," + btoa(JSON.stringify({ a: [1, 2] })));
    log("data-json", await r2.json());
    const blob = new Blob(["blob body"], { type: "text/x-test" });
    const url = URL.createObjectURL(blob);
    const r3 = await fetch(url);
    log("blob", [url.startsWith("blob:"), r3.headers.get("content-type"), r3.headers.get("content-length"), await r3.text()]);
    URL.revokeObjectURL(url);
    try {
      await fetch(url);
      log("revoked", "fetched");
    } catch (e) {
      log("revoked", [(e as Error).name, (e as Error).message]);
    }
    const h = new Headers({ "X-One": "1" });
    h.append("x-one", "2");
    h.set("Content-Type", "text/plain");
    h.append("b-two", " pad ");
    log("headers", [[...h], h.get("X-ONE"), h.has("b-two"), h.get("nope")]);
    h.delete("x-one");
    log("headers-del", [...h.keys()]);
    try {
      new Headers({ "bad name": "x" });
    } catch (e) {
      log("headers-bad", (e as Error).name);
    }
    const res = new Response(JSON.stringify({ k: "v" }), { status: 201, statusText: "Made", headers: { "x-a": "b" } });
    const copy = res.clone();
    log("response", [res.status, res.statusText, res.ok, res.type, await res.json(), await copy.text(), res.bodyUsed]);
    try {
      await res.text();
    } catch (e) {
      log("used", (e as Error).name);
    }
    const j = Response.json({ x: 1 }, { status: 202 });
    log("response-json", [j.status, j.headers.get("content-type"), await j.text()]);
    const err = Response.error();
    log("response-error", [err.type, err.status, err.ok]);
    const ab = await new Response(new Uint8Array([1, 2, 3])).arrayBuffer();
    log("arraybuffer", [ab.byteLength, [...new Uint8Array(ab)]]);
    const bl = await new Response("xyz", { headers: { "content-type": "text/y" } }).blob();
    log("blob-body", [bl.size, bl.type]);
    const req = new Request("data:,x", { method: "post", body: "payload", headers: { "x-r": "1" } });
    log("request", [req.method, req.url, req.headers.get("content-type"), req.headers.get("x-r"), await req.clone().text()]);
    try {
      new Request("data:,x", { method: "GET", body: "no" });
    } catch (e) {
      log("request-get-body", (e as Error).name);
    }
    const ac = new AbortController();
    const p = fetch("data:,later", { signal: ac.signal });
    ac.abort();
    try {
      await p;
      log("abort", "resolved");
    } catch (e) {
      log("abort", [(e as DOMException).name, e instanceof DOMException]);
    }
    try {
      await fetch("http://127.0.0.1:9/");
      log("refused", "resolved");
    } catch (e) {
      log("refused", [(e as Error).name, (e as Error).message]);
    }
    try {
      await fetch("nope://x");
    } catch (e) {
      log("scheme", (e as Error).name);
    }
    setDone(1);
  };
  return (
    <div data-testid="root" className="flex h-full w-full gap-2 bg-white p-4">
      <div data-testid="go" className="h-8 w-16 bg-slate-300" onClick={go} />
      <div data-testid="done" data-value={String(done)} className="h-8 bg-lime-400" style={{ width: 20 + done * 60 }} />
    </div>
  );
}
