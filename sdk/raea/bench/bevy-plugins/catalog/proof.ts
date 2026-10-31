// One declared plugin, proved headless from TypeScript (sdk/raea/docs/bevy-plugins.md, "The catalog"): run.sh runs
// this once per plugin with RAEA_PROOF=<name>, each in its own process, so one plugin's panic is its own failure.
// Every plugin is opened alone in a headless App (with the plugins it needs) and stepped; the ones with an API reached
// from TypeScript are driven through it and checked. Prints one JSON line: {"proof": name, ...what it measured}.
//   RAEA_PROOF=leafwing raea-bevy-sdk motion run sdk/raea/bench/bevy-plugins/catalog/proof.ts
import { BevyApp, at, component, with as w } from "raea/bevy";
import { native, print } from "raea/native";
import { avian } from "raea/plugins/avian";
import { avian2d } from "raea/plugins/avian2d";
import { tnua } from "raea/plugins/tnua";
import { rapier3d } from "raea/plugins/rapier3d";
import { rapier2d } from "raea/plugins/rapier2d";
import { sprite3d } from "raea/plugins/sprite3d";
import { traumaShake } from "raea/plugins/trauma_shake";
import { bigSpace } from "raea/plugins/big_space";
import { rand } from "raea/plugins/rand";
import { pancam } from "raea/plugins/pancam";
import { tween } from "raea/plugins/tween";
import { egui } from "raea/plugins/egui";
import { bevy } from "raea/plugins/bevy";

import { Drive, Transform, check, loaded, part, until } from "./drive";
import { games } from "./games";

// the plugins a proof's App needs beside the one proved
const with_: Record<string, string[]> = {
  tnua: ["avian", "tnua"],
  avian_pickup: ["avian", "avian_pickup"],
  ahoy: ["avian", "ahoy"],
  fps_controller: ["avian", "fps_controller"],
  inspector: ["egui", "inspector"],
  console: ["egui", "console"],
  ldtk: ["tilemap", "ldtk"],
  bevy: ["bevy", "obj"], // the picking drive's cube is catalog/data/cube.obj
  tiled: ["tilemap", "tiled"],
};

