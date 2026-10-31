// WebSocket without a server: the constructor's checks (scheme, fragment, subprotocols), the initial state, and a
// socket to a closed port failing as Chrome reports it: error, then close 1006, not clean.
// proves: api.WebSocket, api.WebSocket.WebSocket, api.WebSocket.url, api.WebSocket.readyState, api.WebSocket.protocol, api.WebSocket.extensions, api.WebSocket.binaryType, api.WebSocket.bufferedAmount, api.WebSocket.close, api.WebSocket.send, api.WebSocket.WebSocket.url_parameter_http_https_relative
import { useState } from "react";
import { trace } from "parity";
export const steps = [{ click: "go" }, { wait: 30 }];
export default function Case() {
  const [state, setState] = useState("idle");
  const go = () => {
    for (const [u, p] of [["ftp://x/", undefined], ["ws://x/#f", undefined], ["ws://x/", ["a", "a"]], ["ws://x/", "bad proto"]] as [string, any][]) {
      try {
        new WebSocket(u, p);
        trace("ctor", u, "ok");
      } catch (e) {
        trace("ctor", u, [(e as DOMException).name, e instanceof DOMException]);
      }
    }
    const ws = new WebSocket("http://127.0.0.1:9/path?q");
    trace("ws", "initial", [ws.url, ws.readyState, ws.protocol, ws.extensions, ws.binaryType, ws.bufferedAmount, WebSocket.CONNECTING, WebSocket.CLOSED]);
    try {
      ws.send("early");
    } catch (e) {
      trace("ws", "send-connecting", (e as DOMException).name);
    }
    try {
      ws.close(1001);
    } catch (e) {
      trace("ws", "close-code", (e as DOMException).name);
    }
    ws.onopen = () => trace("ws", "open");
    ws.onerror = (e) => trace("ws", "error", [e.type, ws.readyState]);
    ws.onclose = (e) => {
      trace("ws", "close", [e.code, e.wasClean, e.reason, ws.readyState]);
      setState("closed");
    };
    setState("connecting");
  };
  return (
    <div data-testid="root" className="flex h-full w-full gap-2 bg-white p-4">
      <div data-testid="go" className="h-8 w-16 bg-slate-300" onClick={go} />
      <div data-testid="state" data-value={state} className={state === "closed" ? "h-8 w-40 bg-rose-400" : "h-8 w-20 bg-slate-200"} />
    </div>
  );
}
