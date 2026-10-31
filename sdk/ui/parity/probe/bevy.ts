// The probe on Bevy's side: @raea/ui on raea-page-host. The trace is read through a page action (`parity.read`, run over
// BRP page.action+watch), the same way an agent reads a page.
import { emit, on } from "@raea/ui/wire";

export const side = "bevy";
const log: unknown[] = [];
export function trace(ev: string, on: string, data?: unknown) {
  log.push(data === undefined ? { ev, on } : { ev, on, data: JSON.parse(JSON.stringify(data ?? null)) });
}
// which JS engine runs the bundle, from the shape of a stack trace: JavaScriptCore writes "fn@file:line:col", QuickJS
// "    at fn (file:line)"; the runner checks it against the column it was asked for
function engine(): string {
  const st = String(new Error("probe").stack ?? "");
  return /^\s+at /m.test(st) ? "quickjs" : /@/.test(st) ? "jsc" : "unknown";
}
emit("actions", [{ name: "parity.read", about: "the case's event trace so far", args: {} }]);
on<{ id: number; name: string }>("action", ({ id, name }) => {
  if (name === "parity.read") emit("actionDone", { id, value: { trace: log, engine: engine(), now: performance.now() } });
  else emit("actionDone", { id, error: `no action ${name}` });
});
