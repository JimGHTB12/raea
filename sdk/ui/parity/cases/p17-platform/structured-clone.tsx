// structuredClone: Map, Set, Date, RegExp, typed arrays sharing a buffer, nested cycles, Errors, boxed primitives and
// class instances as plain objects; a function or a symbol throws DataCloneError (a DOMException).
// proves: api.structuredClone, wintertc.structuredClone, api.DOMException, api.DOMException.name, api.DOMException.message
import { useEffect } from "react";
import { trace } from "parity";
class Point {
  constructor(public x: number, public y: number) {}
  len() { return Math.hypot(this.x, this.y); }
}
export const steps = [];
export default function Case() {
  useEffect(() => {
    const buf = new ArrayBuffer(8);
    const src: any = { m: new Map<unknown, unknown>([["a", 1], [2, { b: true }]]), s: new Set([1, "two"]), d: new Date(86400000), r: /ab+c/gi, u8: new Uint8Array(buf, 2, 4), f32: new Float32Array([1.5, -2]), list: [1, , 3], n: null, u: undefined, big: 12n, boxed: new String("s"), p: new Point(3, 4) };
    src.self = src;
    src.u16 = new Uint16Array(buf);
    src.u8[0] = 9;
    const c = structuredClone(src);
    trace("clone", "map", [c.m instanceof Map, c.m.get("a"), c.m.get(2).b, c.m !== src.m]);
    trace("clone", "set", [...c.s]);
    trace("clone", "date", [c.d instanceof Date, c.d.getTime()]);
    trace("clone", "regexp", [c.r.source, c.r.flags, c.r.lastIndex]);
    trace("clone", "typed", [c.u8.byteOffset, c.u8.length, c.u8[0], c.u8.buffer === c.u16.buffer, c.u16.length, [...c.f32]]);
    trace("clone", "cycle", [c.self === c, c.self !== src]);
    trace("clone", "holes", [c.list.length, 1 in c.list, "u" in c, c.u === undefined, c.n, typeof c.big, c.big === 12n]);
    trace("clone", "boxed", [typeof c.boxed, String(c.boxed)]);
    trace("clone", "class", [c.p instanceof Point, c.p.x, c.p.y, typeof c.p.len]);
    const e = structuredClone(new RangeError("too far", { cause: "x" }));
    trace("clone", "error", [e instanceof RangeError, e.name, e.message, (e as any).cause]);
    for (const bad of [() => 1, Symbol("s"), { f() {} }]) {
      try {
        structuredClone(bad);
        trace("clone", "bad", "no throw");
      } catch (x) {
        const ex = x as DOMException;
        trace("clone", "bad", [ex instanceof DOMException, ex.name, ex.code]);
      }
    }
  }, []);
  return <div data-testid="root" className="h-full w-full bg-white p-4"><div data-testid="box" className="h-8 w-32 bg-violet-300" /></div>;
}
