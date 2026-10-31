// crypto (getRandomValues's shape and limit, randomUUID's form), performance marks and measures, and the console's
// methods all being there.
// proves: wintertc.crypto, api.Performance.mark, api.Performance.measure, api.Performance.getEntriesByName, api.Performance.getEntriesByType, api.Performance.clearMarks, api.Performance.timeOrigin, api.console, api.console.count_static, api.console.time_static, api.console.group_static, api.console.table_static, api.console.assert_static
import { useEffect } from "react";
import { trace } from "parity";
export const steps = [];
export default function Case() {
  useEffect(() => {
    const a = crypto.getRandomValues(new Uint32Array(4));
    trace("crypto", "values", [a.length, a instanceof Uint32Array]);
    try {
      crypto.getRandomValues(new Uint8Array(70000));
    } catch (e) {
      trace("crypto", "limit", (e as DOMException).name);
    }
    trace("crypto", "uuid", /^[0-9a-f]{8}-[0-9a-f]{4}-4[0-9a-f]{3}-[89ab][0-9a-f]{3}-[0-9a-f]{12}$/.test(crypto.randomUUID()));
    performance.mark("a");
    performance.mark("b", { detail: { k: 1 } });
    const m = performance.measure("a-to-b", "a", "b");
    trace("perf", "measure", [m.name, m.entryType, m.duration >= 0, performance.getEntriesByName("b")[0].detail]);
    trace("perf", "types", [performance.getEntriesByType("mark").length, performance.getEntriesByType("measure").length, typeof performance.timeOrigin]);
    performance.clearMarks("a");
    trace("perf", "cleared", performance.getEntriesByType("mark").map((e) => e.name));
    performance.clearMarks();
    performance.clearMeasures();
    trace("console", "methods", ["log", "info", "warn", "error", "debug", "trace", "assert", "count", "countReset", "time", "timeEnd", "timeLog", "group", "groupCollapsed", "groupEnd", "table", "dir", "clear"].filter((k) => typeof (console as any)[k] !== "function"));
  }, []);
  return <div data-testid="root" className="h-full w-full bg-white p-4"><div data-testid="box" className="h-8 w-32 bg-fuchsia-300" /></div>;
}
