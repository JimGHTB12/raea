// BevyApp: the app a script drives.
import { native } from "../native";
import { Component, Condition, Entity, Event, Message, Observing, OneShot, OpenOptions, Plugin, PluginRef, Resource, STARTUP, SetConfig, State, SystemOptions, Then, TsSystem, With, flatten, names } from "./all";

type Call = (name: string, args: unknown) => unknown;

const call: Call = (name, args) => (native as unknown as (n: string, a: unknown) => unknown)(name, args);

/** A headless Bevy App driven from TypeScript through Bevy's remote protocol: open it with its plugins, then spawn,
 * insert, query and step it, and add TypeScript systems to its schedules. */
export class BevyApp {
  /** the methods this App answers (BRP's, raea's and its plugins') */
  readonly methods: string[];
  private constructor(readonly id: number, methods: string[]) {
    this.methods = methods;
  }

  /** A headless App with these plugins; Startup has run, no frame has stepped. TypeScript plugins are built in order
   *  once the App is open, then finished and cleaned up; their Startup systems run before the first frame. */
  static open(o: OpenOptions): BevyApp {
    // raea's own plugins (render's looks, audio's mixer) are in every render and live play, never a headless App
    const own = o.plugins.find((p) => typeof p === "object" && p !== null && (p as { raea?: unknown }).raea === true) as { plugin: string } | undefined;
    if (own) throw new Error(`BevyApp.open: ${JSON.stringify(own.plugin)} is raea's own plugin, in every render and live play; a headless BevyApp has none of raea's drawing or sound (a project says it uses it in package.json raea.plugins)`);
    const all = flatten(o.plugins);
    const rust = all.filter((p): p is string => typeof p === "string");
    const r = call("bevy.app.open", { ...o, plugins: rust }) as { app: number; methods: string[] };
    const app = new BevyApp(r.app, r.methods);
    for (const name of rust) app.added.add(name);
    app.addPlugins(...all.filter((p): p is Plugin => typeof p !== "string"));
    return app;
  }

  private readonly added = new Set<string>();
  private startup: ((app: BevyApp, frame: number) => void)[] = [];

  /** Whether a plugin of this name is in the App (Bevy's is_plugin_added), Rust or TypeScript. */
  isPluginAdded(name: string): boolean {
    return this.added.has(name);
  }

  /** Adds TypeScript plugins and groups (Bevy's add_plugins): each built in order, then each finished, then each
   *  cleaned up. A Rust plugin is compiled in and must be named when the App opens; naming a new one here throws. */
  addPlugins(...refs: PluginRef[]): this {
    const built: Plugin[] = [];
    for (const p of flatten(refs)) {
      if (typeof p === "string") {
        if (!this.added.has(p)) throw new Error(`BevyApp.addPlugins: ${JSON.stringify(p)} is a Rust plugin; name it when the App opens (BevyApp.open({ plugins: [${JSON.stringify(p)}] })), as Rust plugins are added before Startup`);
        continue;
      }
      if (p.isUnique !== false && this.added.has(p.name)) throw new Error(`BevyApp.addPlugins: the plugin ${JSON.stringify(p.name)} is already added`);
      this.added.add(p.name);
      p.build(this);
      built.push(p);
    }
    for (const p of built) p.finish?.(this);
    for (const p of built) p.cleanup?.(this);
    return this;
  }

  /** Systems in a schedule (Bevy's add_systems), each as `system` adds one; one of the startup schedules (PreStartup,
   *  Startup, PostStartup) runs them once, before the App's next frame. Returns the App, to chain. */
  addSystems(schedule: string, systems: ((app: BevyApp, frame: number) => void) | ((app: BevyApp, frame: number) => void)[], o: SystemOptions = {}): this {
    const list = Array.isArray(systems) ? systems : [systems];
    if (STARTUP.includes(schedule)) this.startup.push(...list);
    else for (const s of list) this.system(schedule, s, list.length > 1 ? { ...o, name: undefined } : o);
    return this;
  }

  /** Bevy's insert_resource: the resource's value set (setResource). */
  insertResource<T>(r: Resource<T>, value: T): this {
    this.setResource(r, value);
    return this;
  }

  /** Bevy's add_observer: an observer of an event inside the App (observer). */
  addObserver<T>(e: Event<T> | string, o: { entity?: Entity; then?: Then[] } = {}): Observing<T> {
    return this.observer(e, o);
  }

