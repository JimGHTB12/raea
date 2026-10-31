// /ecs's typed layer (docs/sdk.feature S2d): components declared with storage
// types, queries whose bindings TypeScript infers (`write` views writable, `read` views readonly, `without` and `has`
// filtering), and systems whose structural changes go through commands applied after them, over /ecs's World (Bevy's
// own when the engine is there, TypeScript's tables under bun). A declared component is a bevy.ts handle too, so a
// query mixes it with a plugin's generated handles (`read(avian.LinearVelocity)`: its value as the World has it).
//
// A view from a query lives for one step of its loop: kept past it, it throws ("a view escaped its query"), so code
// never depends on a row being an object of its own.

import { World as MotionWorld, createWorld } from "./ecs";
import type { Component as Handle } from "./bevy";

// ---- storage types ----
export type Kind = "f32" | "f64" | "i32" | "u8" | "u16" | "u32" | "bool" | "str" | "ent";
export interface Field<K extends Kind = Kind> { kind: K }
export const f32: Field<"f32"> = { kind: "f32" };
export const f64: Field<"f64"> = { kind: "f64" };
export const i32: Field<"i32"> = { kind: "i32" };
export const u8: Field<"u8"> = { kind: "u8" };
export const u16: Field<"u16"> = { kind: "u16" };
export const u32: Field<"u32"> = { kind: "u32" };
export const bool: Field<"bool"> = { kind: "bool" };
export const str: Field<"str"> = { kind: "str" };
/** an entity handle stored in a field */
export const ent: Field<"ent"> = { kind: "ent" };

export type Schema = Record<string, Field>;
type ValueOf<F> = F extends Field<"bool"> ? boolean : F extends Field<"str"> ? string : number;
/** a schema's values: what a component or a table row holds */
export type Values<S extends Schema> = { -readonly [K in keyof S]: ValueOf<S[K]> };

/** Coerce a value to its storage type, as the column holding it would (f32 rounds, u8 wraps, bool is a bool). */
export function store(kind: Kind, v: unknown): number | string | boolean {
  switch (kind) {
    case "f32": return Math.fround(Number(v));
    case "f64": return Number(v);
    case "i32": return Number(v) | 0;
    case "u8": return Number(v) & 0xff;
    case "u16": return Number(v) & 0xffff;
    case "u32":
    case "ent": return Number(v) >>> 0;
    case "bool": return Boolean(v);
    case "str": return String(v ?? "");
  }
}

// ---- components ----
/** A component declared here: a name, its fields' storage types, and bevy.ts's handle (`kind`, `path`: the name), so
 *  it's the same kind of handle as a plugin's generated ones and goes wherever they go (useComponent, BevyApp). */
export interface Declared<S extends Schema = Schema> extends Handle<Values<S>> {
  readonly name: string;
  readonly schema: S;
  readonly tag: false;
  /** a value of this component, checked and coerced to its storage types */
  of(v: Values<S>): [Declared<S>, Values<S>];
}
export interface Tag {
  readonly name: string;
  readonly schema: {};
  readonly tag: true;
}

/** A component declared here: a name the engine knows it by, and the storage type of each field (bevy.ts's
 *  `component(name, schema)`, the overload with a schema). */
export function declare<S extends Schema>(name: string, schema: S): Declared<S> {
  const c: Declared<S> = {
    kind: "component", path: name,
    name, schema, tag: false,
    of(v) {
      const out = {} as Values<S>;
      for (const k in schema) (out as any)[k] = store(schema[k].kind, (v as any)[k]);
      return [c, out];
    },
  };
  return c;
}

/** A tag: membership without fields. */
export function tag(name: string): Tag {
  return { name, schema: {}, tag: true };
}

// ---- query terms ----
/** What a term names: a component declared here, a tag, or any bevy.ts handle (a plugin's generated one, say), whose
 *  value is its own type with no storage types. */
export type Named = Declared<any> | Tag | Handle<any>;
const nameOf = (c: Named): string => ("name" in c ? c.name : c.path);

