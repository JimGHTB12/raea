// Plugins in TypeScript as Bevy has them (docs/sdk.feature S6c): definePlugin, PluginGroup, BevyApp.open with Rust and
// TypeScript plugins together, addPlugins, addSystems (Startup once before the first frame), insertResource and
// addObserver. The engine's side is stubbed: a fake __raea_call records what the App asks for and answers as
// bevy_remote would, so this runs under plain bun. The engine itself is checked by bench/bevy-ts/programs.
import { afterAll, beforeEach, describe, expect, test } from "bun:test";
import { BevyApp, PluginGroup, Startup, Update, definePlugin, resource } from "raea/bevy";

const SCHEDULES = ["First", "PreUpdate", "StateTransition", "RunFixedMainLoop", "Update", "SpawnScene", "PostUpdate", "Last"];
let calls: { name: string; args: any }[] = [];
let frame = 0;
const g = globalThis as { __raea_call?: (name: string, json: string) => string | undefined };
g.__raea_call = (name, json) => {
  const args = JSON.parse(json);
  calls.push({ name, args });
  switch (name) {
    case "bevy.app.open":
      frame = 0;
      return JSON.stringify({ app: 1, methods: ["world.query"] });
    case "bevy.app.schedules":
      return JSON.stringify(SCHEDULES);
    case "bevy.app.run":
      if (args.until === undefined) frame++;
      return JSON.stringify({ frame });
    case "bevy.app.step":
      frame += args.frames;
      return JSON.stringify([frame]);
    default:
      return JSON.stringify(null);
  }
};
afterAll(() => void delete g.__raea_call);
beforeEach(() => void (calls = []));

const Score = resource<{ points: number }>("game::Score");

describe("plugins", () => {
  test("open names the Rust plugins to the engine and builds the TypeScript ones after, in order", () => {
    const order: string[] = [];
    const hud = definePlugin({ name: "hud", build: () => void order.push("hud.build"), finish: () => void order.push("hud.finish"), cleanup: () => void order.push("hud.cleanup") });
    const score = definePlugin({ name: "score", build: (app) => void (order.push("score.build"), app.insertResource(Score, { points: 0 })), finish: () => void order.push("score.finish") });
    const app = BevyApp.open({ plugins: ["avian", hud, score], fps: 60 });
    expect(calls[0]).toEqual({ name: "bevy.app.open", args: { plugins: ["avian"], fps: 60 } });
    expect(order).toEqual(["hud.build", "score.build", "hud.finish", "score.finish", "hud.cleanup"]);
    expect(calls.some((c) => c.name === "bevy.app.call" && c.args.method === "world.insert_resources" && c.args.params.resource === "game::Score")).toBe(true);
    expect(app.isPluginAdded("avian")).toBe(true);
    expect(app.isPluginAdded("hud")).toBe(true);
    expect(app.isPluginAdded("lightyear")).toBe(false);
  });

  test("a generated handle object names its Rust plugin, as its name would", () => {
    const avian = { plugin: "avian" as const, RigidBody: { kind: "component", path: "avian3d::RigidBody" } };
    const app = BevyApp.open({ plugins: [avian, new PluginGroup("G", [{ plugin: "tnua" }])] });
    expect(calls[0].args.plugins).toEqual(["avian", "tnua"]);
    expect(app.isPluginAdded("tnua")).toBe(true);
  });

  test("raea's own plugins' handles (S6b: @raea/render/plugin, @raea/audio/plugin, @raea/ui/plugin, @raea/character/plugin) are refused by a headless App", async () => {
    const { audio } = await import("@raea/audio/plugin");
    const { render } = await import("@raea/render/plugin");
    const { page } = await import("@raea/ui/plugin");
    const { character, CharacterGoal } = await import("@raea/character/plugin");
    expect(audio).toEqual({ plugin: "audio", raea: true });
    expect(render.plugin).toBe("render");
    expect(page).toEqual({ plugin: "page", raea: true });
    // character's handle carries its components, generated from the registry; CharacterPlugin uses those
    expect(character.CharacterGoal.path).toBe("raea_runtime_plugins::character::CharacterGoal");
    expect(CharacterGoal).toBe(character.CharacterGoal);
    for (const p of [audio, page, character]) expect(() => BevyApp.open({ plugins: [p] })).toThrow(/raea's own plugin/);
    expect(calls).toEqual([]);
  });

  test("a unique plugin added twice throws; a Rust plugin can't be added after the App opens", () => {
    const hud = definePlugin({ name: "hud", build() {} });
    const app = BevyApp.open({ plugins: [hud] });
    expect(() => app.addPlugins(hud)).toThrow(/already added/);
    expect(() => app.addPlugins("avian")).toThrow(/name it when the App opens/);
    const many = definePlugin({ name: "many", isUnique: false, build() {} });
    app.addPlugins(many, many);
  });

  test("a group adds its members in order, with set and disable as Bevy's", () => {
    const seen: string[] = [];
    const p = (name: string) => definePlugin({ name, build: () => void seen.push(name) });
    const group = new PluginGroup("GamePlugins", ["avian", p("input"), p("hud"), p("audio")]);
    group.disable("audio").set(definePlugin({ name: "hud", build: () => void seen.push("hud2") }));
    const app = BevyApp.open({ plugins: [group] });
    expect(calls[0].args.plugins).toEqual(["avian"]);
    expect(seen).toEqual(["input", "hud2"]);
    expect(app.isPluginAdded("audio")).toBe(false);
    expect(() => group.disable("nope")).toThrow(/no plugin "nope"/);
  });
});

describe("systems", () => {
  test("Startup systems run once before the first frame; Update ones every frame, in the order added", () => {
    const ran: string[] = [];
    const game = definePlugin({
      name: "game",
      build(app) {
        app.addSystems(Startup, () => void ran.push("startup"));
        app.addSystems(Update, [(_, f) => void ran.push(`a${f}`), (_, f) => void ran.push(`b${f}`)]);
      },
    });
    const app = BevyApp.open({ plugins: [game] });
    expect(ran).toEqual([]);
    app.step(2);
    expect(ran).toEqual(["startup", "a0", "b0", "a1", "b1"]);
  });

  test("addSystems chains, and an unknown schedule is refused by name", () => {
    const app = BevyApp.open({ plugins: [] });
    expect(app.addSystems(Update, () => {})).toBe(app);
    expect(() => app.addSystems("Updat", () => {})).toThrow(/no schedule "Updat"/);
  });

  test("addObserver is observer: raea/observe inside the App", () => {
    const app = BevyApp.open({ plugins: [] });
    app.addObserver("game::Scored");
    expect(calls.some((c) => c.args.method === "raea/observe" && c.args.params.event === "game::Scored")).toBe(true);
  });
});
