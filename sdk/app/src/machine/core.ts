// A machine's definition, its snapshot, deciders and the trace.
import { AnyMachine, Edge } from "./all";

// Step-driven state machines: the one core under game logic, playtests and (later) agent workflows (spec: "State
// machines, step by step"; sdk/react/docs/state-machines.md). Nothing here knows about games: games.ts adds per-entity
// stepping and input edges on top, and a game steps its machines once a frame.
//
// The design follows Bevy's seldom_state: each entity carries its own machine, and a transition is a trigger, a pure
// predicate polled every step, not an event someone sends. The definition is plain data in XState's shape (states /
// initial / entry / exit / after / on, `target` and `guard` accepted beside `to` and `when`), so what an agent knows of
// XState carries over; what differs is that guards are polled each step and delays are counted in steps.
//
// A transition is either guard-driven (the first candidate whose guard holds fires: the game case) or, at a choice point
// (a state's `choose` list), picked by a Decider the step is given: code, a replayed log, a search bot, a person, or a
// model whose answer was taken outside the step. Every transition can be reported as one row (TransitionRecord: step,
// machine, from, to, edge, decider, evidence) to a sink the step is given, so one trace records all of them.
//
// The machine's runtime state is a Snapshot, plain JSON kept inside the context it drives (`player.sm`): a fold stays
// pure, checkpoints and renders resume from any step, and a playtest reads every machine's state from the game state.
// Nothing here reads a clock or a random source: the same inputs and decisions give the same states step by step.

// ---- the definition ------------------------------------------------------------------------------------------------

/** A trigger: polled each step while its state is active (the context, and whatever the step passes as env). */
export type Guard<C, E> = (ctx: C, env: E) => boolean;

/** An action: a pure function returning the new context (never a changed old one). */
export type Action<C, E> = (ctx: C, env: E) => C;

export type Actions<C, E> = Action<C, E> | readonly Action<C, E>[];

type Target<N extends string> = { to: NoInfer<N>; target?: undefined } | { target: NoInfer<N>; to?: undefined };

type When<C, E> = { when?: Guard<C, E>; guard?: undefined } | { guard?: Guard<C, E>; when?: undefined };

/** A transition: where to go (`to`, or XState's `target`), when (`when`, or XState's `guard`; none: always), after how
 *  many steps in the state at least (`after`), and what to do on the way (`actions`). */
export type TransitionConfig<C, E, N extends string> = Target<N> & When<C, E> & {
  /** steps in the state before this transition may fire (0) */
  after?: number;
  actions?: Actions<C, E>;
  /** what the transition means: a decider reads it (a model's option description), reports show it */
  description?: string;
};

/** A target's name alone, one transition, or several checked in order. */
export type Transitions<C, E, N extends string> =
  | NoInfer<N>
  | TransitionConfig<C, E, N>
  | readonly (NoInfer<N> | TransitionConfig<C, E, N>)[];

export interface StateConfig<C, E, N extends string> {
  /** run when the state is entered (outer states before inner ones) */
  entry?: Actions<C, E>;
  /** run when the state is left (inner states before outer ones) */
  exit?: Actions<C, E>;
  /** run every step the state is active, after this step's transition (outer before inner, the entry step too) */
  tick?: Actions<C, E>;
  /** triggers polled every step, in order: the first whose guard holds fires (XState's eventless `always`) */
  always?: Transitions<C, E, N>;
  /** delays in steps: `after: { 30: "fall" }` fires once the state has been active 30 steps (and its guard holds) */
  after?: { readonly [steps: number]: Transitions<C, E, N> };
  /** named events the step is given this time (input edges: games.ts keyEdges gives "keydown:ArrowUp") */
  on?: { readonly [event: string]: Transitions<C, E, N> };
  /** a choice point: of these candidates, those whose guards hold go to the step's Decider, which picks one or waits
   *  (no decider: firstDecider takes the first). Polled after `always` and `after`. */
  choose?: Transitions<C, E, N>;
  /** a compound state's first substate */
  initial?: NoInfer<N>;
  /** substates: names are unique across the whole machine, so `to: "run"` names one state wherever it is */
  states?: { [K in N]?: StateConfig<C, E, N> };
  /** re-entering this compound state resumes the substate it left ("shallow"), or that substate's own (deep) */
  history?: "shallow" | "deep";
  /** a compound state's transition taken when its active substate is final */
  onDone?: Transitions<C, E, N>;
  /** a final state: no transitions out (a dead enemy, the game won, the work shipped) */
  type?: "final";
  description?: string;
}

