// T2d's recorded calls (io/os recorded.rs), headless: each answer kept in order, then the whole record logged as
// "recorded JSON" and emitted as `recorded`. Plain script, no wire.ts, so both targets run exactly this. The test
// queued a pick (a.txt), a save (b.txt) and a dialog's second button before it ran.
(() => {
  const log = [];
  const waiting = {};
  let next = 1;
  const steps = [
    ["os.open", { url: "https://raea.io/" }],
    ["os.open", { url: "file:///etc/hosts" }],
    ["net.clipboard.write", { text: "copied" }],
    ["net.clipboard.read", {}],
    ["fs.pick", { accept: ["txt"] }],
    ["fs.read", { id: 1 }],
    ["fs.save", { name: "b.txt" }],
    ["fs.write", { id: 2, data: "eHl6", binary: true }],
    ["fs.read", { id: 2 }],
    ["fs.read", { id: 99 }],
    ["dialog.message", { title: "Discard?", text: "The draft goes.", kind: "warning", buttons: ["Keep", "Discard"] }],
    ["dialog.message", { title: "Again?", text: "No step answers this one." }],
  ];
  const go = (i) => {
    if (i === steps.length) {
      const s = JSON.stringify(log);
      console.log("recorded " + s);
      return __ui_emit("recorded", s);
    }
    const [name, args] = steps[i];
    const id = next++;
    waiting[id] = (m) => {
      log.push({ call: name, ok: m.ok, value: m.ok ? m.value : m.error });
      go(i + 1);
    };
    __ui_call(id, name, JSON.stringify(args));
  };
  const before = globalThis.__ui_deliver;
  globalThis.__ui_deliver = (json) => {
    const m = JSON.parse(json);
    if (m.t === "answer" && waiting[m.id]) {
      const f = waiting[m.id];
      delete waiting[m.id];
      return f(m);
    }
    if (before) before(json);
  };
  go(0);
})();
