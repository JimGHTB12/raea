// T2b's recorded calls (io/net/calls serve.rs): made in order, each answer and each socket event kept, then the
// whole record logged as "recorded JSON" and emitted as `recorded`. Plain script, no wire.ts: it speaks the wire's
// __ui_call and __ui_deliver itself, so both targets run exactly this.
(() => {
  const log = [];
  const waiting = {};
  let next = 1;
  const call = (name, args, then) => {
    const id = next++;
    waiting[id] = (m) => {
      log.push({ call: name, ok: m.ok, value: m.ok ? m.value : m.error });
      if (then) then(m);
    };
    __ui_call(id, name, JSON.stringify(args));
  };
  const done = () => {
    const s = JSON.stringify(log);
    console.log("recorded " + s);
    __ui_emit("recorded", s);
  };
  const ws = (e) => {
    log.push({ event: "net.ws", value: e });
    if (e.ev === "open") call("net.ws.send", { id: 1, data: "hi" });
    else if (e.ev === "message") call("net.ws.close", { id: 1, code: 1000, reason: "bye" });
    else if (e.ev === "close") done();
  };
  const before = globalThis.__ui_deliver;
  globalThis.__ui_deliver = (json) => {
    const m = JSON.parse(json);
    if (m.t === "answer" && waiting[m.id]) {
      const f = waiting[m.id];
      delete waiting[m.id];
      return f(m);
    }
    if (m.t === "named" && m.name === "net.ws") return ws(m.value);
    if (before) before(json);
  };
  call("net.fetch", { url: "%HTTP%/hello", method: "GET", headers: [], body: null, redirect: "follow" }, () =>
    call("net.storage.set", { area: "t2b", change: { k: "a", v: "1" } }, () =>
      call("net.storage.get", { area: "t2b" }, () => call("net.ws.open", { id: 1, url: "%WS%", protocols: [] }))));
})();