  private runStartup(): void {
    if (this.startup.length === 0) return;
    const list = this.startup;
    this.startup = [];
    for (const s of list) s(this, this.lastFrame);
    this.flushQueued(this.lastFrame);
  }

  /** Apps stepped together, a frame at a time: a network's peers. Returns each one's frame. */
  static stepTogether(apps: BevyApp[], frames = 1): number[] {
    return call("bevy.app.step", { apps: apps.map((a) => a.id), frames }) as number[];
  }

  /** `frames` frames on. Returns the frame it's on. With TypeScript systems (`system`), each frame is run schedule
   *  by schedule and they run in theirs. */
  step(frames = 1): number {
    this.runStartup();
    if (this.systems.size === 0) return (call("bevy.app.step", { apps: [this.id], frames }) as number[])[0];
    let frame = 0;
    for (let i = 0; i < frames; i++) {
      // a system in a fixed schedule opens the fixed step up (bridge.rs run_until's `fixed`)
      const fixed = [...this.systems.keys()].some((k) => k.startsWith("Fixed"));
      for (const [schedule, list] of this.order()) {
        frame = (call("bevy.app.run", { app: this.id, until: schedule, fixed }) as { frame: number }).frame;
        // a set's condition is asked once a schedule run, before its first member (Bevy's configure_sets run_if)
        const setOk = new Map<string, boolean>();
        const passes = (set: string | undefined): boolean => {
          if (set === undefined) return true;
          if (!setOk.has(set)) setOk.set(set, this.sets.get(set)?.when?.(this, frame) ?? true);
          return setOk.get(set)!;
        };
        for (const sys of list) {
          if (!this.systemsLive.has(sys)) continue;
          if (!passes(sys.set)) continue;
          if (sys.when && !sys.when(this, frame)) continue;
          sys.run(this, frame);
          this.flushQueued(frame);
        }
        this.flushQueued(frame);
      }
      frame = (call("bevy.app.run", { app: this.id, fixed }) as { frame: number }).frame;
      this.flushQueued(frame);
    }
    return frame;
  }

  private readonly systems = new Map<string, TsSystem[]>();
  private readonly systemsLive = new Set<TsSystem>();
  private readonly sets = new Map<string, SetConfig>();
  private sorted = new Map<string, TsSystem[]>();
  private queued: OneShot[] = [];
  private schedules: string[] | null = null;
  private lastFrame = 0;

  /** A system written in TypeScript: `run` is called every frame right after Bevy's own systems of `schedule` (one of
   *  the frame's: "First", "PreUpdate", "StateTransition", "RunFixedMainLoop", the fixed step's "FixedFirst",
   *  "FixedPreUpdate", "FixedUpdate", "FixedPostUpdate", "FixedLast" (one fixed step a frame: the clock's step is the
   *  timestep), "Update", "SpawnScene", "PostUpdate", "Last"), on that frame's World, before the next schedule's; it queries, changes and observes through the App's
   *  methods (query, get, insert, call("world.mutate_components"), observe...). Returns a function that removes it.
   *
   *  `o` places it as Bevy's add_systems(...).run_if().before().after().in_set() would: `when` is its run condition
   *  (asked every frame before it runs; see `conditions`), `before`/`after` name other systems or sets of the same
   *  schedule it runs before or after (a cycle is refused), `set` puts it in a set that `configureSet` gives a
   *  condition and an order. Without an order, systems run in the order they were added. */
  system(schedule: string, run: (app: BevyApp, frame: number) => void, o: SystemOptions = {}): () => void {
    const known = this.frameSchedules();
    if (!known.includes(schedule)) throw new Error(`BevyApp.system: no schedule ${JSON.stringify(schedule)} in Bevy's frame (it runs: ${known.join(", ")})`);
    const list = this.systems.get(schedule) ?? [];
    if (o.name !== undefined && list.some((x) => x.name === o.name)) throw new Error(`BevyApp.system: ${schedule} already has a system named ${JSON.stringify(o.name)}`);
    const sys: TsSystem = { run, name: o.name, set: o.set, when: o.when, before: names(o.before), after: names(o.after) };
    list.push(sys);
    this.systems.set(schedule, list);
    this.systemsLive.add(sys);
    try {
      this.sort(schedule);
    } catch (e) {
      list.pop();
      this.systemsLive.delete(sys);
      if (list.length === 0) this.systems.delete(schedule);
      throw e;
    }
    return () => {
      const l = this.systems.get(schedule);
      if (!l) return;
      this.systemsLive.delete(sys);
      l.splice(l.indexOf(sys) >>> 0, 1);
      if (l.length === 0) this.systems.delete(schedule);
      this.sort(schedule);
    };
  }

