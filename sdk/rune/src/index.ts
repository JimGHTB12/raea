// @raea/rune: what Rune can do that CSS can't, as typed props on any element (docs/rune.feature R6a). They travel as
// one `data-rune` attribute, so React DOM, Blitz and a browser ignore them and the same TSX renders there unchanged;
// raea-rune-host reads it (engine/rune/style/src/runeprops.rs) and maps each field to Rune's builder.
//
//   <div {...rune({ motion: "pop", transition: { effect: "slide" } })}>...</div>
//
// Everything not marked with motion lands at once in the Rune host, as the DOM does; an element that asks for motion,
// a transition or lead and trail animates as Rune does.

export * from "./kit.tsx";

/** A spring as Rune has it: how long it takes to settle, in seconds, and how much it overshoots (0 none, 1 a lot). */
export type Spring = { duration: number; bounce?: number };

/** A spring by duration and bounce. */
export const Spring = (duration: number, bounce = 0): Spring => ({ duration, bounce });

/** Rune's motion tokens (rune-motion tokens.rs), by what they're for. */
export type SpringPreset =
  /** pointer-tied: takes the edge off a jump */
  | "instant"
  /** small controls answering a click */
  | "snap"
  /** the default state change */
  | "settle"
  /** layout moving out of the way */
  | "reflow"
  /** big surfaces travelling: drawers, sheets */
  | "glide"
  /** one shape becoming another */
  | "morph"
  /** a release with a little life: presses, confirmations */
  | "pop"
  /** slow reveals the eye should follow */
  | "drift";

/** A spring: a preset's name or one of your own. */
export type Motion = SpringPreset | Spring;

/** The edge a transition moves through. */
export type Edge = "leading" | "trailing" | "top" | "bottom";

/** How an element enters when it mounts and leaves when it's removed (Rune keeps it on screen while it leaves). */
export type Effect =
  /** no motion: whole the frame it mounts, gone the frame it's removed */
  | "none"
  | "opacity"
  /** in from the leading edge, out through the trailing edge */
  | "slide"
  /** an in-place replacement: a little blur, scale and fade */
  | "blurReplace"
  | { scale: number }
  | { offset: [number, number] }
  /** in from and out to an edge, by the element's size */
  | { move: Edge }
  /** in from an edge, out through the opposite one, fading */
  | { push: Edge }
  /** a Gaussian blur (sigma in points) and a fade */
  | { blur: number };

/** A transition: its effect, and the springs it enters and exits on (Rune's defaults when left out). */
export type Transition = { effect: Effect; enter?: Motion; exit?: Motion };

/** Rune-only behaviour for one element. */
export type RuneProps = {
  /** the spring this element's layout changes move on (place, size, opacity) */
  motion?: Motion;
  /** its entrance and exit */
  transition?: Transition | Effect;
  /** when it travels, its leading edge moves on `lead` and its trailing edge on `trail` */
  lead?: { lead: Motion; trail: Motion };
  /** every change inside it lands at once, even where a parent animates */
  instant?: boolean;
  /** a Gaussian blur of the element itself, sigma in points */
  blur?: number;
  /** a blur of what's behind the element, sigma in points */
  backdropBlur?: number;
  /** the system's glass material behind the element (Liquid Glass on macOS; a blur elsewhere) */
  glass?: boolean;
};

/** The prop that carries `p`: spread it onto any element. */
export function rune(p: RuneProps): { "data-rune": string } {
  return { "data-rune": JSON.stringify(p) };
}
