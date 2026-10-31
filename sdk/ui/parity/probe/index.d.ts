// What a case imports from "parity" (the harness's probe; probe/chrome.ts and probe/bevy.ts implement it per side).
/** Which side renders this bundle: "chrome" (React DOM, the oracle) or "bevy" (@raea/ui on bevy_ui). A self-check run
 * (`ui-parity calibrate`) renders both sides in Chrome and names the second "b". Only calibration cases read it. */
export declare const side: "chrome" | "bevy" | "b";
/** One entry of the case's event trace: what happened (an event's name), on which element (its data-testid) and what
 * the handler saw (optional, JSON). Compared entry for entry after each step. */
export declare function trace(ev: string, on: string, data?: unknown): void;
/** A step of the input script (src/case.rs reads them from the bundle's manifest). */
export type Step =
  | { click: string; at?: [number, number]; clicks?: number; button?: "left" | "right" }
  | { move: string | [number, number] }
  | { down: string } | { up: string }
  | { leave: true }
  | { wheel: string; dy: number; dx?: number }
  | { key: string; mods?: ("shift" | "ctrl" | "alt" | "meta")[] }
  | { text: string }
  | { wait: number };