export interface MachineConfig<C, E, N extends string, K extends string> {
  /** the machine's name in traces and reports */
  id: string;
  initial: NoInfer<N>;
  states: { [S in N]?: StateConfig<C, E, N> };
  /** the context's field holding the snapshot ("sm") */
  key?: K;
  /** refuse unreachable states and non-final dead ends when the machine is built (true) */
  strict?: boolean;
}

// ---- the snapshot: the machine's runtime state, plain JSON ----------------------------------------------------------

/** The machine's runtime state, kept in the context it drives. `path` is the active states outermost first ([] before
 *  the first step); `at[i]` is the step on which path[i] was entered; `n` counts the machine's steps from 0; `e` is the
 *  edge the last transition took (an index into machine.edges); `by` names the decider that chose it (absent: a
 *  guard); `hist` is each history state's last substate. */
export interface Snapshot<N extends string = string> {
  path: N[];
  at: number[];
  n: number;
  e: number;
  by?: string;
  hist?: { [S in N]?: N };
}

/** A machine's state in some context, for a trace (`Playtest.machines`). */
export interface Watched {
  machine: AnyMachine;
  snapshot: Snapshot;
}

/** The active leaf of a snapshot (undefined before the first step). */
export function stateOf<N extends string>(snap: Snapshot<N>): N | undefined {
  return snap.path[snap.path.length - 1];
}

/** Whether `name` is active in the snapshot (the leaf or any state around it), as XState's `state.matches`. */
export function inState<N extends string>(snap: Snapshot<N>, name: N): boolean {
  return snap.path.indexOf(name) >= 0;
}

// ---- deciders and the trace ----------------------------------------------------------------------------------------

/** A choice point as a decider sees it: which machine, where, and the candidates whose guards hold. */
export interface ChoicePoint<C = unknown, E = unknown> {
  machine: string;
  /** the state whose `choose` list this is, and the active leaf */
  state: string;
  leaf: string;
  /** the machine's step */
  n: number;
  /** the candidates whose guards hold, in their order (each an edge of machine.edges) */
  candidates: { edge: number; to: string; description?: string }[];
  ctx: C;
  env: E;
}

/** What a decider answers: the edge it picks (one of the candidates' `edge`), or null to wait this step; `evidence` is
 *  whatever supports the choice (a bot's score, a model's probabilities), recorded beside the transition. */
export interface Decision {
  edge: number | null;
  evidence?: unknown;
}

/** Picks a transition at a choice point. Called inside the step, so it must answer synchronously: a model or a person
 *  is asked outside the step and their answer replayed (`replayDecider`). Deterministic deciders keep a fold pure. */
export interface Decider {
  name: string;
  decide(point: ChoicePoint): Decision;
}

/** The default decider: the first candidate. */
export const firstDecider: Decider = { name: "first", decide: (p) => ({ edge: p.candidates[0].edge }) };

/** A decider that replays recorded decisions: `log[machine][n]` is the edge chosen at that machine's step n (missing:
 *  wait). How a model's or a person's choices, taken outside the step, re-enter it exactly. */
export function replayDecider(log: Record<string, Record<number, number | { edge: number; evidence?: unknown }>>, name = "replay"): Decider {
  return {
    name,
    decide: (p) => {
      const got = log[p.machine]?.[p.n];
      if (got === undefined) return { edge: null };
      return typeof got === "number" ? { edge: got } : got;
    },
  };
}

/** One transition as a row: what a trace keeps and rdb loads (step, machine, from, to, edge, decider, evidence). `from` is
 *  null for the first step's entry into the initial states. */
export interface TransitionRecord {
  machine: string;
  step: number;
  from: string | null;
  to: string;
  edge: number;
  kind: Edge["kind"] | "start";
  /** "guard" for a trigger, "start" for the first entry, else the decider's name */
  by: string;
  evidence?: unknown;
}

/** What a step may be given besides the context and env. */
export interface StepOptions {
  /** named events this step (an `on` edge waits for one) */
  events?: readonly string[];
  /** picks at choice points (default firstDecider) */
  decide?: Decider;
  /** receives every transition this step takes, as a row */
  trace?: (r: TransitionRecord) => void;
}
