// Handles (components, resources, events, states, assets), plugins and schedules.
import { declare, type Declared, type Schema } from "../typed";
import { BevyApp } from "./all";

/** An entity, as BRP writes it (Entity::to_bits). */
export type Entity = number;

interface TypeHandle<K extends string, T> {
  readonly kind: K;
  /** the type path the registry knows it by */
  readonly path: string;
  /** phantom: the value's type */
  readonly __value?: T;
}

export type Component<T> = TypeHandle<"component", T>;

export type Resource<T> = TypeHandle<"resource", T>;

export type Event<T> = TypeHandle<"event", T>;

export type Message<T> = TypeHandle<"message", T>;

export type State<T> = TypeHandle<"state", T>;

/** A component's handle by its type path; with a schema, a component declared in TypeScript with its fields' storage
 *  types (typed.ts: `component("Health", { hp: u16 })`), the same kind of handle, usable in the same query. */
export function component<T>(path: string): Component<T>;

export function component<S extends Schema>(name: string, schema: S): Declared<S>;

export function component(path: string, schema?: Schema): Component<unknown> {
  return schema ? declare(path, schema) : { kind: "component", path };
}

export const resource = <T>(path: string): Resource<T> => ({ kind: "resource", path });

export const event = <T>(path: string): Event<T> => ({ kind: "event", path });

export const message = <T>(path: string): Message<T> => ({ kind: "message", path });

export const state = <T>(path: string): State<T> => ({ kind: "state", path });

/** An asset by its path (absolute, or from the edit's folder): what a handle field takes (a Mesh3d's, a material's
 *  texture), loaded before the frame that names it; `useAsset(h)` for its size. Typed by the asset it names. */
export type Handle<T> = string & { readonly __asset?: T };

export const asset = <T = unknown>(path: string): Handle<T> => path as Handle<T>;

/** A component with its value, for spawn and insert. */
export interface With {
  path: string;
  value: unknown;
}

export function with_<T>(c: Component<T>, value: T): With {
  return { path: c.path, value };
}

export { with_ as with };

/** A plugin written in TypeScript, as bevy_app's Plugin trait: `build` adds its systems, resources, observers and
 *  states to the App; once every plugin is built, each one's `finish` runs, then each one's `cleanup`, as App::run
 *  does. Unique by `name` unless `isUnique` is false (Bevy panics on a unique plugin added twice; this throws). */
export interface Plugin {
  name: string;
  build(app: BevyApp): void;
  finish?(app: BevyApp): void;
  cleanup?(app: BevyApp): void;
  isUnique?: boolean;
}

/** A TypeScript plugin (docs/sdk.feature S6c): `definePlugin({ name: "hud", build(app) { app.addSystems(Update, ...) } })`. */
export const definePlugin = (p: Plugin): Plugin => p;

/** A plugin of any author: a Rust plugin in raea's plugin build by its declared name (package.json raea.plugins:
 *  "avian", or a half: "lightyear_server") or by its generated handles (`avian` from avian.gen.tsx, which carries
 *  `plugin: "avian"`), a TypeScript Plugin, or a group of them. */
export type PluginRef = string | { readonly plugin: string } | Plugin | PluginGroup;

/** Plugins added together, as Bevy's PluginGroup: `group.set(p)` replaces the member of p's name, `disable(name)`
 *  leaves one out, and the App adds the rest in order. */
export class PluginGroup {
  private readonly members: { key: string; plugin: string | Plugin; on: boolean }[];
  constructor(readonly name: string, plugins: (string | { readonly plugin: string } | Plugin)[]) {
    this.members = plugins.map((p) => {
      const plugin = rustName(p) ?? (p as Plugin);
      return { key: typeof plugin === "string" ? plugin : plugin.name, plugin, on: true };
    });
  }
  set(p: Plugin): PluginGroup {
    const m = this.member(p.name);
    m.plugin = p;
    return this;
  }
  disable(name: string): PluginGroup {
    this.member(name).on = false;
    return this;
  }
  enable(name: string): PluginGroup {
    this.member(name).on = true;
    return this;
  }
  build(): (string | Plugin)[] {
    return this.members.filter((m) => m.on).map((m) => m.plugin);
  }
  private member(name: string) {
    const m = this.members.find((x) => x.key === name);
    if (!m) throw new Error(`PluginGroup ${this.name}: no plugin ${JSON.stringify(name)} (it has ${this.members.map((x) => x.key).join(", ")})`);
    return m;
  }
}

function rustName(p: unknown): string | null {
  if (typeof p === "string") return p;
  const named = p as { plugin?: unknown; build?: unknown };
  return typeof named.plugin === "string" && typeof named.build !== "function" ? named.plugin : null;
}

export function flatten(refs: PluginRef[]): (string | Plugin)[] {
  return refs.flatMap((r) => (r instanceof PluginGroup ? r.build() : [rustName(r) ?? (r as Plugin)]));
}

/** Bevy's schedules by their labels, for `app.addSystems(Update, ...)` as in Rust. */
export const PreStartup = "PreStartup", Startup = "Startup", PostStartup = "PostStartup";

/** The frame's schedules, in the order Bevy runs them (with StateTransition and the fixed step's inside). */
export const First = "First", PreUpdate = "PreUpdate", StateTransition = "StateTransition", RunFixedMainLoop = "RunFixedMainLoop";

/** The fixed step's schedules, run once a frame at the clock's step. */
export const FixedFirst = "FixedFirst", FixedPreUpdate = "FixedPreUpdate", FixedUpdate = "FixedUpdate", FixedPostUpdate = "FixedPostUpdate", FixedLast = "FixedLast";

/** The frame's main schedules: Update for game logic, then SpawnScene, PostUpdate and Last. */
export const Update = "Update", SpawnScene = "SpawnScene", PostUpdate = "PostUpdate", Last = "Last";

/** Bevy's startup schedules: a system added to one runs once, before the App's next frame. */
export const STARTUP = [PreStartup, Startup, PostStartup];

export interface OpenOptions {
  /** the App's plugins, in order: Rust ones by the name this build registered them under (a declaration's name, or its
   *  halves: lightyear_server), compiled in and added before Startup; TypeScript ones (definePlugin) built after */
  plugins: PluginRef[];
  /** frames a second: one step is 1/fps of Time (default 60) */
  fps?: number;
  /** serve BRP over HTTP on this port (a dev tool, a BRP client, curl) */
  serve?: number;
}
