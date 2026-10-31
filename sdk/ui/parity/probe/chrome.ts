// The probe on the oracle's side: React DOM in headless Chrome. The runner reads window.__parity over CDP.
const w = window as unknown as { __parity: { side: string; trace: unknown[] } };
w.__parity ??= { side: "chrome", trace: [] };
export const side = w.__parity.side;
export function trace(ev: string, on: string, data?: unknown) {
  w.__parity.trace.push(data === undefined ? { ev, on } : { ev, on, data: JSON.parse(JSON.stringify(data)) });
}
