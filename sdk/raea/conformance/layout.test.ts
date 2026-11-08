// The SDK's layout (docs/sdk.feature S1, M2): Bevy-named entry points that both bundlers resolve alike, and a prelude
// that holds every everyday name.
import { describe, expect, test } from "bun:test";
import { existsSync } from "node:fs";
import { join } from "node:path";

const ROOT = join(import.meta.dir, "..");
// sdk/tsconfig.base.json's paths: the one list of what raea/<module> means (S1f); package.json exports only "."
const paths = (await Bun.file(join(ROOT, "../tsconfig.base.json")).json()).compilerOptions.paths as Record<string, string[]>;
const fileOf = (sub: string) => Bun.resolveSync(`raea/${sub}`, ROOT);

// Bevy's crates (bevy_ prefix dropped) and raea's own subjects; anything else is an older topic subpath (S7 M5)
const BEVY = ["app", "asset", "time", "state", "input", "transform", "math", "color", "mesh", "image", "camera", "light", "pbr", "post_process", "sprite", "text", "ui", "animation", "audio", "picking", "gizmos", "scene", "remote", "dev_tools", "feathers", "window", "ecs", "prelude"];
const RAEA = ["video", "physics", "character", "room", "ml"];
// modules the guides name whose file isn't sdk/raea/src/<name> (S1f's trim dropped raea/bevy-own for a while)
const NAMED = ["bevy-own", "glb", "views", "modelling", "arena", "window", "screen", "asset"];

describe("entry points", () => {
  test("the list names files that exist, raea/* is sdk/raea/src/*, and package.json exports only the prelude", async () => {
    for (const [key, [file]] of Object.entries(paths)) if (!key.endsWith("*") && key.startsWith("raea")) expect(existsSync(join(ROOT, "..", file))).toBe(true);
    expect(paths["raea/*"]).toEqual(["./raea/src/*"]);
    expect((await Bun.file(join(ROOT, "package.json")).json()).exports).toEqual({ ".": "./src/prelude.ts" });
  });

  test("every Bevy-named module resolves", () => {
    for (const sub of [...BEVY, ...RAEA, ...NAMED]) expect(existsSync(fileOf(sub))).toBe(true);
  });

  test("each module loads, and re-exports the same values as the files it names", async () => {
    const root = await import("../src/index");
    const three = await import("../src/3d");
    const bevy3d = (await import("@raea/render/bevy3d")) as Record<string, unknown>;
    // the re-export modules (window, ecs, scene, physics and raea's own subjects are files of their own: /ml's Model is
    // its model class, not 3D's)
    const reexports = [...BEVY, "video"].filter((s) => !["window", "ecs", "scene", "prelude"].includes(s));
    for (const sub of reexports) {
      const m = await import(fileOf(sub));
      expect(Object.keys(m).length).toBeGreaterThan(0);
      for (const [k, v] of Object.entries(m)) {
        // /light's lights are Bevy's, in Bevy's units (bevy3d.tsx), where /3d's have raea's
        if (sub === "light" && k in bevy3d) expect(v).toBe(bevy3d[k]);
        else if (k in root && typeof v === "function") expect(v).toBe((root as Record<string, unknown>)[k]);
        else if (k in three && typeof v === "function") expect(v).toBe((three as Record<string, unknown>)[k]);
      }
    }
  });
});

describe("prelude", () => {
  test("holds every value of the root and of /3d (Bevy's lights in place of /3d's), one import for 2D and 3D", async () => {
    const prelude = (await import("../src/prelude")) as Record<string, unknown>;
    const root = (await import("../src/index")) as Record<string, unknown>;
    const three = (await import("../src/3d")) as Record<string, unknown>;
    for (const [k, v] of Object.entries(root)) expect(prelude[k]).toBe(v);
    const bevy3d = (await import("@raea/render/bevy3d")) as Record<string, unknown>;
    // the lights both name are Bevy's (lux, lumens, cd/m²); /3d keeps raea's relative units
    const LIGHTS = ["AmbientLight", "DirectionalLight", "PointLight", "SpotLight"];
    for (const [k, v] of Object.entries(three)) expect(prelude[k]).toBe(LIGHTS.includes(k) ? bevy3d[k] : v);
    expect(prelude.Group).toBe(three.Group);
  });
});

describe("the engine's name", () => {
  test('"raea" is the prelude and "raea/<module>" each module (tsconfig paths here; both bundlers alike)', async () => {
    const raea = (await import("raea")) as Record<string, unknown>;
    const prelude = (await import("../src/prelude")) as Record<string, unknown>;
    expect(Object.keys(raea).sort()).toEqual(Object.keys(prelude).sort());
    for (const [k, v] of Object.entries(prelude)) expect(raea[k]).toBe(v);
    for (const sub of BEVY.filter((s) => s !== "prelude")) {
      const a = (await import(`raea/${sub}`)) as Record<string, unknown>;
      const b = (await import(fileOf(sub))) as Record<string, unknown>;
      expect(Object.keys(a).length).toBeGreaterThan(0);
      for (const [k, v] of Object.entries(b)) expect(a[k]).toBe(v);
    }
  });
});
