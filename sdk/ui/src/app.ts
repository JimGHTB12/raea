// An app's target and window (docs/targets.feature T0a). An app file may export one, as a literal:
//
//   import type { AppConfig } from "@raea/ui/app";
//   export const app: AppConfig = { target: "native", title: "Tally", size: [1080, 1920] };
//
// raea reads it from the file before bundling, without running it (apps/targets), so it must be a literal: objects,
// arrays, strings, numbers. A variable, a call or a spread there is refused with its file and line. The default is
// written down: with no `app` export, or no `target` in it, an app is "web". `--target` on the command line wins over
// the file. Nothing here crosses the wire; it's types only.

/** Where an app runs: "web" (Blitz on Bevy, raea's page, the default) or "native" (Rune: native windows, springs). */
export type AppTarget = "web" | "native";

/** An app's `export const app`: its target and its window. */
export type AppConfig = {
  /** the host: "web" when left out */
  target?: AppTarget;
  /** the window's title (the file's name when left out) */
  title?: string;
  /** the window's size in points, [width, height] */
  size?: [number, number];
  /** the smallest the window may be made, [width, height] */
  minSize?: [number, number];
};