const drives: Record<string, Drive> = {
  leafwing(app) {
    const player = app.call<number>("leafwing/player", { keys: { Jump: "Space", Fire: "KeyF" }, move: "wasd" });
    app.call("raea/input", { key: "Space" });
    app.call("raea/input", { key: "KeyD" });
    app.step(2);
    const s = app.call<{ pressed: string[]; move: [number, number] }>("leafwing/state", { entity: player });
    check(s.pressed.includes("Jump"), `Space should press Jump: ${JSON.stringify(s)}`);
    check(s.move[0] > 0.5, `D should move right: ${JSON.stringify(s)}`);
    app.call("raea/input", { key: "Space", pressed: false });
    app.step(2);
    const after = app.call<{ pressed: string[] }>("leafwing/state", { entity: player });
    check(!after.pressed.includes("Jump"), "Jump should be released");
    return { pressed: s.pressed, move: s.move, released: after.pressed };
  },
  enhanced_input(app) {
    const pawn = app.call<number>("enhanced_input/pawn");
    app.call("raea/input", { key: "Space" });
    app.call("raea/input", { key: "KeyW" });
    app.step(2);
    const s = app.call<Record<string, unknown>>("enhanced_input/state", { entity: pawn });
    check(s.Jump === true, `Space should fire Jump: ${JSON.stringify(s)}`);
    const m = s.Move as [number, number];
    check(m[1] > 0.5, `W should move forward: ${JSON.stringify(s)}`);
    return { state: s };
  },
  tnua(app) {
    app.spawn(w(avian.RigidBody, "Static"), w(avian.ColliderConstructor, { Cuboid: { x_length: 40, y_length: 1, z_length: 40 } }), at(0, -0.5, 0));
    const body = app.spawn(w(avian.RigidBody, "Dynamic"), w(avian.ColliderConstructor, { Capsule: { radius: 0.5, height: 1 } }), at(0, 2, 0));
    app.step(1);
    app.call("tnua/character", { entity: body, float_height: 1.5, speed: 3 });
    app.step(60);
    const rest = app.get(body, avian.Position)!;
    check(Math.abs(rest[1] - 1.5) < 0.2, `it should float at 1.5: ${rest}`);
    app.insert(body, w(tnua.TnuaInput, { motion: [1, 0, 0], jump: false, crouch: false, dash: false }));
    app.step(60);
    const walked = app.get(body, avian.Position)!;
    check(walked[0] > 1, `it should walk +x: ${walked}`);
    app.insert(body, w(tnua.TnuaInput, { motion: [0, 0, 0], jump: true, crouch: false, dash: false }));
    let top = walked[1];
    for (let i = 0; i < 40; i++) {
      app.step(1);
      top = Math.max(top, app.get(body, avian.Position)![1]);
    }
    check(top > rest[1] + 0.5, `jump should lift it: top ${top}`);
    return { rest, walked, jumpTop: top };
  },
  avian2d(app) {
    app.spawn(w(avian2d.RigidBody, "Static"), w(avian2d.ColliderConstructor, { Rectangle: { x_length: 20, y_length: 1 } }), w(Transform, { translation: [0, -0.5, 0] }));
    const ball = app.spawn(w(avian2d.RigidBody, "Dynamic"), w(avian2d.ColliderConstructor, { Circle: { radius: 0.5 } }), w(Transform, { translation: [0, 4, 0] }));
    app.step(120);
    const p = app.get(ball, avian2d.Position)!;
    check(Math.abs(p[1] - 0.5) < 0.05, `the 2D ball should rest on the floor at 0.5: ${p}`);
    return { ball: p };
  },
  rapier3d(app) {
    // a ball dropped on a fixed floor whose top is y = 0 rests with its centre at its radius (ColliderShape builds
    // rapier's Collider, which isn't reflected)
    app.spawn(w(rapier3d.RigidBody, "Fixed"), w(rapier3d.ColliderShape, { Cuboid: { half_size: [20, 0.5, 20] } }), at(0, -0.5, 0));
    const ball = app.spawn(w(rapier3d.RigidBody, "Dynamic"), w(rapier3d.ColliderShape, { Ball: { radius: 0.5 } }), at(0, 4, 0));
    app.step(180);
    const p = app.get(ball, Transform)!.translation!;
    check(Math.abs(p[1] - 0.5) < 0.05, `the ball should rest on the floor at 0.5: ${p}`);
    return { ball: p };
  },
  rapier2d(app) {
    app.spawn(w(rapier2d.RigidBody, "Fixed"), w(rapier2d.ColliderShape, { Cuboid: { half_size: [10, 0.5] } }), at(0, -0.5, 0));
    const ball = app.spawn(w(rapier2d.RigidBody, "Dynamic"), w(rapier2d.ColliderShape, { Ball: { radius: 0.5 } }), at(0, 4, 0));
    app.step(180);
    const p = app.get(ball, Transform)!.translation!;
    check(Math.abs(p[1] - 0.5) < 0.05, `the 2D ball should rest on the floor at 0.5: ${p}`);
    return { ball: p };
  },
  bevy(app) {
    // Bevy's own gamepads: raea/input connects a virtual one and presses through its raw events, which Bevy's gamepad
    // systems turn into the Gamepad a game reads
    type Pad = { buttons: string[]; axes: Record<string, number> };
    const pad = app.call<number>("raea/input", { gamepad: "South" });
    app.call("raea/input", { axis: "LeftStickX", value: 0.5 });
    app.step(2);
    check(app.query({ has: [bevy.Gamepad] }).some((r) => r.entity === pad), "the virtual gamepad has Bevy's Gamepad");
    const held = app.call<Pad>("raea/input.gamepad");
    check(held.buttons.includes("South"), `South should be held: ${JSON.stringify(held)}`);
    check(Math.abs((held.axes.LeftStickX ?? 0) - 0.5) < 1e-6, `LeftStickX should read 0.5: ${JSON.stringify(held)}`);
    app.call("raea/input", { gamepad: "South", pressed: false });
    app.step(2);
    check(!app.call<Pad>("raea/input.gamepad").buttons.includes("South"), "South let go");
    const stick = held.axes;
    // picking: a camera at z = 5 looking down -z at a unit cube (catalog/data/cube.obj) at the origin, in the primary
    // window; the pointer over the window's middle hovers it, a press there presses it, and a corner is off it
    const dir = native("host.env", { name: "RAEA_PROOF_DIR" })!;
    app.spawn(w(bevy.Camera3d, {}), at(0, 0, 5));
    const cube = app.spawn(w(bevy.Mesh3d, `${dir}/cube.obj`), at(0, 0, 0));
    const interaction = () => app.get(cube, bevy.PickingInteraction) ?? "None";
    app.call("raea/input", { pointer: [640, 360] });
    const hovered = until(app, () => (interaction() === "Hovered" ? interaction() : null), "a hover on the cube");
    app.call("raea/input", { pointer: [640, 360], mouse: "Left" });
    app.step(2);
    const pressed = interaction();
    check(pressed === "Pressed", `a press over the cube: ${pressed}`);
    app.call("raea/input", { pointer: [5, 5], mouse: "Left", pressed: false });
    app.step(2);
    const off = interaction();
    check(off === "None", `the pointer off the cube: ${off}`);
    return { gamepad: { pressed: held.buttons, stick }, picking: { hovered, pressed, off } };
  },
  obj(app) {
    const got = loaded(app, "cube.obj", "bevy_mesh::mesh::Mesh");
    check((got.vertices ?? 0) >= 8, `the cube should have at least its 8 corners: ${JSON.stringify(got)}`);
    return { loaded: got };
  },
  vox(app) {
    const got = loaded(app, "voxel.vox");
    // a .vox is a scene: bevy_vox_scene's loader makes a WorldAsset of its models (and their meshes and materials)
    check(got.type === "bevy_world_serialization::world_asset::WorldAsset", `voxel.vox should load as a scene: ${JSON.stringify(got)}`);
    return { loaded: got };
  },
  common_assets(app) {
    const dir = native("host.env", { name: "RAEA_PROOF_DIR" })!;
    const files = ["data.json", "data.toml", "data.yaml", "rows.csv"];
    for (const f of files) app.call("common_assets/load", { path: `${dir}/${f}` });
    const got: Record<string, unknown> = {};
    for (let i = 0; i < 200 && Object.keys(got).length < files.length; i++) {
      app.step(1);
      for (const f of files) if (!(f in got)) {
        const v = app.call("common_assets/get", { path: `${dir}/${f}` });
        if (v !== null) got[f] = v;
      }
    }
    check(Object.keys(got).length === files.length, `loaded ${Object.keys(got).join(", ")}`);
    for (const f of ["data.json", "data.toml", "data.yaml"]) check((got[f] as { speed: number }).speed === 3.5, `${f}: ${JSON.stringify(got[f])}`);
    check((got["rows.csv"] as { name: string }[])[1].name === "goblin", `rows.csv: ${JSON.stringify(got["rows.csv"])}`);
    return { loaded: got };
  },
  noise(app) {
    const p = app.call<number[]>("noise/sample", { kind: "perlin", seed: 7, frequency: 0.1, points: [[0, 0], [12.3, 4.5], [7, 8, 9]] });
    const again = app.call<number[]>("noise/sample", { kind: "perlin", seed: 7, frequency: 0.1, points: [[0, 0], [12.3, 4.5], [7, 8, 9]] });
    check(JSON.stringify(p) === JSON.stringify(again), "a seed should give the same noise");
    const grid = app.call<number[][]>("noise/grid", { kind: "fbm", seed: 1, frequency: 0.05, octaves: 5, origin: [0, 0], size: [64, 32] });
    check(grid.length === 32 && grid[0].length === 64, "grid 64x32");
    const flat = grid.flat();
    const span = Math.max(...flat) - Math.min(...flat);
    check(span > 0.2 && flat.every((v) => v >= -2 && v <= 2), `fbm should vary in range: span ${span}`);
    const fast = app.call<number[]>("noise/sample", { kind: "fast_open_simplex2", seed: 3, frequency: 0.02, points: [[10, 20], [30, 40]] });
    check(fast.every((v) => v >= -1 && v <= 1) && fast[0] !== fast[1], `fastnoise: ${fast}`);
    return { perlin: p, fbmSpan: span, fast };
  },
  noisy(app) {
    const pts = [[0.5, 0.25], [12.3, 4.5], [7, 8, 9]];
    const p = app.call<number[]>("noisy/sample", { kind: "simplex", seed: 2, frequency: 0.1, points: pts });
    const again = app.call<number[]>("noisy/sample", { kind: "simplex", seed: 2, frequency: 0.1, points: pts });
    const other = app.call<number[]>("noisy/sample", { kind: "simplex", seed: 9, frequency: 0.1, points: pts });
    check(JSON.stringify(p) === JSON.stringify(again), "a seed should give the same noise");
    check(JSON.stringify(p) !== JSON.stringify(other), "another seed should give other noise");
    const grid = app.call<number[][]>("noisy/grid", { kind: "fbm", octaves: 5, frequency: 0.05, origin: [0, 0], size: [64, 32] });
    check(grid.length === 32 && grid[0].length === 64, "grid 64x32");
    const flat = grid.flat();
    const span = Math.max(...flat) - Math.min(...flat);
    check(span > 0.2 && flat.every((v) => v >= -2 && v <= 2), `fbm should vary in range: span ${span}`);
    const cells = app.call<number[]>("noisy/sample", { kind: "worley", points: [[0.5, 0.5], [3.2, 1.7]] });
    check(cells.every((v) => v >= 0 && v <= 1.5), `worley distances: ${cells}`);
    return { simplex: p, fbmSpan: span, worley: cells };
  },
  egui(app) {
    // EguiGlobalSettings steers whether cameras get the primary context: with auto_create_primary_context off, a new
    // camera gets none; on again, exactly one camera gets it (bevy_egui's setup reads cameras added since it last ran,
    // so it may be the one added while off). EguiContext brings EguiContextSettings, a reflected component
    const Camera = component<Record<string, never>>("bevy_camera::camera::Camera");
    const settings = app.resource(egui.EguiGlobalSettings);
    app.insertResource(egui.EguiGlobalSettings, { ...settings, auto_create_primary_context: false });
    const off = app.spawn(w(Camera, {}));
    app.step(2);
    const contexts = () => app.query({ has: [egui.EguiContextSettings] }).map((r) => r.entity);
    check(contexts().length === 0, `auto_create_primary_context off: no camera gets an egui context: ${contexts()}`);
    app.insertResource(egui.EguiGlobalSettings, { ...settings, auto_create_primary_context: true });
    const on = app.spawn(w(Camera, {}));
    app.step(2);
    const got = contexts();
    check(got.length === 1 && (got[0] === off || got[0] === on), `on: exactly one camera gets the primary context: ${got}`);
    return { primary: got[0] };
  },
  pancam(app) {
    // the primary window the declaration's "primary-window" need opens (no backend draws it), with the cursor put
    // over it through BRP (pancam's movement reads the cursor first), and an orthographic camera 2 x 2 world units
    // across its 1280 px; min_x is the one bound JSON can say (the rest are infinite: the reflected Default fills them)
    const PrimaryWindow = component<Record<string, never>>("bevy_window::window::PrimaryWindow");
    const Camera = component<Record<string, never>>("bevy_camera::camera::Camera");
    const Projection = component<unknown>("bevy_camera::projection::Projection");
    const ortho = { near: 0, far: 1000, viewport_origin: [0.5, 0.5], scaling_mode: "WindowSize", scale: 1, area: { min: [-1, -1], max: [1, 1] } };
    const win = app.query({ has: [PrimaryWindow] }).map((r) => r.entity)[0];
    check(win !== undefined, "the primary window");
    app.call("world.mutate_components", { entity: win, component: "bevy_window::window::Window", path: ".internal.physical_cursor_position", value: [640, 360] });
    const cam = app.spawn(w(Camera, {}), w(Projection, { Orthographic: ortho }), w(pancam.PanCam, part({ speed: 1e6, min_x: -5 })), at(0, 0, 0));
    app.step(2);
    check(app.get(cam, Transform)!.translation![0] === 0, "no key held: the camera stays");
    app.call("raea/input", { key: "KeyA" });
    // held left, it pans until its left edge (x - 1) meets min_x: x = -4
    const x = until(app, () => { const t = app.get(cam, Transform)!.translation!; return t[0] <= -4 + 1e-3 ? t : null; }, "a pan to the bound");
    app.step(10);
    const held = app.get(cam, Transform)!.translation!;
    check(Math.abs(held[0] + 4) < 1e-3 && held[1] === 0, `held at the bound x = -4: ${held}`);
    app.call("raea/input", { key: "KeyA", pressed: false });
    return { reached: x, held };
  },
  trauma_shake(app) {
    const cam = app.spawn(w(traumaShake.Shake, { trauma: 0, reference_translation: null }), w(traumaShake.ShakeSettings, { amplitude: 2, trauma_power: 2, decay_per_second: 0.8, frequency: 15, octaves: 1 }), at(0, 0, 10));
    app.step(1);
    const t = app.call<number>("trauma_shake/add", { entity: cam, amount: 0.9 });
    check(Math.abs(t - 0.9) < 1e-6, `trauma after adding 0.9: ${t}`);
    let moved = 0;
    for (let i = 0; i < 10; i++) {
      app.step(1);
      const p = app.get(cam, Transform)?.translation ?? [0, 0, 10];
      moved = Math.max(moved, Math.hypot(p[0], p[1], p[2] - 10));
    }
    const left = app.call<number>("trauma_shake/get", { entity: cam });
    check(moved > 0.01, `the camera should shake: moved at most ${moved}`);
    check(left < t && left > 0, `trauma should decay: ${t} -> ${left}`);
    const all = app.call<number[]>("trauma_shake/add", { amount: 5 });
    check(all.length === 1 && all[0] === 1, `trauma clamps at 1 on every Shake: ${all}`);
    return { trauma: t, decayedTo: left, moved };
  },
  http(app) {
    const base = native("host.env", { name: "RAEA_PROOF_HTTP" });
    check(!!base, "RAEA_PROOF_HTTP=<url of a local server> (run.sh starts one)");
    // the plugin fetches on the IO pool, so the answer lands a few frames later
    const answer = (id: number) => until(app, () => app.call<Record<string, unknown> | null>("http/response", { id }), `a response to request ${id}`);
    const get = answer(app.call<number>("http/request", { url: `${base}/hello`, headers: { "x-proof": "yes" } }));
    check(get.ok === true && get.status === 200, `GET: ${JSON.stringify(get)}`);
    const got = JSON.parse(get.text as string);
    check(got.path === "/hello" && got.method === "GET" && got.proof === "yes", `GET echoed: ${get.text}`);
    const post = answer(app.call<number>("http/request", { url: `${base}/things`, method: "POST", json: { n: 3 } }));
    const posted = JSON.parse(post.text as string);
    check(posted.method === "POST" && JSON.parse(posted.body).n === 3, `POST echoed: ${post.text}`);
    const missing = answer(app.call<number>("http/request", { url: `${base}/missing` }));
    check(missing.status === 404 && missing.ok === false, `404: ${JSON.stringify(missing)}`);
    return { get: got, post: posted, missing: missing.status };
  },
  console(app) {
    const names = app.call<string[]>("console/commands");
    check(["help", "clear", "exit"].every((n) => names.includes(n)), `commands: ${names}`);
    app.call("console/run", { line: "help" });
    app.step(2);
    const out = app.call<string[]>("console/output");
    check(out.some((l) => l.includes("Available commands")) && out.some((l) => l.includes("clear")), `help printed: ${JSON.stringify(out)}`);
    check(app.call<string[]>("console/output").length === 0, "output is read once");
    let unknown = "";
    try {
      app.call("console/run", { line: "fly --high" });
    } catch (e) {
      unknown = String(e);
    }
    check(unknown.includes("no console command fly"), `an unknown command is an error: ${unknown}`);
    return { commands: names, printed: out.length };
  },
  simple_tilemap(app) {
    const dir = native("host.env", { name: "RAEA_PROOF_DIR" })!;
    const map = app.call<number>("simple_tilemap/map", { image: `${dir}/sheet.png`, tile: [32, 32], columns: 2, rows: 1 });
    const n = app.call<number>("simple_tilemap/set", {
      entity: map,
      tiles: [{ at: [0, 0], sprite: 1 }, { at: [70, -3], sprite: 0, flip_x: true }, { at: [1, 1, 2], sprite: 1, color: [1, 0, 0, 1] }],
    });
    check(n === 3, `queued ${n}`);
    app.step(2);
    const a = app.call<{ sprite: number } | null>("simple_tilemap/get", { entity: map, at: [0, 0] });
    const far = app.call<{ sprite: number; flip_x: boolean } | null>("simple_tilemap/get", { entity: map, at: [70, -3] });
    const layered = app.call<{ color: number[] } | null>("simple_tilemap/get", { entity: map, at: [1, 1, 2] });
    check(a?.sprite === 1 && far?.sprite === 0 && far.flip_x && layered?.color[1] === 0, `tiles read back: ${JSON.stringify([a, far, layered])}`);
    check(app.call("simple_tilemap/get", { entity: map, at: [5, 5] }) === null, "an unset tile is null");
    app.call("simple_tilemap/clear", { entity: map, layer: 2 });
    app.step(2);
    check(app.call("simple_tilemap/get", { entity: map, at: [1, 1, 2] }) === null && app.call("simple_tilemap/get", { entity: map, at: [0, 0] }) !== null, "clearing layer 2 leaves layer 0");
    return { tiles: [a, far, layered] };
  },
  sprite3d(app) {
    const dir = native("host.env", { name: "RAEA_PROOF_DIR" })!;
    const whole = app.spawn(w(sprite3d.SpriteQuad, part({ image: `${dir}/sheet.png`, pixels_per_metre: 32 })), at(0, 0, 0));
    const cell = app.spawn(w(sprite3d.SpriteQuad, part({ image: `${dir}/sheet.png`, pixels_per_metre: 16, atlas: { tile: [32, 32], columns: 2, rows: 1, index: 1 } })), at(3, 0, 0));
    type State = { loaded: boolean; size: [number, number] | null; index: number | null };
    let a: State = { loaded: false, size: null, index: null };
    let b: State = a;
    for (let i = 0; i < 2000 && !(a.loaded && b.loaded); i++) {
      app.step(1);
      a = app.call<State>("sprite3d/state", { entity: whole });
      b = app.call<State>("sprite3d/state", { entity: cell });
    }
    check(a.loaded && b.loaded, `the image should load: ${JSON.stringify([a, b])}`);
    const near = (s: [number, number] | null, w_: number, h: number) => !!s && Math.abs(s[0] - w_) < 1e-3 && Math.abs(s[1] - h) < 1e-3;
    check(near(a.size, 2, 1), `64x32 px at 32 px/m is 2x1 m: ${a.size}`);
    check(near(b.size, 2, 2) && b.index === 1, `one 32 px cell at 16 px/m is 2x2 m: ${JSON.stringify(b)}`);
    return { whole: a.size, cell: b.size };
  },
  lookup_curve(app) {
    const c = app.call<number>("lookup_curve/curve", { knots: [{ position: [0, 0], interpolation: "Linear" }, { position: [1, 2], interpolation: "Linear" }, { position: [2, 2] }] });
    const ys = app.call<number[]>("lookup_curve/sample", { curve: c, x: [0, 0.5, 1, 1.5, 2] });
    check(Math.abs(ys[1] - 1) < 1e-4 && Math.abs(ys[2] - 2) < 1e-4 && Math.abs(ys[4] - 2) < 1e-4, `linear knots: ${ys}`);
    const knots = app.call<{ position: [number, number] }[]>("lookup_curve/knots", { curve: c });
    check(knots.length === 3 && knots[1].position[1] === 2, `knots: ${JSON.stringify(knots)}`);
    return { samples: ys };
  },
  tweening(app) {
    const e = app.spawn(at(0, 0, 0));
    app.call("tweening/tween", { entity: e, lens: "position", start: [0, 0, 0], end: [4, 0, 0], duration: 1, ease: "Linear" });
    check(app.call<boolean>("tweening/playing", { entity: e }), "the tween should be playing");
    app.step(30);
    const mid = app.get(e, Transform)!.translation!;
    check(mid[0] > 1.5 && mid[0] < 2.5, `halfway at 60 fps after 30 frames: ${mid}`);
    app.step(40);
    const end = app.get(e, Transform)!.translation!;
    check(Math.abs(end[0] - 4) < 1e-3 && !app.call<boolean>("tweening/playing", { entity: e }), `it ends at 4 and is removed: ${end}`);
    let bad = "";
    try {
      app.call("tweening/tween", { entity: e, lens: "wobble", start: 0, end: 1, duration: 1 });
    } catch (err) {
      bad = String(err);
    }
    check(bad.includes("no lens wobble"), `an unknown lens is an error: ${bad}`);
    // a colour lens: a sprite from white to black, sRGB, halfway after 30 frames
    const Sprite = component<{ color?: { Srgba: { red: number; green: number; blue: number; alpha: number } } }>("bevy_sprite::sprite::Sprite");
    const s = app.spawn(w(Sprite, {}));
    app.call("tweening/tween", { entity: s, lens: "sprite_color", start: [1, 1, 1, 1], end: [0, 0, 0, 1], duration: 1 });
    app.step(30);
    const grey = app.get(s, Sprite)!.color!;
    check(Math.abs(grey.Srgba.red - 0.5) < 0.05 && Math.abs(grey.Srgba.blue - 0.5) < 0.05, `halfway grey: ${JSON.stringify(grey)}`);
    return { mid, end, grey };
  },
  tween(app) {
    // a runner of one second with one span child: Linear, the target's translation from 0 to 4 on x
    const ChildOf = component<number>("bevy_ecs::hierarchy::ChildOf");
    const target = app.spawn(w(tween.AnimationTarget, {}), at(0, 0, 0));
    const second = { secs: 1, nanos: 0 };
    const runner = app.spawn(w(tween.TimeRunner, { paused: false, elasped: { now: 0, now_period: 0, previous: 0, previous_period: 0 }, length: second, direction: "Forward", time_scale: 1, repeat: null }));
    app.spawn(
      w(ChildOf, runner),
      w(tween.TimeSpan, { min: { Inclusive: { secs: 0, nanos: 0 } }, max: { Inclusive: second } }),
      w(tween.EaseKind, "Linear"),
      w(tween.TweenTargetComponentTranslation, { target: { Entity: target }, interpolator: { start: [0, 0, 0], end: [4, 0, 0], delta: false } }),
    );
    app.step(30);
    const mid = app.get(target, Transform)!.translation!;
    check(mid[0] > 1.5 && mid[0] < 2.5, `halfway at 60 fps after 30 frames: ${mid}`);
    app.step(40);
    const end = app.get(target, Transform)!.translation!;
    check(Math.abs(end[0] - 4) < 1e-3, `it ends at 4: ${end}`);
    return { mid, end };
  },
  debugdump(app) {
    const names = app.call<string[]>("debugdump/schedules");
    check(names.includes("Update"), `schedules: ${names.join(", ")}`);
    const dot = app.call<string>("debugdump/schedule", { schedule: "Update" });
    check(dot.startsWith("digraph"), `dot: ${dot.slice(0, 40)}`);
    return { schedules: names.length, dotBytes: dot.length };
  },
  big_space(app) {
    // a grid of 10 m cells under a BigSpace root, and a body put 25 m out in cell 0: the plugin moves it into the cell
    // it's in and keeps the rest as its Transform. Cells are centred (cell n spans n * 10 +- 5 m), so that's cell 3, -5 m
    const ChildOf = component<number>("bevy_ecs::hierarchy::ChildOf");
    // a glam DAffine3 travels as its 12 numbers, column by column (the matrix's axes, then the translation)
    const identity = [1, 0, 0, 0, 1, 0, 0, 0, 1, 0, 0, 0];
    const origin = { cell: { x: 0, y: 0, z: 0 }, translation: [0, 0, 0], rotation: [0, 0, 0, 1], grid_transform: identity, is_local_origin_unchanged: false };
    const root = app.spawn(w(bigSpace.BigSpace, { floating_origin: null }), w(bigSpace.Grid, { local_floating_origin: origin as never, cell_edge_length: 10, maximum_distance_from_origin: 10 }), at(0, 0, 0));
    app.spawn(w(bigSpace.FloatingOrigin, {}), w(bigSpace.CellCoord, { x: 0, y: 0, z: 0 }), w(ChildOf, root), at(0, 0, 0));
    const body = app.spawn(w(bigSpace.CellCoord, { x: 0, y: 0, z: 0 }), w(ChildOf, root), at(25, 0, 0));
    app.step(4);
    const cell = app.get(body, bigSpace.CellCoord)!;
    const t = app.get(body, Transform)!.translation!;
    check(cell.x === 3 && Math.abs(t[0] + 5) < 1e-3, `25 m in centred 10 m cells is cell 3 - 5 m: ${JSON.stringify(cell)} ${t}`);
    return { cell, translation: t };
  },
  rand(app) {
    // a seeded entity gets its own generator from the plugin (WyRand is opaque to reflection: its presence is the proof)
    const seeded = app.spawn(w(rand.RngSeedWyRand, { seed: [1, 2, 3, 4, 5, 6, 7, 8] }));
    app.step(2);
    const has = app.query({ has: [rand.WyRand] }).map((r) => r.entity);
    check(has.includes(seeded), `the seeded entity should have a WyRand: ${has}`);
    return { seeded: has.length };
  },
  kira(app) {
    // a silent clip, so a proof never sounds through the machine's speakers (kira opens the default output device)
    const dir = native("host.env", { name: "RAEA_PROOF_DIR" })!;
    const n = app.call<number>("kira/play", { path: `${dir}/silence.wav`, looped: true, volume: -12 });
    type S = { state: string; position: number | null };
    const state = () => app.call<S>("kira/state", { instance: n });
    const playing = until(app, () => (state().state === "Playing" ? state() : null), "Playing state");
    app.call("kira/volume", { instance: n, db: -24 });
    app.call("kira/stop", { instance: n });
    const stopped = until(app, () => (state().state === "Stopped" ? state() : null), "Stopped state");
    return { playing, stopped: stopped.state };
  },
  quinnet(app) {
    const { port } = app.call<{ port: number }>("quinnet/listen");
    check(port > 0, `listen gave port ${port}`);
    app.call("quinnet/connect", { port });
    type St = { listening: boolean; clients: number[]; client: string | null; id: number | null };
    const st = until(app, () => {
      const s = app.call<St>("quinnet/status");
      return s.client === "Connected" && s.clients.length === 1 && s.id !== null ? s : null;
    }, "connection");
    const client = st.clients[0];
    app.call("quinnet/send", { text: "hello server" });
    const atServer = until(app, () => app.call<string[]>("quinnet/receive", { client }).at(0), "message at the server");
    check(atServer === "hello server", `server got ${atServer}`);
    app.call("quinnet/send", { client, text: "hello client" });
    const atClient = until(app, () => app.call<string[]>("quinnet/receive").at(0), "message at the client");
    check(atClient === "hello client", `client got ${atClient}`);
    return { port, client, atServer, atClient };
  },
  ...games,
};

export default function main() {
  const name = native("host.env", { name: "RAEA_PROOF" });
  try {
    if (!name) throw new Error("RAEA_PROOF=<plugin> names the plugin to prove");
    const plugins = with_[name] ?? [name];
    const app = BevyApp.open({ plugins, fps: 60 });
    app.step(30);
    const drive = drives[name];
    const measured = drive ? drive(app) : {};
    const own = app.methods.filter((m) => !m.startsWith("world.") && !m.startsWith("registry.") && !m.startsWith("rpc.") && !m.startsWith("schedule.") && !m.startsWith("raea/"));
    print(JSON.stringify({ proof: name, driven: !!drive, frames: app.step(1), methods: own, ...measured }));
    app.close();
  } catch (e) {
    print(`FAIL ${name}: ${e instanceof Error ? e.message : String(e)}`);
    throw e;
  }
}
