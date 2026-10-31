// S2b's hooks on the World's shared state (docs/blitz-bevy.feature B3 H4): useResource and useComponent read the
// World's values as the JS host's prelude hands them over (`__raea_world`), and setResource / setComponent write to the
// World through the native call `world.write`, keeping no copy of their own. sdk/react's shared.rs is the World side
// (its test: react_and_the_world_share_one_value).
import { afterEach, describe, expect, test } from "bun:test";
import { renderToString } from "react-dom/server";
import { entity, has, observe, read, Resource, without, write, setAppState, setComponent, setResource, subscribe, useAppState, useComponent, useQuery, useResource, useMessages, writeMessage, system, InState, OnEnter, useTime, useAsset, HostWorld } from "raea/ecs";
import { Clock } from "@raea/app/clock";
// the same handles App.setResource and BRP take: a hook reads what bevy.ts's handles name
import { asset, component, event, message, resource, state } from "@raea/app/bevy";

const Score = resource<{ points: number }>("game::Score");
const Health = component<{ hp: number }>("game::Health");
const Mode = state<"Menu" | "Play">("game::Mode");
const Ping = event<{ n: number }>("game::Ping");
const g = globalThis as { __raea_world?: (p: unknown[]) => void; __raea_call?: (n: string, j: string) => string | undefined };

function Hud({ hero }: { hero: number }) {
  const score = useResource(Score);
  const health = useComponent(hero, Health);
  return <p>{score ? score.points : "-"}/{health ? health.hp : "-"}</p>;
}
const text = (hero: number) => renderToString(<Hud hero={hero} />).replace(/<!-- -->/g, "");

afterEach(() => {
  delete g.__raea_call;
  g.__raea_world?.([{ path: Score.path, value: null }, { path: Health.path, entity: 7, value: null }]);
});