  /** A set of TypeScript systems (Bevy's SystemSet with configure_sets): `when` is asked once per schedule run, before
   *  its first member, and a false skips them all; `before`/`after` order the whole set against systems or sets. */
  configureSet(name: string, o: { when?: Condition; before?: string | string[]; after?: string | string[] }): void {
    this.sets.set(name, { when: o.when, before: names(o.before), after: names(o.after) });
    for (const s of [...this.systems.keys()]) this.sort(s);
  }

  /** A one-shot system (Bevy's register_system + run_system): `run()` runs it now, on the World as it is; `queue()` runs
   *  it once the current TypeScript system returns (as Commands::run_system applies at the next sync point), or at the
   *  end of the frame if called outside a step. `remove()` unregisters it. */
  oneShot<R = void>(fn: (app: BevyApp, frame: number) => R): OneShot<R> {
    const app = this;
    const shot: OneShot<R> = {
      run: () => fn(app, app.lastFrame),
      queue: () => void app.queued.push(shot as OneShot),
      remove: () => void (app.queued = app.queued.filter((q) => q !== shot)),
    };
    return shot;
  }

  private flushQueued(frame: number): void {
    this.lastFrame = frame;
    // a queued system may queue another: run until none is left (Bevy's apply_deferred loop)
    for (let guard = 0; this.queued.length > 0; guard++) {
      if (guard > 1000) throw new Error("BevyApp.oneShot: queued systems kept queueing more (1000 rounds)");
      const q = this.queued;
      this.queued = [];
      for (const shot of q) shot.run();
    }
  }

  /** A schedule's systems in their run order: before/after edges (a set's on each member), ties by when added. */
  private sort(schedule: string): void {
    const list = this.systems.get(schedule) ?? [];
    const label = (s: TsSystem) => [s.name, s.set].filter((x): x is string => x !== undefined);
    const edges = new Map<TsSystem, Set<TsSystem>>(list.map((s) => [s, new Set()]));
    const matching = (n: string) => list.filter((s) => label(s).includes(n));
    for (const s of list) {
      const set = s.set !== undefined ? this.sets.get(s.set) : undefined;
      for (const b of [...s.before, ...(set?.before ?? [])]) for (const t of matching(b)) if (t !== s) edges.get(s)!.add(t);
      for (const a of [...s.after, ...(set?.after ?? [])]) for (const t of matching(a)) if (t !== s) edges.get(t)!.add(s);
    }
    const indeg = new Map(list.map((s) => [s, 0]));
    for (const [, outs] of edges) for (const t of outs) indeg.set(t, indeg.get(t)! + 1);
    const out: TsSystem[] = [];
    const ready = list.filter((s) => indeg.get(s) === 0);
    while (ready.length > 0) {
      ready.sort((a, b) => list.indexOf(a) - list.indexOf(b));
      const s = ready.shift()!;
      out.push(s);
      for (const t of edges.get(s)!) {
        indeg.set(t, indeg.get(t)! - 1);
        if (indeg.get(t) === 0) ready.push(t);
      }
    }
    if (out.length !== list.length) {
      const stuck = list.filter((s) => !out.includes(s)).map((s) => s.name ?? s.set ?? "(unnamed)");
      throw new Error(`BevyApp.system: ${schedule}'s before/after order has a cycle among ${stuck.join(", ")}`);
    }
    this.sorted.set(schedule, out);
  }

  /** The schedules a frame runs, in order (Bevy's MainScheduleOrder). */
  frameSchedules(): string[] {
    this.schedules ??= call("bevy.app.schedules", { app: this.id }) as string[];
    return this.schedules;
  }

  private order(): [string, TsSystem[]][] {
    return this.frameSchedules().filter((s) => this.systems.has(s)).map((s) => [s, [...(this.sorted.get(s) ?? this.systems.get(s)!)]]);
  }

  /** Any method of the App's table, run now: its answer, or a throw with the method's message. */
  call<A = unknown>(method: string, params?: unknown): A {
    return call("bevy.app.call", { app: this.id, method, params }) as A;
  }

