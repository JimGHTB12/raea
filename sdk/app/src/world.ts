// raea/world: a live game's TypeScript reaching the Bevy World its scene draws in (docs/bevy-ts.feature G16).
// Every Bevy Remote Protocol method (world.query, world.get_components, world.mutate_components, world.spawn_entity,
// world.trigger_event, a compiled-in plugin's methods such as avian3d/cast_ray...) on anything Reflect and
// registered, through the same layer BevyApp programs and void's spaces use (engine/bevy-world). The game's step runs
// before the Engine updates and holds no handle on its World, so a call is queued: `send` gives a ticket, the Engine
// runs it at the start of its next update, and `answer(ticket)` has its answer from the next frame on. An entity in
// params (`entity`, `parent`, each of `entities`) may be a part's name (<Mesh name>, <Entity name>) or row id.
//
//   const t = world.send("world.get_components", { entity: "ball", components: [TRANSFORM] });
//   ... a frame later: world.answer(t)?.value
import { native } from "./native";

type Call = (name: string, args: unknown) => unknown;
const call: Call = (name, args) => (native as unknown as Call)(name, args);

/** A queued call's answer: its value, or the method's error. */
export type Answer<T> = { value: T; error?: undefined } | { value?: undefined; error: string };

export const world = {
  /** A BRP method queued for the Engine's next update: its ticket. */
  send(method: string, params?: unknown): number {
    return call("bevy.world.send", params === undefined ? { method } : { method, params }) as number;
  },
  /** The answer to a ticket once the Engine has run its call, undefined before. */
  answer<T = unknown>(ticket: number): Answer<T> | undefined {
    return (call("bevy.world.answer", { ticket }) as Answer<T> | null) ?? undefined;
  },
};