describe("hooks", () => {
  test("the World's pushes are what the hooks read, by path and entity", () => {
    expect(text(7)).toBe("<p>-/-</p>");
    g.__raea_world!([{ path: Score.path, value: { points: 3 } }, { path: Health.path, entity: 7, value: { hp: 90 } }]);
    expect(text(7)).toBe("<p>3/90</p>");
    // another entity's component is its own
    expect(text(8)).toBe("<p>3/-</p>");
    // gone: null
    g.__raea_world!([{ path: Health.path, entity: 7, value: null }]);
    expect(text(7)).toBe("<p>3/-</p>");
  });

  test("a write goes to the World, and the value is the World's once it comes back", () => {
    const calls: [string, unknown][] = [];
    g.__raea_call = (name, json) => {
      calls.push([name, JSON.parse(json)]);
      return undefined;
    };
    setResource(Score, { points: 10 });
    setComponent(7, Health, { hp: 1 });
    expect(calls).toEqual([
      ["world.write", { path: "game::Score", value: { points: 10 } }],
      ["world.write", { path: "game::Health", entity: 7, value: { hp: 1 } }],
    ]);
    // no second copy: unchanged until the World sends it
    expect(text(7)).toBe("<p>-/-</p>");
    g.__raea_world!([{ path: Score.path, value: { points: 10 } }]);
    expect(text(7)).toBe("<p>10/-</p>");
  });

  test("a value's first reader starts the World's watch of it, its last stops it", () => {
    const calls: [string, unknown][] = [];
    g.__raea_call = (name, json) => {
      calls.push([name, JSON.parse(json)]);
      return undefined;
    };
    let seen = 0;
    const a = subscribe(Score, () => seen++);
    const b = subscribe(Score, () => seen++);
    const c = subscribe(Health, () => seen++, 7);
    g.__raea_world!([{ path: Score.path, value: { points: 2 } }]);
    expect(seen).toBe(2);
    a();
    b();
    c();
    expect(calls).toEqual([
      ["world.watch", { path: "game::Score" }],
      ["world.watch", { path: "game::Health", entity: 7 }],
      ["world.unwatch", { path: "game::Score" }],
      ["world.unwatch", { path: "game::Health", entity: 7 }],
    ]);
  });

  test("useQuery reads the World's watched world.query, a row per entity in the order asked", () => {
    const Pos = component<{ x: number }>("game::Pos");
    function List() {
      const rows = useQuery([Health, Pos] as const, { without: [component("game::Dead")] });
      return <p>{rows.map(([e, h, p]) => `${e}:${h.hp}@${p.x}`).join(",")}</p>;
    }
    const html = () => renderToString(<List />).replace(/<!-- -->/g, "");
    expect(html()).toBe("<p></p>");
    const params = { data: { components: ["game::Health", "game::Pos"] }, filter: { with: [], without: ["game::Dead"] } };
    g.__raea_world!([{ key: JSON.stringify(["world.query", params]), value: [{ entity: 3, components: { "game::Health": { hp: 5 }, "game::Pos": { x: 1 } } }] }]);
    expect(html()).toBe("<p>3:5@1</p>");
  });

  test("useQuery takes /ecs's terms; a write binding's field sets send the whole value", () => {
    const calls: [string, unknown][] = [];
    g.__raea_call = (name, json) => {
      calls.push([name, JSON.parse(json)]);
      return undefined;
    };
    const Pos = component<{ x: number; y: number }>("game::Pos");
    let hit: { hp: number } | undefined;
    function List() {
      const rows = useQuery(entity, write(Health), read(Pos), has(component("game::Player")), without(component("game::Dead")));
      hit = rows[0]?.[1];
      return <p>{rows.map(([e, h, p]) => `${e}:${h.hp}@${p.x}`).join(",")}</p>;
    }
    const html = () => renderToString(<List />).replace(/<!-- -->/g, "");
    const params = { data: { components: ["game::Health", "game::Pos"] }, filter: { with: ["game::Player"], without: ["game::Dead"] } };
    g.__raea_world!([{ key: JSON.stringify(["world.query", params]), value: [{ entity: 4, components: { "game::Health": { hp: 7 }, "game::Pos": { x: 2, y: 0 } } }] }]);
    expect(html()).toBe("<p>4:7@2</p>");
    hit!.hp = 6;
    expect(calls).toContainEqual(["world.write", { path: "game::Health", entity: 4, value: { hp: 6 } }]);
  });

  test("useMessages reads this frame's from raea/messages.read; writeMessage queues raea/messages.write", () => {
    const calls: [string, unknown][] = [];
    g.__raea_call = (name, json) => {
      calls.push([name, JSON.parse(json)]);
      return undefined;
    };
    const Hit = message<{ dmg: number }>("game::Hit");
    function Log() {
      return <p>{useMessages(Hit).map((h) => h.dmg).join(",")}</p>;
    }
    const key = JSON.stringify(["raea/messages.read", { message: "game::Hit" }]);
    expect(renderToString(<Log />)).toBe("<p></p>");
    g.__raea_world!([{ key, value: [{ dmg: 3 }, { dmg: 4 }] }]);
    expect(renderToString(<Log />)).toBe("<p>3,4</p>");
    writeMessage(Hit, { dmg: 9 });
    expect(calls).toContainEqual(["bevy.world.send", { method: "raea/messages.write", params: { message: "game::Hit", value: { dmg: 9 } } }]);
  });

  test("a system gets each frame's tick and rows, writes back through its write bindings, and stops", () => {
    const calls: [string, { key?: string; query?: unknown }][] = [];
    g.__raea_call = (name, json) => {
      calls.push([name, JSON.parse(json)]);
      return undefined;
    };
    const Pos = component<{ x: number }>("game::Pos");
    const Vel = component<{ x: number }>("game::Vel");
    const frames: number[] = [];
    const stop = system("Update", [entity, write(Pos), read(Vel)], (rows, frame) => {
      frames.push(frame);
      for (const [, p, v] of rows) p.x += v.x;
    });
    const [[name, ask]] = calls;
    expect(name).toBe("world.system");
    expect(ask.query).toEqual({ data: { components: ["game::Pos", "game::Vel"] }, filter: { with: [], without: [] } });
    const rows = [{ entity: 2, components: { "game::Pos": { x: 1 }, "game::Vel": { x: 2 } } }];
    g.__raea_world!([{ key: ask.key!, tick: 10, value: rows }]);
    g.__raea_world!([{ key: ask.key!, tick: 11, value: rows }]);
    expect(frames).toEqual([10, 11]);
    expect(calls.filter(([n]) => n === "world.write").length).toBe(2);
    expect(calls[1]).toEqual(["world.write", { path: "game::Pos", entity: 2, value: { x: 3 } } as never]);
    stop();
    expect(calls.at(-1)).toEqual(["world.unsystem", { key: ask.key }]);
    g.__raea_world!([{ key: ask.key!, tick: 12, value: rows }]);
    expect(frames).toEqual([10, 11]);
  });

  test("<Resource value> sets the resource as setResource does", () => {
    const calls: [string, unknown][] = [];
    g.__raea_call = (name, json) => {
      calls.push([name, JSON.parse(json)]);
      return undefined;
    };
    // an effect: server rendering doesn't run it, so the element is the call, made as React commits it
    expect(renderToString(<Resource resource={Score} value={{ points: 2 }} />)).toBe("");
    expect(calls).toEqual([]);
  });

  test("useAppState reads the World's watched raea/state.get; setAppState asks for the next state", () => {
    const calls: [string, unknown][] = [];
    g.__raea_call = (name, json) => {
      calls.push([name, JSON.parse(json)]);
      return undefined;
    };
    function Title() {
      return <p>{useAppState(Mode) ?? "-"}</p>;
    }
    const key = JSON.stringify(["raea/state.get", { state: "game::Mode" }]);
    expect(renderToString(<Title />)).toBe("<p>-</p>");
    g.__raea_world!([{ key, value: "Menu" }]);
    expect(renderToString(<Title />)).toBe("<p>Menu</p>");
    setAppState(Mode, "Play");
    expect(calls).toContainEqual(["bevy.world.send", { method: "raea/state.set", params: { state: "game::Mode", value: "Play" } }]);
    // the World's, until its StateTransition sends the new one
    expect(renderToString(<Title />)).toBe("<p>Menu</p>");
    g.__raea_world!([{ key, value: "Play" }]);
    expect(renderToString(<Title />)).toBe("<p>Play</p>");
  });

  test("<InState> and <OnEnter> hold their children only in the state; useTime is the composition's clock", () => {
    g.__raea_call = () => undefined;
    const Stage = state<"Menu" | "Play">("game::Stage");
    const key = JSON.stringify(["raea/state.get", { state: "game::Stage" }]);
    const tree = () =>
      renderToString(
        <p>
          <InState state={Stage} is="Play"><b>hud</b></InState>
          <OnEnter state={Stage} is="Menu"><i>menu</i></OnEnter>
        </p>,
      );
    // nothing until the World has sent the state
    expect(tree()).toBe("<p></p>");
    g.__raea_world!([{ key, value: "Menu" }]);
    expect(tree()).toBe("<p><i>menu</i></p>");
    g.__raea_world!([{ key, value: "Play" }]);
    expect(tree()).toBe("<p><b>hud</b></p>");
    function T() {
      const t = useTime();
      return <>{`${t.frame} ${t.elapsed} ${t.delta}`}</>;
    }
    expect(renderToString(<Clock frame={45} fps={30} width={8} height={8} durationInFrames={60}><T /></Clock>)).toBe("45 1.5 0.03333333333333333");
  });

  test("useAsset loads an asset once through the host (asset.load), before the component renders on", () => {
    const calls: string[] = [];
    g.__raea_call = (name, json) => {
      calls.push(`${name} ${json}`);
      return name === "asset.load" ? JSON.stringify({ path: "/edit/dot.png", bytes: 120, width: 64, height: 32 }) : undefined;
    };
    const dot = asset<"image">("dot.png");
    function Pic() {
      const a = useAsset(dot);
      return <i>{`${a.handle} ${a.width}x${a.height} ${a.bytes}`}</i>;
    }
    expect(renderToString(<Pic />)).toBe("<i>dot.png 64x32 120</i>");
    expect(renderToString(<Pic />)).toBe("<i>dot.png 64x32 120</i>");
    expect(calls).toEqual([`asset.load {"path":"dot.png"}`]);
  });

  test("an observer asks the World for one, gets what it saw, and is gone when stopped", () => {
    const calls: [string, { key?: string }][] = [];
    g.__raea_call = (name, json) => {
      calls.push([name, JSON.parse(json)]);
      return undefined;
    };
    const seen: [number, number | undefined][] = [];
    const stop = observe(Ping, (ev, entity) => seen.push([ev.n, entity]), { entity: 5 });
    const [[name, ask]] = calls;
    expect(name).toBe("world.observe");
    expect(ask).toMatchObject({ event: "game::Ping", entity: 5 });
    g.__raea_world!([{ key: ask.key!, events: [{ entity: 5, event: { n: 1 } }, { entity: 5, event: { n: 2 } }] }]);
    expect(seen).toEqual([[1, 5], [2, 5]]);
    stop();
    expect(calls[1]).toEqual(["world.unobserve", { key: ask.key }]);
    g.__raea_world!([{ key: ask.key!, events: [{ entity: 5, event: { n: 3 } }] }]);
    expect(seen.length).toBe(2);
  });

  test("with no World (plain bun) the store is the copy", () => {
    setResource(Score, { points: 4 });
    expect(text(7)).toBe("<p>4/-</p>");
    setAppState(Mode, "Play");
    expect(renderToString(<p>{(() => { function T() { return <>{useAppState(Mode)}</>; } return <T />; })()}</p>)).toBe("<p>Play</p>");
  });
});

