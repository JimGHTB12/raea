// A stand-in for the host module (apps/legacy/shell-bevy/src/tsx.rs) in TypeScript: it takes each batch as the host does
// (`{v, ops}`, the version checked), applies the ops to a tree of nodes as the host's entities would stand, and is
// strict where the host is lenient (an op on an id it doesn't know, a second create of an id, a listen twice, an
// update with nothing in it all throw), so a driver's mistake fails a test instead of passing quietly. Events go back
// through the wire's own `__ui_deliver`, as the host delivers them.
import { OPS_VERSION, type Ev, type Op } from "../src/wire";

export type N = { kind: string; props: Record<string, unknown>; text?: string; on: Set<Ev>; children: number[]; parent?: number };

export class Host {
  nodes = new Map<number, N>([[0, { kind: "#root", props: {}, on: new Set(), children: [] }]]);
  batches: Op[][] = [];
  emits: [string, unknown][] = [];
  calls: [number, string, unknown][] = [];

  constructor() {
    const g = globalThis as any;
    g.__ui_flush = (json: string) => {
      const b = JSON.parse(json);
      if (b.v !== OPS_VERSION) throw new Error(`a batch of op version ${b.v}; the host speaks ${OPS_VERSION}`);
      this.batches.push(b.ops);
      for (const op of b.ops) this.apply(op);
    };
    g.__ui_emit = (name: string, json: string) => this.emits.push([name, JSON.parse(json)]);
    g.__ui_call = (id: number, name: string, json: string) => this.calls.push([id, name, JSON.parse(json)]);
  }

  private get(id: number): N {
    const n = this.nodes.get(id);
    if (!n) throw new Error(`no element ${id}`);
    return n;
  }
  private detach(c: number) {
    const n = this.get(c);
    if (n.parent === undefined) return;
    const k = this.get(n.parent).children;
    k.splice(k.indexOf(c), 1);
    n.parent = undefined;
  }
  private drop(c: number) {
    for (const k of this.get(c).children) this.drop(k);
    this.nodes.delete(c);
  }
  apply(op: Op) {
    switch (op.op) {
      case "clear":
        for (const c of [...this.get(0).children]) {
          this.detach(c);
          this.drop(c);
        }
        break;
      case "create":
        if (this.nodes.has(op.id)) throw new Error(`element ${op.id} made twice`);
        this.nodes.set(op.id, { kind: op.kind, props: { ...op.props }, text: op.text, on: new Set(), children: [] });
        break;
      case "append":
      case "insert": {
        const p = this.get(op.parent);
        this.detach(op.child);
        const at = op.op === "insert" ? p.children.indexOf(op.before) : -1;
        if (op.op === "insert" && at < 0) throw new Error(`insert before ${op.before}, which isn't under ${op.parent}`);
        if (at < 0) p.children.push(op.child);
        else p.children.splice(at, 0, op.child);
        this.get(op.child).parent = op.parent;
        break;
      }
      case "remove":
        this.detach(op.id);
        this.drop(op.id);
        break;
      case "update": {
        const n = this.get(op.id);
        if (!op.set && !op.unset && op.text === undefined) throw new Error(`an empty update of ${op.id}`);
        Object.assign(n.props, op.set ?? {});
        for (const k of op.unset ?? []) delete n.props[k];
        if (op.text !== undefined) n.text = op.text;
        break;
      }
      case "listen":
        for (const e of op.events) {
          if (this.get(op.id).on.has(e)) throw new Error(`${op.id} listens for ${e} twice`);
          this.get(op.id).on.add(e);
        }
        break;
      case "unlisten":
        for (const e of op.events) {
          if (!this.get(op.id).on.delete(e)) throw new Error(`${op.id} unlistens ${e}, which it didn't listen for`);
        }
        break;
    }
  }

  /** The tree as the host's entities stand, ids left out (two drivers number their elements differently). */
  shape(id = 0): unknown {
    const x = this.get(id);
    const props = Object.fromEntries(Object.entries(x.props).sort(([a], [b]) => (a < b ? -1 : 1)));
    return [x.kind, props, x.text ?? null, [...x.on].sort(), x.children.map((c) => this.shape(c))];
  }
  /** The element with this data-testid. */
  tid(t: string): number {
    for (const [id, x] of this.nodes) if (x.props["data-testid"] === t) return id;
    throw new Error(`no element ${t}`);
  }
  /** An element's event, as the host sends it (only to an element that listens for it, as the host does). */
  send(t: string | number, event: Ev, payload: unknown = null) {
    const id = typeof t === "number" ? t : this.tid(t);
    if (!this.get(id).on.has(event)) throw new Error(`${t} doesn't listen for ${event}`);
    (globalThis as any).__ui_deliver(JSON.stringify({ t: "event", id, event, payload }));
  }
  named(name: string, value: unknown) {
    (globalThis as any).__ui_deliver(JSON.stringify({ t: "named", name, value }));
  }
  answer(id: number, value: unknown) {
    (globalThis as any).__ui_deliver(JSON.stringify({ t: "answer", id, ok: true, value }));
  }
  /** The ops since batch `from`. */
  since(from: number): Op[] {
    return this.batches.slice(from).flat();
  }
}

export const kinds = (ops: Op[]) => ops.reduce((a: Record<string, number>, o) => ((a[o.op] = (a[o.op] ?? 0) + 1), a), {});
