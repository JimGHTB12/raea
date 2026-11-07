// What a bundle expects of its platform before it runs in raea-rune-host (R4a): the console, microtasks, timers the
// host runs on Rune's virtual clock through __ui_timers(now), performance.now, process.env and the frame number. The
// host's natives, installed first: __ui_log(level, text), __ui_now() (the virtual clock, ms), __ui_flush(json),
// __ui_emit(name, json), __ui_call(id, name, json), __ui_sync(name, json).
(() => {
  const g = globalThis;
  const str = (a) => a.map((x) => (x && x.stack) || (typeof x === "object" ? safe(x) : String(x))).join(" ");
  const safe = (x) => {
    try {
      return JSON.stringify(x);
    } catch {
      return String(x);
    }
  };
  g.console = {
    log: (...a) => __ui_log(0, str(a)),
    info: (...a) => __ui_log(0, str(a)),
    debug: () => {},
    warn: (...a) => __ui_log(1, str(a)),
    error: (...a) => __ui_log(2, str(a)),
  };
  if (typeof g.queueMicrotask !== "function") g.queueMicrotask = (fn) => void Promise.resolve().then(fn);
  // Timers on the host's clock: each is due at a whole millisecond, those due together run in the order they were set,
  // and while one runs "now" is when it was due, so a timer it sets is due after it by its delay
  let seq = 1;
  let cur = null;
  const timers = new Map();
  const due = (ms) => Math.floor(cur ?? __ui_now()) + Math.max(0, Number(ms) || 0);
  g.setTimeout = (fn, ms = 0, ...args) => {
    const id = seq++;
    timers.set(id, { at: due(ms), fn: () => fn(...args) });
    return id;
  };
  // an interval runs again unless cleared, from its own callback too
  const live = new Set();
  g.setInterval = (fn, ms = 0, ...args) => {
    const id = seq++;
    const run = () => {
      fn(...args);
      if (live.has(id)) timers.set(id, { at: due(ms), fn: run });
    };
    live.add(id);
    timers.set(id, { at: due(ms), fn: run });
    return id;
  };
  g.clearInterval = g.clearTimeout = (id) => {
    timers.delete(id);
    live.delete(id);
  };
  // The earliest timer due by `now`, run alone (its microtasks run before the host's next call). Answers 0 when another
  // is due by `now`, else ms to the next one, or -1 when none is set.
  g.__ui_timers = (now) => {
    let first = null;
    for (const e of timers) if (e[1].at <= now && (first === null || e[1].at < first[1].at)) first = e;
    if (first) {
      timers.delete(first[0]);
      cur = first[1].at;
      try {
        first[1].fn();
      } catch (e) {
        console.error(e);
      }
      cur = null;
    }
    let next = -1;
    for (const t of timers.values()) next = next < 0 ? Math.max(0, t.at - now) : Math.min(next, Math.max(0, t.at - now));
    return next;
  };
  g.performance = { now: () => __ui_now() };
  g.process = { env: { NODE_ENV: "production" } };
  g.__ui_frame = 0;
  g.__ui_setFrame = (n) => void (g.__ui_frame = n);
})();
