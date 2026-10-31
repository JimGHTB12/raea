// What a bundle expects of its platform before it runs, on any engine (JavaScriptCore, QuickJS): the console, microtasks,
// timers kept here and run by the host each frame through __ui_timers(now), performance.now, process.env, and the
// frame number the host sets before it delivers anything in a frame. The host's natives (installed before this runs):
// __ui_log(level, text), __ui_now() (ms), __ui_flush(json), __ui_emit(name, json), __ui_call(id, name, json).
(() => {
  const g = globalThis;
  const str = (a) => a.map((x) => (x && x.stack) || String(x)).join(" ");
  g.console = {
    log: (...a) => __ui_log(0, str(a)),
    info: (...a) => __ui_log(0, str(a)),
    debug: () => {},
    warn: (...a) => __ui_log(1, str(a)),
    error: (...a) => __ui_log(2, str(a)),
  };
  if (typeof g.queueMicrotask !== "function") g.queueMicrotask = (fn) => void Promise.resolve().then(fn);
  let seq = 1;
  const timers = new Map();
  // Timers run on a virtual clock as the host steps them: while one runs, "now" is the time it was due, so a timer set
  // from it is due after it by its delay as in a browser, not after the frame the host ran it in (a zero timer set by
  // a timer runs before a 10 ms one set with it). Nested past 5 levels, a delay is at least 4 ms (HTML's clamp)
  let cur = null;
  let depth = 0;
  const due = (ms) => {
    let d = Math.max(0, Number(ms) || 0);
    if (depth >= 5) d = Math.max(d, 4);
    // whole milliseconds, as a browser's deadlines: timers set in one task's same millisecond tie, and run in the
    // order they were set (a zero timer set by the first of them runs after the others)
    return Math.floor(cur ?? __ui_now()) + d;
  };
  g.setTimeout = (fn, ms = 0, ...args) => {
    const id = seq++;
    timers.set(id, { at: due(ms), fn, args, depth: depth + 1 });
    return id;
  };
  g.setInterval = (fn, ms = 0, ...args) => {
    const id = seq++;
    const tick = () => {
      fn(...args);
      // cleared from its own callback: not set again
      if (!cleared.delete(id)) timers.set(id, { at: due(ms), fn: tick, args: [], depth: depth + 1, interval: true });
    };
    timers.set(id, { at: due(ms), fn: tick, args: [], depth: depth + 1, interval: true });
    return id;
  };
  const cleared = new Set();
  let running = 0;
  // one list for both, as a browser's: either clears a timeout or an interval, from its own callback too
  g.clearInterval = g.clearTimeout = (id) => {
    if (!timers.delete(id) && id === running) cleared.add(id);
  };
  // the earliest timer due at `now` (the first set, of those due together), run alone: the host calls again while this
  // answers 0, so each timer is its own call and the microtasks it queued run before the next timer, as a browser's
  // tasks do. Answers 0 when another is due at `now`, else ms to the next one, or -1 when none is set
  g.__ui_timers = (now) => {
    let first = null;
    for (const e of timers) if (e[1].at <= now && (first === null || e[1].at < first[1].at)) first = e;
    if (first) {
      timers.delete(first[0]);
      // no further back than a frame: a timer running late (a stalled frame) sets the next one from about now
      cur = Math.max(first[1].at, now - 20);
      depth = first[1].depth;
      running = first[0];
      try {
        first[1].fn(...first[1].args);
      } catch (e) {
        console.error(e);
      }
      cur = null;
      depth = 0;
      running = 0;
      cleared.delete(first[0]);
    }
    let next = -1;
    for (const t of timers.values()) next = next < 0 ? Math.max(0, t.at - now) : Math.min(next, Math.max(0, t.at - now));
    return next;
  };
  g.performance = { now: () => __ui_now() };
  g.process = { env: { NODE_ENV: "production" } };
  g.__ui_frame = 0;
  g.__ui_setFrame = (n) => void (g.__ui_frame = n);
  // Number's toLocaleString through one Intl.NumberFormat per locale and options, kept (JavaScriptCore made a
  // formatter at every call: a counter formatted each frame was most of its frame). Only where there's an Intl
  if (typeof Intl !== "undefined" && Intl.NumberFormat) {
    const formats = new Map();
    const toLocale = Number.prototype.toLocaleString;
    Object.defineProperty(Number.prototype, "toLocaleString", {
      value: function toLocaleString(locales, options) {
        "use strict";
        if (typeof this !== "number") return toLocale.call(this, locales, options);
        let key;
        if (options === undefined) key = locales === undefined ? "" : typeof locales === "string" ? locales : null;
        else if (typeof options === "object" && options !== null && (locales === undefined || typeof locales === "string")) {
          try {
            key = (locales || "") + "\u0000" + JSON.stringify(options);
          } catch {
            key = null;
          }
        } else key = null;
        if (key === null) return toLocale.call(this, locales, options);
        let f = formats.get(key);
        if (f === undefined) {
          f = new Intl.NumberFormat(locales, options);
          if (formats.size < 64) formats.set(key, f);
        }
        return f.format(this);
      },
      writable: true,
      configurable: true,
    });
  }
})();
