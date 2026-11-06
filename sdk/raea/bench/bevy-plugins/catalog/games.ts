// The game plugins' drives (docs/plugins.feature PL0a) and save's (PL16), spread into proof.ts's
import { at, with as w } from "raea/bevy";
import { easings } from "raea/plugins/easings";
import { native } from "raea/native";
import { Drive, Transform, check, part } from "./drive";

// within a frame (1/60 s) of the curve
const near = (a: number, b: number, what: string, by = 0.15) => check(Math.abs(a - b) < by, `${what}: ${a}, not ${b}`);

export const games: Record<string, Drive> = {
  easings(app) {
    // EaseTransform eases from where the entity is: x 0 to 4 over 1 s, linearly, so halfway (2) at 30 frames of 60 fps
    // and 4 at the end; QuadraticIn is at a quarter (1) halfway, which a linear fallback would miss
    const to = { translation: [4, 0, 0], rotation: [0, 0, 0, 1], scale: [1, 1, 1] };
    const linear = app.spawn(at(0, 0, 0), w(easings.EaseTransform, part({ to, seconds: 1, function: "Linear" })));
    const quad = app.spawn(at(0, 0, 0), w(easings.EaseTransform, part({ to, seconds: 1, function: "QuadraticIn" })));
    const x = (e: number) => app.get(e, Transform)!.translation![0];
    app.step(30);
    const half = [x(linear), x(quad)];
    app.step(40);
    const end = [x(linear), x(quad)];
    near(half[0], 2, "linear at half time");
    near(half[1], 1, "QuadraticIn at half time");
    near(end[0], 4, "linear at the end", 1e-3);
    near(end[1], 4, "QuadraticIn at the end", 1e-3);
    return { half, end };
  },
  navigator(app) {
    // a 10 by 10 room with a wall from the floor to y = 8 between x 4 and 6: the way from (1, 1) to (9, 1) climbs over
    // it, (1, 1) to (4, 8) to (6, 8) to (9, 1), about 17.23; the same room rebuilt without the wall is the straight 8
    type Path = { length: number; points: [number, number][] } | null;
    const room: [number, number][] = [[0, 0], [10, 0], [10, 10], [0, 10]];
    const mesh = app.call<number>("navigator/mesh", { edges: room, obstacles: [[[4, 0], [6, 0], [6, 8], [4, 8]]] });
    const round = app.call<Path>("navigator/path", { mesh, from: [1, 1], to: [9, 1] });
    check(!!round, "no way round the wall");
    near(round!.length, 2 * Math.hypot(3, 7) + 2, "the way over the wall");
    check(round!.points.some(([, y]) => Math.abs(y - 8) < 1e-3), `the way should pass the wall's top: ${JSON.stringify(round!.points)}`);
    check(app.call<Path>("navigator/path", { mesh, from: [5, 4], to: [9, 1] }) === null, "a start inside the wall has no way");
    app.call("navigator/mesh", { edges: room, mesh });
    const straight = app.call<Path>("navigator/path", { mesh, from: [1, 1], to: [9, 1] });
    near(straight?.length ?? 0, 8, "the way with the wall gone");
    return { round: round!.length, points: round!.points, straight: straight!.length };
  },
  northstar(app) {
    // a 10 by 10 grid walled at x = 5 but for y = 9: A* from [0, 0] to [9, 0] goes through the gap; walled in on all
    // sides, [9, 0] has no way at all
    type Path = { cost: number; points: [number, number, number][] } | null;
    const walls = Array.from({ length: 9 }, (_, y) => [5, y]);
    const grid = app.call<number>("northstar/grid", { size: [10, 10], walls, chunk: 5 });
    const way = app.call<Path>("northstar/path", { grid, from: [0, 0], to: [9, 0], mode: "astar" });
    check(!!way, "no way through the gap");
    check(way!.points.some(([x, y]) => x === 5 && y === 9), `the way should pass the gap at [5, 9]: ${JSON.stringify(way!.points)}`);
    check(JSON.stringify(way!.points.at(-1)) === "[9,0,0]", `the way should end at [9, 0]: ${way!.points.at(-1)}`);
    app.call("northstar/grid", { size: [10, 10], walls: [...walls, [5, 9], [8, 0], [8, 1], [9, 1]], chunk: 5, grid });
    check(app.call<Path>("northstar/path", { grid, from: [0, 0], to: [9, 0], mode: "astar" }) === null, "a walled-in goal has no way");
    return { cost: way!.cost, steps: way!.points.length };
  },
  save(app) {
    // two of three entities marked Save, saved, one of them moved, then loaded: the file's two come back as new
    // entities where they were saved (the old ones despawned, the move with them) and the unmarked one stays put; the
    // loaded ones are marked again, so a second save takes them
    const xs = [1, 2, 3].map((x) => app.spawn(at(x, 0, 0)));
    app.call("save/mark", { entities: xs.slice(0, 2) });
    const path = `${native("host.env", { name: "TMPDIR" }) ?? "/tmp"}/raea-proof-save.ron`;
    const saved = app.call<{ saved: number[] }>("save/save", { path }).saved;
    check(JSON.stringify([...saved].sort()) === JSON.stringify(xs.slice(0, 2).sort()), `saved ${saved}, not ${xs.slice(0, 2)}`);
    app.insert(xs[0], w(Transform, part({ translation: [10, 0, 0] })));
    const loaded = app.call<{ loaded: number[] }>("save/load", { path }).loaded;
    const x = (e: number) => app.get(e, Transform)?.translation?.[0];
    const back = loaded.map(x).sort();
    check(JSON.stringify(back) === "[1,2]", `loaded at ${back}, not 1 and 2`);
    check(!loaded.includes(xs[0]) && app.query({ data: [Transform] }).every((q) => x(q.entity) !== 10), "the moved entity should be gone");
    check(x(xs[2]) === 3, `the unmarked entity moved to ${x(xs[2])}`);
    const again = app.call<{ saved: number[] }>("save/save", { path }).saved;
    check(JSON.stringify([...again].sort()) === JSON.stringify([...loaded].sort()), `the second save took ${again}, not ${loaded}`);
    return { saved, loaded, back };
  },
};
