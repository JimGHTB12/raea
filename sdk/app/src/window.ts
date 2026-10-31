// raea/window: a live game's window (docs/bevy-ts.feature G19), as Bevy's Window component would set it:
// its title, size, fullscreen, resizable, decorations and the cursor (visible, hidden, locked to the window for mouse
// look, or confined to it). The host that owns the window applies them (raea play's winit window; `play --json`
// reports them); in a render there's no window and they're only recorded.
//
//   gameWindow.set({ title: "Dungeon", cursor: "locked" });
import { native } from "@raea/app/native";

type Call = (name: string, args: unknown) => unknown;
const call: Call = (name, args) => (native as unknown as Call)(name, args);

export interface WindowSettings {
  title?: string;
  /** physical pixels */
  size?: [number, number];
  fullscreen?: boolean;
  resizable?: boolean;
  decorations?: boolean;
  /** "locked": hidden and held in place (mouse look reads the motion); "confined": kept inside the window */
  cursor?: "visible" | "hidden" | "locked" | "confined";
}

export const gameWindow = {
  /** Changes the settings given (the rest stay as set before). Returns them all. */
  set(s: WindowSettings): WindowSettings {
    return call("raea.window.set", s) as WindowSettings;
  },
  /** The settings set so far. */
  get(): WindowSettings {
    return (call("raea.window.get", {}) as WindowSettings | null) ?? {};
  },
};
