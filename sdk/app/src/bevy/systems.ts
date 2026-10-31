// Systems' options, run conditions, observers, and Transform.
import { BevyApp, Entity, Resource, State, With, component, with_ } from "./all";

/** A BRP call an observer makes when it fires ("$entity" and "$event" in params are filled in). */
/** A run condition: asked on the frame's World before a system or set runs; false skips it this frame. */
export type Condition = (app: BevyApp, frame: number) => boolean;

export interface SystemOptions {
  /** its name, for other systems' before/after */
  name?: string;
  /** the set it's in (configureSet) */
  set?: string;
  /** its run condition */
  when?: Condition;
  /** systems or sets (by name) it runs before */
  before?: string | string[];
  /** systems or sets (by name) it runs after */
  after?: string | string[];
}

export interface OneShot<R = void> {
  run(): R;
  queue(): void;
  remove(): void;
}

export interface TsSystem {
  run: (app: BevyApp, frame: number) => void;
  name?: string;
  set?: string;
  when?: Condition;
  before: string[];
  after: string[];
}

export interface SetConfig {
  when?: Condition;
  before: string[];
  after: string[];
}

export function names(x: string | string[] | undefined): string[] {
  return x === undefined ? [] : Array.isArray(x) ? x : [x];
}

/** Bevy's common run conditions (bevy_ecs::schedule::common_conditions, bevy_state's in_state, bevy_time's on_timer),
 *  as Conditions for `system(..., { when })` and `configureSet`. Each call makes a fresh condition with its own memory
 *  (runOnce, resourceChanged, onTimer). */
export const conditions = {
  /** true the first time it's asked, then never */
  runOnce(): Condition {
    let done = false;
    return () => (done ? false : (done = true));
  },
  /** the resource exists in the World */
  resourceExists(r: Resource<unknown> | string): Condition {
    const path = typeof r === "string" ? r : r.path;
    return (app) => {
      try {
        app.call("world.get_resources", { resource: path });
        return true;
      } catch {
        return false;
      }
    };
  },
  /** the resource exists and equals `value` (compared as JSON) */
  resourceEquals<T>(r: Resource<T>, value: T): Condition {
    const want = JSON.stringify(value);
    return (app) => {
      try {
        return JSON.stringify(app.resource(r)) === want;
      } catch {
        return false;
      }
    };
  },
  /** the resource's value differs from the last time this condition saw it (true the first time it exists) */
  resourceChanged(r: Resource<unknown>): Condition {
    let last: string | undefined;
    return (app) => {
      let now: string | undefined;
      try {
        now = JSON.stringify(app.resource(r));
      } catch {
        now = undefined;
      }
      const changed = now !== undefined && now !== last;
      last = now;
      return changed;
    };
  },
  /** the state is `value` */
  inState<T>(s: State<T>, value: T): Condition {
    const want = JSON.stringify(value);
    return (app) => JSON.stringify(app.state(s)) === want;
  },
  /** every `seconds` of the App's clock (frame / fps), true on the frame the timer finishes */
  onTimer(seconds: number, fps: number): Condition {
    if (!(seconds > 0) || !(fps > 0)) throw new RangeError(`conditions.onTimer: seconds and fps are above 0, got ${seconds}, ${fps}`);
    let next = seconds;
    return (_app, frame) => {
      if (frame / fps + 1e-9 < next) return false;
      next += seconds;
      return true;
    };
  },
  not(c: Condition): Condition {
    return (app, f) => !c(app, f);
  },
  and(...cs: Condition[]): Condition {
    return (app, f) => cs.every((c) => c(app, f));
  },
  or(...cs: Condition[]): Condition {
    return (app, f) => cs.some((c) => c(app, f));
  },
};

export interface Then {
  method: string;
  params?: unknown;
}

/** An observer TypeScript put in the App: what it fired for since the last look, and its removal. */
export interface Observing<T = unknown> {
  seen(): { entity: Entity | null; event: T | null }[];
  stop(): void;
}

/** Bevy's own Transform (bevy_transform's): where a plugin's bodies start. avian reads a new body's place from it, so
 *  give a body a Transform, not a Position. Fields left out are the default (no rotation, scale 1).
 * @bevy bevy_transform::components::transform::Transform */
export const Transform = component<{ translation?: [number, number, number]; rotation?: [number, number, number, number]; scale?: [number, number, number] }>("bevy_transform::components::transform::Transform");

/** A Transform at a point. */
export function at(x: number, y: number, z: number): With {
  return with_(Transform, { translation: [x, y, z] });
}