// docs/sdk.feature S2d: /ecs's typed queries on the host's World answer from its last frame (HostWorld)
describe("a typed world on the host's World", () => {
  const Pos = component<{ x: number }>("game::Pos");
  const params = { data: { components: ["game::Health", "game::Pos"] }, filter: { with: [], without: [] } };
  const k = JSON.stringify(["world.query", params]);
  const rows = (w: HostWorld) => [...w.query(entity, write(Health), read(Pos))].map(([e, h, p]) => `${e}:${h.hp}@${p.x}`);

  test("exactly one frame stale: a term set's first query watches it and answers nothing, then the last frame's rows", () => {
    const calls: [string, unknown][] = [];
    g.__raea_call = (name, json) => (calls.push([name, JSON.parse(json)]), undefined);
    const w = new HostWorld();
    expect(rows(w)).toEqual([]);
    expect(calls).toEqual([["world.watch", { key: k, method: "world.query", params }]]);
    // the World's end of frame: its rows, pushed
    g.__raea_world!([{ key: k, value: [{ entity: 4, components: { "game::Health": { hp: 7 }, "game::Pos": { x: 2 } } }] }]);
    expect(rows(w)).toEqual(["4:7@2"]);
    // asked again: watched once
    expect(calls.filter(([n]) => n === "world.watch").length).toBe(1);
    w.dispose();
  });

  test("no read-your-writes within a frame: a write goes to the World, and the query shows it once the World does", () => {
    const calls: [string, unknown][] = [];
    g.__raea_call = (name, json) => (calls.push([name, JSON.parse(json)]), undefined);
    const w = new HostWorld();
    rows(w);
    g.__raea_world!([{ key: k, value: [{ entity: 4, components: { "game::Health": { hp: 7 }, "game::Pos": { x: 2 } } }] }]);
    for (const [, h] of w.query(entity, write(Health), read(Pos))) h.hp = 6;
    expect(calls).toContainEqual(["world.write", { path: "game::Health", entity: 4, value: { hp: 6 } }]);
    expect(rows(w)).toEqual(["4:7@2"]);
    g.__raea_world!([{ key: k, value: [{ entity: 4, components: { "game::Health": { hp: 6 }, "game::Pos": { x: 2 } } }] }]);
    expect(rows(w)).toEqual(["4:6@2"]);
    w.dispose();
  });

  test("a term set's watch stops with the last world using it", () => {
    const calls: [string, unknown][] = [];
    g.__raea_call = (name, json) => (calls.push([name, JSON.parse(json)]), undefined);
    const a = new HostWorld(), b = new HostWorld();
    rows(a);
    rows(b);
    expect(a.watching).toBe(1);
    a.dispose();
    expect(calls.some(([n]) => n === "world.unwatch")).toBe(false);
    b.dispose();
    expect(calls).toContainEqual(["world.unwatch", { key: k }]);
    expect(b.watching).toBe(0);
  });
});
