// What every drive in proof.ts and games.ts uses: checks, stepping until an answer, loading a fixture
import { BevyApp, component } from "raea/bevy";
import { native } from "raea/native";

// a value giving only some fields, for a type whose reflected Default fills the rest (a handle's type is the whole value)
export const part = <T,>(v: Partial<T>) => v as T;

export function check(ok: boolean, what: string) {
  if (!ok) throw new Error(what);
}

export const Transform = component<{ translation?: [number, number, number] }>("bevy_transform::components::transform::Transform");

export type Drive = (app: BevyApp) => Record<string, unknown>;

// steps until got() is non-null: for a plugin that answers off the main thread (IO pool, sockets, the audio thread).
// A lost answer fails at 20000 frames, it never hangs
export function until<T>(app: BevyApp, got: () => T | null | undefined, what: string): T {
  for (let i = 0; i < 20000; i++) {
    app.step(1);
    const r = got();
    if (r !== null && r !== undefined) return r;
  }
  throw new Error(`no ${what} in 20000 frames`);
}

export type AssetState = { state: "Loading" | "Loaded" | "Failed"; type?: string; error?: string; vertices?: number };

// a fixture in catalog/data loaded through raea/asset.load (type, when given, picks the loader) until it's in or fails
export function loaded(app: BevyApp, file: string, type?: string): AssetState {
  const dir = native("host.env", { name: "RAEA_PROOF_DIR" })!;
  const id = app.call<number>("raea/asset.load", { path: `${dir}/${file}`, type });
  const st = until(app, () => {
    const s = app.call<AssetState>("raea/asset.state", { id });
    return s.state === "Loading" ? null : s;
  }, `${file} loaded`);
  check(st.state === "Loaded", `${file}: ${st.error ?? st.state}`);
  return st;
}