export interface Read<C extends Declared<any> | Handle<any>> { readonly t: "read"; readonly c: C }
export interface Write<C extends Declared<any> | Handle<any>> { readonly t: "write"; readonly c: C }
export interface Without { readonly t: "without"; readonly c: Named }
export interface HasTerm { readonly t: "with"; readonly c: Named }
export interface EntityTerm { readonly t: "entity" }
export type Term = Read<any> | Write<any> | Without | HasTerm | EntityTerm;

export const read = <C extends Declared<any> | Handle<any>>(c: C): Read<C> => ({ t: "read", c });
export const write = <C extends Declared<any> | Handle<any>>(c: C): Write<C> => ({ t: "write", c });
export const without = (c: Named): Without => ({ t: "without", c });
/** an entity must have it, without binding it (a tag, or a component the loop doesn't read) */
export const has = (c: Named): HasTerm => ({ t: "with", c });
/** binds the entity's handle */
export const entity: EntityTerm = { t: "entity" };

type ViewOf<T> = T extends Read<Declared<infer S>> ? Readonly<Values<S>>
  : T extends Write<Declared<infer S>> ? Values<S>
  : T extends Read<Handle<infer V>> ? Readonly<V>
  : T extends Write<Handle<infer V>> ? V
  : T extends EntityTerm ? number
  : never;
/** the bindings a query's terms give, filters (without, has) left out */
export type Bindings<Ts extends readonly Term[]> =
  Ts extends readonly [infer H, ...infer R extends readonly Term[]]
    ? ViewOf<H> extends never ? Bindings<R> : [ViewOf<H>, ...Bindings<R>]
    : [];

// ---- views that live for one step ----
let generation = 0;
/** A view's lifetime, for row loops outside this file: a new row's mark, the mark now, and the
 *  row left. */
export const rows = { born: () => ++generation, now: () => generation, leave: () => { generation++; } };

export function view<S extends Schema>(schema: S, get: (k: string) => unknown, set: ((k: string, v: unknown) => void) | null, gen: () => number, born: number, what: string): Values<S> {
  const o: any = {};
  for (const k of Object.keys(schema)) {
    Object.defineProperty(o, k, {
      enumerable: true,
      get() {
        if (gen() !== born) throw new Error(`a view escaped its query: ${what}.${k} read after its row was left (copy() it to keep it)`);
        return get(k);
      },
      set(v) {
        if (gen() !== born) throw new Error(`a view escaped its query: ${what}.${k} written after its row was left`);
        if (!set) throw new Error(`${what} is read-only in this query (ask for write(${what}))`);
        set(k, store(schema[k].kind, v));
      },
    });
  }
  return o;
}

/** A view's values as a plain object of its own, to keep past its row. */
export function copy<T extends object>(v: T): T {
  return { ...v };
}

// ---- the world ----
type Spawnable = [Declared<any> | Handle<any>, unknown] | Tag;

/** Structural changes made during a system, applied when it ends. */
export class Commands {
  readonly ops: (() => void)[] = [];
  constructor(private readonly w: Data) {}
  spawn(...parts: Spawnable[]): void { this.ops.push(() => { this.w.spawn(...parts); }); }
  despawn(e: number): void { this.ops.push(() => this.w.motion.despawn(e)); }
  insert(e: number, ...parts: Spawnable[]): void { this.ops.push(() => this.w.insert(e, ...parts)); }
  remove(e: number, ...cs: Named[]): void { this.ops.push(() => this.w.motion.remove(e, ...cs.map(nameOf))); }
}

export interface SystemInfo { name: string; reads: string[]; writes: string[] }

/** A typed layer over raea's World: components by declaration, queries by terms, systems by access. */
export class Data {
  readonly systems: SystemInfo[] = [];
  constructor(readonly motion: MotionWorld) {}

