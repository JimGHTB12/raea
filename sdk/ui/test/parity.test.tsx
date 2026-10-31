// Panels driven through React's adapter: with and without memo they leave the identical host tree after the mount and
// after every step, compared node for node (kind, props, text, listeners, children), at 20 and at 500 rows, and each
// step sends only the ops it needs (from the wire's proof bench; the core driver it was compared with went
// with raea-runtime-js's bevy_ui host, 10-27).
import { describe, expect, test } from "bun:test";
import { mount as mountReact } from "../src/react";
import { restart, type Ev } from "../src/wire";
import { Host, kinds } from "./host";
import { Shelf as ShelfReact } from "./panels/shelf-react";
import { Tasks as TasksReact } from "./panels/tasks-react";

type Step = [string, (h: Host) => [string | number, Ev, unknown]];
type Run = { name: string; host: Host; shapes: string[]; ops: number[]; kinds: Record<string, number>[] };

function drive(name: string, start: () => void, steps: Step[]): Run {
  restart();
  const host = new Host();
  start();
  const run: Run = { name, host, shapes: [JSON.stringify(host.shape())], ops: [host.since(0).length], kinds: [kinds(host.since(0))] };
  for (const [, step] of steps) {
    const from = host.batches.length;
    const [t, ev, payload] = step(host);
    host.send(t, ev, payload);
    const ops = host.since(from);
    run.shapes.push(JSON.stringify(host.shape()));
    run.ops.push(ops.length);
    run.kinds.push(kinds(ops));
  }
  return run;
}

const press = { x: 0, y: 0, clientX: 0, clientY: 0, button: 0, detail: 1 };
const list = (h: Host) => h.nodes.get(h.tid("list"))!.children;
const TASKS: Step[] = [
  ["type a char", () => ["draft", "change", "Buy milk"]],
  ["enter (adds a row)", () => ["draft", "keydown", { key: "Enter", value: "Buy milk" }]],
  ["remove that row", (h) => [`x-${h.nodes.get(list(h).at(-1)!)!.props["data-testid"]}`, "pointerdown", press]],
  ["toggle (the reflow)", () => ["toggle", "pointerdown", press]],
  ["toggle back", () => ["toggle", "pointerdown", press]],
  ["mark a row done", () => ["r5", "pointerdown", press]],
  ["unmark it", () => ["r5", "pointerdown", press]],
  ["move last row to front", () => ["rotate", "pointerdown", press]],
  ["move first row to end", () => ["unrotate", "pointerdown", press]],
  ["all done (shows a line)", () => ["all", "pointerdown", press]],
  ["all undone (hides it)", () => ["all", "pointerdown", press]],
  ["add a second row", () => ["draft", "change", "Call mum"]],
  ["enter it", () => ["draft", "keydown", { key: "Enter", value: "Call mum" }]],
  ["remove a middle row", () => ["x-r3", "pointerdown", press]],
];

function same(a: Run, b: Run, steps: Step[]) {
  expect(b.shapes[0]).toBe(a.shapes[0]);
  steps.forEach(([what], i) => {
    if (a.shapes[i + 1] !== b.shapes[i + 1]) throw new Error(`${a.name} and ${b.name} differ after "${what}"`);
  });
}

for (const n of [20, 500]) {
  describe(`the tasks panel, ${n} rows`, () => {
    const react = drive("react", () => void mountReact(<TasksReact n={n} useMemo={false} />), TASKS);
    const memo = drive("react+memo", () => void mountReact(<TasksReact n={n} useMemo={true} />), TASKS);

    test("the same tree after the mount and every step, with memo and without", () => same(react, memo, TASKS));
    test("the same ops a step with memo and without; a row moved to the front re-appends every row after it", () => {
      for (let i = 0; i <= TASKS.length; i++) expect([i, react.ops[i]]).toEqual([i, memo.ops[i]]);
      const front = 1 + TASKS.findIndex(([s]) => s === "move last row to front");
      expect(react.kinds[front]).toEqual({ append: n - 1 });
    });
    test("one update an element a step", () => {
      for (const r of [react])
        for (const b of r.host.batches) {
          const ids = b.filter((o) => o.op === "update").map((o) => (o as { id: number }).id);
          expect(new Set(ids).size).toBe(ids.length);
        }
    });
  });
}

const SHELF: Step[] = [
  ["theme (context: every row restyles)", () => ["theme", "pointerdown", press]],
  ["mode (a Dynamic swaps kinds)", () => ["mode", "pointerdown", press]],
  ["grow (a style object changes)", () => ["grow", "pointerdown", press]],
  ["pad (a style appears, Show flips)", () => ["pad", "pointerdown", press]],
  ["rename (a span's text)", () => ["rename", "pointerdown", press]],
  ["lock (rows stop listening)", () => ["lock", "pointerdown", press]],
  ["unlock (they listen again)", () => ["lock", "pointerdown", press]],
  ["pick a row", () => ["i1", "pointerdown", press]],
  ["a row at the front", () => ["front", "pointerdown", press]],
  ["cut the third", () => ["cut", "pointerdown", press]],
  ["type", () => ["field", "change", "hi"]],
  ["focus by ref", () => ["focus", "pointerdown", press]],
  ["pad again (the style goes)", () => ["pad", "pointerdown", press]],
  ["mode again", () => ["mode", "pointerdown", press]],
  ["theme again", () => ["theme", "pointerdown", press]],
];

describe("the shelf panel", () => {
  const react = drive("react", () => void mountReact(<ShelfReact />), SHELF);
  test("a locked row doesn't listen; unlocked, it does", () => {
    const lock = 1 + SHELF.findIndex(([s]) => s.startsWith("lock"));
    for (const r of [react]) {
      expect(r.kinds[lock]).toEqual({ unlisten: 5, update: 1 });
      expect(r.kinds[lock + 1]).toEqual({ listen: 5, update: 1 });
    }
  });
  test("a ref's focus asks the host to focus that element by its wire id", () => {
    for (const r of [react]) expect(r.host.emits).toEqual([["focus", r.host.tid("field")]]);
  });
  test("the style is sent whole, and unset when it goes", () => {
    for (const r of [react]) {
      const ups = r.host.batches.flat().filter((o) => o.op === "update" && o.id === r.host.tid("pad"));
      expect(ups).toEqual([
        { op: "update", id: r.host.tid("pad"), set: { style: { padding: 4 } } },
        { op: "update", id: r.host.tid("pad"), unset: ["style"] },
      ]);
    }
  });
  test("one front row is one create, one insert", () => {
    const at = 1 + SHELF.findIndex(([s]) => s === "a row at the front");
    expect(react.kinds[at]).toEqual({ create: 2, append: 1, insert: 1, listen: 1 });
  });
});
