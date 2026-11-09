---
title: "raea/state"
summary: "State machines stepped by frame, for game logic and workflows, typed so a wrong transition doesn't compile."
order: 25
sources: ["sdk/raea/src/state.ts"]
---

A game's logic in raea is a pure step: given the last state and this frame's input, return the next state. That
makes a play-through repeatable, so the same input renders the same frames. `machine` defines a state machine whose
states and transitions are checked by TypeScript: a transition to a state that doesn't exist fails to compile.
`useGameState` and `useGameEvents` run a step function inside a composition.

```ts
import { machine } from "raea/state";

type Player = { y: number; vy: number };
type Env = { jump: boolean };

const player = machine<Player, Env>()({
  id: "player",
  initial: "standing",
  states: {
    standing: { always: { to: "jumping", when: (_c, e) => e.jump } },
    jumping: { always: { to: "standing", when: (c) => c.y <= 0 } },
  },
});
```

See [Your first game](../../start/first-game.md) for a whole game built on a step function.

<!-- generated:begin by tools/manual-ref/sdk.ts from the SDK's exports and doc comments; edit outside this block -->

```ts
import { firstDecider, inState, machine, replayDecider } from "raea/state"
```

`raea/state` exports 2 hooks, 6 functions, 1 value, 20 types. Bevy's counterpart: `bevy_state`. Source: [sdk/raea/src/state.ts](https://github.com/shinyobjectz/raea/blob/main/sdk/raea/src/state.ts).

## Hooks

### `useGameEvents`

The frames, in order, at which `isEvent(before, after)` holds: `after` is the state `step` makes at frame f from
`before`, the state at f - 1. It runs the same (kept) fold as useGameState through the composition's last frame,
counted from the nearest enclosing Sequence, so sound can be cued to what happens in the game. While a person plays
(live input) it runs only through this frame: what comes next hasn't been played yet. The list is read-only (frozen):
the same one is handed out while nothing new happened, so copy it before changing it.

```ts
function useGameEvents<S>(initial: S, step: Step<S>, isEvent: (before: S, after: S) => boolean): number[]
```

### `useGameState`

The game's state at this frame: `initial` at frame 0, then `step(state, input, g)` applied for each earlier frame g
in order. Frames count from the nearest enclosing Sequence, so a game starts when its Sequence does.

```ts
function useGameState<S>(initial: S, step: Step<S>): S
```

## Functions

### `inState`

Whether `name` is active in the snapshot (the leaf or any state around it), as XState's `state.matches`.

```ts
function inState<N extends string>(snap: Snapshot<N>, name: N): boolean
```

### `machine`

A machine for contexts of type C, stepped with an env of type E (a game's frame input and world, a workflow's facts).
Curried so the state names are inferred from the definition while C and E are given:
`machine<Player, Env>()({ id, initial, states })`. A transition to a state that doesn't exist, or a guard reading a
field C lacks, fails to compile; what types can't hold (an initial that isn't a substate, unreachable states, dead
ends) throws here, when the module loads.

```ts
function machine<C, E = unknown>(): <const N extends string, const K extends string = "sm">(config: MachineConfig<C, E, N, K>) => Machine<C, E, N, K>
```

### `replayDecider`

A decider that replays recorded decisions: `log[machine][n]` is the edge chosen at that machine's step n (missing:
wait). How a model's or a person's choices, taken outside the step, re-enter it exactly.

```ts
function replayDecider(log: Record<string, Record<number, number | { edge: number; evidence?: unknown; }>>, name?: string): Decider
```

### `stateOf`

The active leaf of a snapshot (undefined before the first step).

```ts
function stateOf<N extends string>(snap: Snapshot<N>): N | undefined
```

### `stepAll`

One frame for each entity of a list, each carrying its own machine (one per enemy), in order.

```ts
function stepAll<C, E, N extends string, K extends string>(m: Machine<C, E, N, K>, entities: readonly C[], env: E, opts?: readonly string[] | StepOptions): C[]
```