  /** A new entity with these components (`Position.of({x, y})`) and tags; its handle. */
  spawn(...parts: Spawnable[]): number {
    return this.motion.add(this.parts(parts));
  }
  insert(e: number, ...parts: Spawnable[]): void {
    this.motion.insert(e, this.parts(parts));
  }
  private parts(parts: Spawnable[]): Record<string, unknown> {
    const comps: Record<string, unknown> = {};
    for (const p of parts) {
      if (Array.isArray(p)) comps[nameOf(p[0])] = p[1];
      else comps[p.name] = true;
    }
    return comps;
  }

  /** One entity's component, or undefined when it hasn't it. */
  get<S extends Schema>(e: number, c: Declared<S>): Values<S> | undefined;
  get<V>(e: number, c: Handle<V>): V | undefined;
  get(e: number, c: Declared<any> | Handle<any>): unknown {
    return this.motion.components(e)[nameOf(c)];
  }

  /**
   * The entities with every bound and `has` component and none of the `without`, with views of what's bound, in the
   * order of the terms. A `write` view's changes go back to the world as the loop leaves its row.
   */
  *query<const Ts extends readonly Term[]>(...terms: Ts): Generator<Bindings<Ts>> {
    const need = terms.filter((t) => t.t === "read" || t.t === "write" || t.t === "with").map((t) => nameOf((t as HasTerm).c));
    const not = terms.filter((t) => t.t === "without").map((t) => nameOf((t as Without).c));
    const ids = this.motion.find({ with: need, without: not });
    for (const id of ids) {
      const comps = this.motion.components(id) as Record<string, any>;
      const born = ++generation;
      const changed = new Set<string>();
      const out: unknown[] = [];
      for (const t of terms) {
        if (t.t === "entity") out.push(id);
        else if (t.t === "read" || t.t === "write") {
          const name = nameOf(t.c);
          if (!("schema" in t.c)) {
            // a handle with no storage types (a plugin's): its value as the World has it (a tuple, a number, an
            // object), a copy; a `write` one goes back as the loop leaves the row
            const v = structuredClone(comps[name]);
            if (t.t === "write") {
              changed.add(name);
              comps[name] = v;
            }
            out.push(v);
            continue;
          }
          const vals = { ...(comps[name] ?? {}) };
          out.push(view(t.c.schema, (k) => vals[k], t.t === "write" ? (k, v) => { vals[k] = v; changed.add(name); comps[name] = vals; } : null, () => generation, born, name));
        }
      }
      yield out as Bindings<Ts>;
      generation++;
      if (changed.size) {
        const upd: Record<string, unknown> = {};
        for (const n of changed) upd[n] = comps[n];
        this.motion.insert(id, upd);
      }
    }
  }

  /**
   * A system: run on every frame the world steps (motion's `system`, so `world.at(frame)` is the same whenever it's
   * asked for), with commands applied after it. Its reads and writes are declared for whoever schedules systems.
   */
  system(name: string, access: { reads?: Named[]; writes?: Named[] }, fn: (w: Data, frame: number, commands: Commands) => void): this {
    this.systems.push({ name, reads: (access.reads ?? []).map(nameOf), writes: (access.writes ?? []).map(nameOf) });
    // motion replays systems on a copy of the world for `at(frame)`: run against the world it hands over
    this.motion.system((m, frame) => {
      const d = m === this.motion ? this : new Data(m);
      const cmd = new Commands(d);
      fn(d, frame, cmd);
      for (const op of cmd.ops) op();
    });
    return this;
  }

  /** Run `fn` once now, with its commands applied after it (a system outside the frame loop). */
  run(fn: (w: Data, commands: Commands) => void): void {
    const cmd = new Commands(this);
    fn(this, cmd);
    for (const op of cmd.ops) op();
  }
}

/** A typed world: over `w`, or a new one (native tables in the engine, TypeScript's under bun). */
export function typed(w?: MotionWorld | { native?: boolean }): Data {
  return new Data(w instanceof MotionWorld ? w : createWorld(w ?? {}));
}