  /** A new entity with these components. */
  spawn(...components: With[]): Entity {
    const r = this.call<{ entity: Entity }>("world.spawn_entity", { components: Object.fromEntries(components.map((c) => [c.path, c.value])) });
    return r.entity;
  }

  insert(entity: Entity, ...components: With[]): void {
    this.call("world.insert_components", { entity, components: Object.fromEntries(components.map((c) => [c.path, c.value])) });
  }

  /** One component of an entity, or undefined when it hasn't got one. */
  get<T>(entity: Entity, c: Component<T>): T | undefined {
    const r = this.call<{ components: Record<string, T> }>("world.get_components", { entity, components: [c.path], strict: false });
    return r.components[c.path];
  }

  /** Entities with every component in `has` (and none in `without`), with the `data` components' values. */
  query(o: { data?: Component<unknown>[]; has?: Component<unknown>[]; without?: Component<unknown>[] }): { entity: Entity; components: Record<string, unknown> }[] {
    return this.call("world.query", {
      data: { components: (o.data ?? []).map((c) => c.path) },
      filter: { with: (o.has ?? []).map((c) => c.path), without: (o.without ?? []).map((c) => c.path) },
      strict: false,
    });
  }

  resource<T>(r: Resource<T>): T {
    return this.call<{ value: T }>("world.get_resources", { resource: r.path }).value;
  }

  setResource<T>(r: Resource<T>, value: T): void {
    this.call("world.insert_resources", { resource: r.path, value });
  }

  /** Triggers an event: its observers run now. */
  trigger<T>(e: Event<T>, value: T): void {
    this.call("world.trigger_event", { event: e.path, value });
  }

  /** Writes a message, for its readers' next run. */
  write<T>(m: Message<T>, value: T): void {
    this.call("world.write_message", { message: m.path, value });
  }

  /** Starts observing an event (BRP's world.observe+watch). Each call of the returned function gives the events
   *  triggered since the last call, in order. `entity` limits it to events aimed at that entity. */
  observe<T>(e: Event<T>, entity?: Entity): () => T[] {
    const params = entity === undefined ? { event: e.path } : { event: e.path, entity };
    this.call("world.observe+watch", params);
    return () => (this.call<T[] | null>("world.observe+watch", params) ?? []);
  }

  /** A component hook as an observer inside the App: when `component` is added, inserted, discarded, removed or its
   *  entity despawned, `then`'s calls (BRP methods, "$entity" in their params standing for the entity) run right
   *  there, in the same flush, before anything else sees the World. `seen()` gives the entities it fired for since the
   *  last call; `stop()` removes it. */
  hook(on: "Add" | "Insert" | "Discard" | "Remove" | "Despawn", component: Component<unknown> | string, then: Then[] = []): Observing {
    return this.observing({ on, component: typeof component === "string" ? component : component.path, then });
  }

  /** An observer inside the App for a reflected event: on each trigger (or each aimed at `entity`), `then`'s calls run
   *  in the same flush, "$event" standing for the event's value and "$entity" for its target. */
  observer<T>(e: Event<T> | string, o: { entity?: Entity; then?: Then[] } = {}): Observing<T> {
    return this.observing({ event: typeof e === "string" ? e : e.path, entity: o.entity, then: o.then ?? [] }) as Observing<T>;
  }

  private observing(p: Record<string, unknown>): Observing {
    const key = `ts${++BevyApp.observers}`;
    this.call("raea/observe", { key, ...p });
    return {
      seen: () => this.call("raea/observed", { key }),
      stop: () => void this.call("raea/unobserve", { key }),
    };
  }
  private static observers = 0;

  /** A state's value (null before it exists). */
  state<T>(s: State<T>): T | null {
    return this.call<T | null>("raea/state.get", { state: s.path });
  }

  /** A state's next value: it changes at the next frame's StateTransition. */
  setState<T>(s: State<T>, value: T): void {
    this.call("raea/state.set", { state: s.path, value });
  }

  /** The App's registered types as JSON schema (BRP's registry.schema), filtered by crate. */
  schema(crates?: string[]): Record<string, unknown> {
    return this.call("registry.schema", crates ? { with_crates: crates } : undefined);
  }

  close(): void {
    call("bevy.app.close", { app: this.id });
  }
}