### `transitionOf`

The transition a step took, read from two snapshots of one machine (null when none fired): how a trace that only
sees states (a playtest over a game's pure fold) recovers the TransitionRecord a sink would have had, minus evidence.

```ts
function transitionOf(m: AnyMachine, before: Snapshot<string> | undefined, after: Snapshot<string>): TransitionRecord | null
```

## Values

### `firstDecider`

The default decider: the first candidate.

## Types

### `Action`

An action: a pure function returning the new context (never a changed old one).

```ts
type Action = (ctx: C, env: E) => C
```

### `Actions`

```ts
type Actions = Action<C, E> | readonly Action<C, E>[]
```

### `Analysis`

| Field | Type | What it is |
| --- | --- | --- |
| `id` (required) | `string` | – |
| `states` (required) | `{ name: N; parent: N \| null; depth: number; kind: "leaf" \| "compound" \| "final"; initia…` | – |
| `edges` (required) | `Edge<N>[]` | – |
| `reachable` (required) | `N[]` | states some sequence of transitions reaches from the initial state (guards assumed satisfiable) |
| `unreachable` (required) | `N[]` | – |
| `deadEnds` (required) | `N[]` | non-final leaves with no transition out (their own or an enclosing state's): once entered, never left |

### `AnyMachine`

```ts
type AnyMachine = Machine<any, any, string, string>
```

### `ChoicePoint`

A choice point as a decider sees it: which machine, where, and the candidates whose guards hold.

| Field | Type | What it is |
| --- | --- | --- |
| `machine` (required) | `string` | – |
| `state` (required) | `string` | the state whose `choose` list this is, and the active leaf |
| `leaf` (required) | `string` | – |
| `n` (required) | `number` | the machine's step |
| `candidates` (required) | `{ edge: number; to: string; description?: string }[]` | the candidates whose guards hold, in their order (each an edge of machine.edges) |
| `ctx` (required) | `C` | – |
| `env` (required) | `E` | – |

### `Decider`

Picks a transition at a choice point. Called inside the step, so it must answer synchronously: a model or a person
is asked outside the step and their answer replayed (`replayDecider`). Deterministic deciders keep a fold pure.

| Field | Type | What it is |
| --- | --- | --- |
| `name` (required) | `string` | – |
| `decide` (required) | `(point: ChoicePoint<unknown, unknown>) => Decision` | – |

### `Decision`

What a decider answers: the edge it picks (one of the candidates' `edge`), or null to wait this step; `evidence` is
whatever supports the choice (a bot's score, a model's probabilities), recorded beside the transition.

| Field | Type | What it is |
| --- | --- | --- |
| `edge` (required) | `number \| null` | – |
| `evidence` | `unknown` | – |

### `Edge`

| Field | Type | What it is |
| --- | --- | --- |
| `index` (required) | `number` | – |
| `from` (required) | `N` | – |
| `to` (required) | `N` | – |
| `kind` (required) | `"always" \| "after" \| "on" \| "done" \| "choose"` | – |
| `event` | `string` | the event an `on` edge waits for |
| `after` (required) | `number` | steps in `from` before it may fire |
| `guarded` (required) | `boolean` | – |
| `description` | `string` | – |

### `Guard`

A trigger: polled each step while its state is active (the context, and whatever the step passes as env).

```ts
type Guard = (ctx: C, env: E) => boolean
```

### `Machine`

| Field | Type | What it is |
| --- | --- | --- |
| `id` (required) | `string` | – |
| `key` (required) | `K` | – |
| `states` (required) | `readonly N[]` | every state, in definition order (outer before inner) |
| `edges` (required) | `readonly Edge<N>[]` | every transition, in the order they're checked within a state |
| `start` (required) | `() => Snapshot<N>` | a snapshot before the first step: the first step enters the initial states (running their entry actions) |
| `step` (required) | `(ctx: C, env: E, opts?: readonly string[] \| StepOptions \| undefined) => C` | One step: poll the active states' transitions (innermost first, then outward; within a state `on` for this step's events, then `always`, `after`, `onDone`, then the `choose` candidates for the decider), take the first that fires (at most one a step), then run the active states' tick actions. Returns the new context with its snapshot; the old one is unchanged. The third argument is this step's events, or StepOptions. |
| `state` (required) | `(ctx: C) => N` | the active leaf (the initial leaf before the first step) |
| `path` (required) | `(ctx: C) => N[]` | the active states, outermost first |
| `is` (required) | `(ctx: C, name: N) => boolean` | whether `name` is active (the leaf or a state around it) |
| `steps` (required) | `(ctx: C, name?: N \| undefined) => number` | steps since `name` (the leaf by default) was entered; -1 when it isn't active |
| `entered` (required) | `(before: C, after: C, name?: N \| undefined) => boolean` | whether the step from `before` to `after` entered `name` (any state when omitted) |
| `watch` (required) | `(ctx: C) => Watched` | this machine's state in `ctx`, for a trace |
| `analyze` (required) | `() => Analysis<N>` | the static analysis machine checks read |

### `MachineConfig`

| Field | Type | What it is |
| --- | --- | --- |
| `id` (required) | `string` | the machine's name in traces and reports |
| `initial` (required) | `NoInfer<N>` | – |
| `states` (required) | `{ [S in N]?: StateConfig<C, E, N> }` | – |
| `key` | `K` | the context's field holding the snapshot ("sm") |
| `strict` | `boolean` | refuse unreachable states and non-final dead ends when the machine is built (true) |

### `Playtest`

What a playtester needs from a game, exported beside the view as `export const playtest: Playtest<S> = {...}`
(sdk/react/docs/playtest.md). Optional: a game without one is played from pixels and events alone. Everything here is
pure, so a bot steps the game headlessly, restores any state it has seen (states are values) and searches from it;
whatever it plays is an input timeline `raea render --input` turns back into the same frames.

| Field | Type | What it is |
| --- | --- | --- |
| `initial` (required) | `S` | the state at frame 0, as useGameState is given it |
| `step` (required) | `Step<S>` | the fold useGameState is given: the same function, so a bot plays the game the render shows |
| `actions` (required) | `string[][]` | the key chords worth trying ([] is "nothing held"); a bot's moves are these, each held some frames |
| `observe` | `(state: S, frame: number) => O` | what a player could know at this frame, as plain JSON (positions, lives, what's near): a model reads it |
| `progress` | `(state: S, frame: number) => number` | how far toward the goal, 0..1: search follows it, and "stuck" means it stopped rising |
| `outcome` | `(state: S, frame: number) => "won" \| "lost" \| null` | "won" when the goal is reached, "lost" on game over, null while play goes on |
| `events` | `Record<string, (before: S, after: S) => boolean>` | named moments as useGameEvents takes them (a death, a coin): counted, placed and rendered for the judge |
| `machines` | `(state: S) => Record<string, Watched>` | every state machine in the state, by entity name (`{ player: player.watch(s.player), flow: flow.watch(s) }`): the trace records each one's state and transitions, and the machine checks (coverage, softlocks, unreachable states, reached) read them (sdk/react/docs/state-machines.md) |

### `Snapshot`

The machine's runtime state, kept in the context it drives. `path` is the active states outermost first ([] before
the first step); `at[i]` is the step on which path[i] was entered; `n` counts the machine's steps from 0; `e` is the
edge the last transition took (an index into machine.edges); `by` names the decider that chose it (absent: a
guard); `hist` is each history state's last substate.

| Field | Type | What it is |
| --- | --- | --- |
| `path` (required) | `N[]` | – |
| `at` (required) | `number[]` | – |
| `n` (required) | `number` | – |
| `e` (required) | `number` | – |
| `by` | `string` | – |
| `hist` | `{ [S in N]?: N }` | – |

### `StateConfig`

| Field | Type | What it is |
| --- | --- | --- |
| `entry` | `Actions<C, E>` | run when the state is entered (outer states before inner ones) |
| `exit` | `Actions<C, E>` | run when the state is left (inner states before outer ones) |
| `tick` | `Actions<C, E>` | run every step the state is active, after this step's transition (outer before inner, the entry step too) |
| `always` | `Transitions<C, E, N>` | triggers polled every step, in order: the first whose guard holds fires (XState's eventless `always`) |
| `after` | `{ readonly [steps: number]: Transitions<C, E, N> }` | delays in steps: `after: { 30: "fall" }` fires once the state has been active 30 steps (and its guard holds) |
| `on` | `{ readonly [event: string]: Transitions<C, E, N> }` | named events the step is given this time (input edges: games.ts keyEdges gives "keydown:ArrowUp") |
| `choose` | `Transitions<C, E, N>` | a choice point: of these candidates, those whose guards hold go to the step's Decider, which picks one or waits (no decider: firstDecider takes the first). Polled after `always` and `after`. |
| `initial` | `NoInfer<N>` | a compound state's first substate |
| `states` | `{ [K in N]?: StateConfig<C, E, N> }` | substates: names are unique across the whole machine, so `to: "run"` names one state wherever it is |
| `history` | `"shallow" \| "deep"` | re-entering this compound state resumes the substate it left ("shallow"), or that substate's own (deep) |
| `onDone` | `Transitions<C, E, N>` | a compound state's transition taken when its active substate is final |
| `type` | `"final"` | a final state: no transitions out (a dead enemy, the game won, the work shipped) |
| `description` | `string` | – |

### `StateName`

The state names of a machine.

```ts
type StateName = M extends Machine<any, any, infer N, any> ? N : never
```

### `StepOptions`

What a step may be given besides the context and env.

| Field | Type | What it is |
| --- | --- | --- |
| `events` | `readonly string[]` | named events this step (an `on` edge waits for one) |
| `decide` | `Decider` | picks at choice points (default firstDecider) |
| `trace` | `(r: TransitionRecord) => void` | receives every transition this step takes, as a row |

### `TransitionConfig`

A transition: where to go (`to`, or XState's `target`), when (`when`, or XState's `guard`; none: always), after how
many steps in the state at least (`after`), and what to do on the way (`actions`).

```ts
type TransitionConfig = Target<N> & When<C, E> & { /** steps in the state before this transition may fire (0) */ after?: number; actions?: Actions<C, E>; /** what the transition means: a decider reads it (a model's option description), reports show it */ description?: string; }
```

### `TransitionRecord`

One transition as a row: what a trace keeps and rdb loads (step, machine, from, to, edge, decider, evidence). `from` is
null for the first step's entry into the initial states.

| Field | Type | What it is |
| --- | --- | --- |
| `machine` (required) | `string` | – |
| `step` (required) | `number` | – |
| `from` (required) | `string \| null` | – |
| `to` (required) | `string` | – |
| `edge` (required) | `number` | – |
| `kind` (required) | `Edge["kind"] \| "start"` | – |
| `by` (required) | `string` | "guard" for a trigger, "start" for the first entry, else the decider's name |
| `evidence` | `unknown` | – |

### `Transitions`

A target's name alone, one transition, or several checked in order.

```ts
type Transitions = | NoInfer<N> | TransitionConfig<C, E, N> | readonly (NoInfer<N> | TransitionConfig<C, E, N>)[]
```

### `Watched`

A machine's state in some context, for a trace (`Playtest.machines`).

| Field | Type | What it is |
| --- | --- | --- |
| `machine` (required) | `AnyMachine` | – |
| `snapshot` (required) | `Snapshot` | – |

<!-- generated:end -->
