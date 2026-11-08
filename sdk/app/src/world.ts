// raea/world: a live game's TypeScript reaching the Bevy World its scene draws in (docs/bevy-ts.feature G16).
// Every Bevy Remote Protocol method (world.query, world.get_components, world.mutate_components, world.spawn_entity,
// world.trigger_event, a compiled-in plugin's methods such as avian3d/cast_ray...) on anything Reflect and
// registered, through the same layer BevyApp programs and void's spaces use (engine/bevy-world). The game's step runs
// before the Engine updates and holds no handle on its World, so a call is queued: `send` gives a ticket, the Engine
// runs it at the start of its next update, and its answer is there from the next frame on. An entity in params
// (`entity`, `parent`, each of `entities`) may be a part's name (<Mesh name>, <Entity name>) or row id.
//
// In a component, read an answer with useWorldAnswer(ticket): the component renders again the frame it arrives, and
// nothing else does. `world.answer(ticket)` is the plain read for code outside React (a game's step, a system); a
// component that reads it is only as fresh as whatever else makes it render.
//
//   const t = world.send("world.get_components", { entity: "ball", components: [TRANSFORM] });
//   function Ball() { const got = useWorldAnswer(t); ... got?.value ... }
import { useCallback, useSyncExternalStore } from "react";
import { onClockTick, useClockState } from "./clock";
import { native } from "./native";

type Call = (name: string, args: unknown) => unknown;
const call: Call = (name, args) => (native as unknown as Call)(name, args);

/** A queued call's answer: its value, or the method's error. */
export type Answer<T> = { value: T; error?: undefined } | { value?: undefined; error: string };

// the answers come in by ticket, once each, and never change: kept here, a reader gets the same object every time
// (useSyncExternalStore compares snapshots by identity), and an answer outlives the Engine's queue (it keeps the last
// few hundred)
const g = globalThis as { __raea_world_answers?: Map<number, Answer<unknown>> };
const answers: Map<number, Answer<unknown>> = (g.__raea_world_answers ??= new Map());

export const world = {
  /** A BRP method queued for the Engine's next update: its ticket. */
  send(method: string, params?: unknown): number {
    return call("bevy.world.send", params === undefined ? { method } : { method, params }) as number;
  },
  /** The answer to a ticket once the Engine has run its call, undefined before. The plain read, for code outside
   *  React; inside a component, useWorldAnswer renders it again when the answer arrives. */
  answer<T = unknown>(ticket: number): Answer<T> | undefined {
    let a = answers.get(ticket);
    if (a === undefined) {
      a = (call("bevy.world.answer", { ticket }) as Answer<unknown> | null) ?? undefined;
      if (a !== undefined) answers.set(ticket, a);
    }
    return a as Answer<T> | undefined;
  },
};

/** A queued call's answer, read in a component: undefined until the Engine has run the call, then its answer, the
 *  component rendering again on the frame it arrives (in live play and in a render alike). Only the components that
 *  read it render again. */
export function useWorldAnswer<T = unknown>(ticket: number): Answer<T> | undefined {
  const clock = useClockState("useWorldAnswer");
  const subscribe = useCallback(
    (changed: () => void) => {
      if (answers.has(ticket)) return () => {};
      const stop = onClockTick(clock, () => {
        if (world.answer(ticket) === undefined) return;
        stop();
        changed();
      });
      return stop;
    },
    [clock, ticket],
  );
  return useSyncExternalStore(subscribe, () => world.answer<T>(ticket));
}
