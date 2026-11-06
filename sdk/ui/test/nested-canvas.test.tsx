import { beforeAll, expect, test } from "bun:test";
import { install } from "../src/dom";

beforeAll(() => install());
const g = globalThis as any;

test("a canvas drawn on another goes as its list, copied, keyed by what it holds (systems/canvas replays it)", () => {
  const a = g.document.createElement("canvas");
  a.width = 20;
  a.height = 10;
  const ca = a.getContext("2d");
  ca.fillStyle = "red";
  ca.fillRect(0, 0, 5, 5);
  const b = g.document.createElement("canvas");
  const cb = b.getContext("2d");
  cb.drawImage(a, 3, 4, 10, 5);
  const op = (cb as any).list.at(-1);
  expect(op[0]).toBe("canvas");
  expect(op[1]).toMatchObject({ w: 20, h: 10 });
  expect(op[1].ops).toHaveLength(1);
  expect(op.slice(2, 10)).toEqual([null, null, null, null, 3, 4, 10, 5]);
  // drawn on after: the copy sent before is unchanged and the next draw has a new key
  const key = op[1].key;
  ca.fillRect(5, 5, 5, 5);
  expect(op[1].ops).toHaveLength(1);
  cb.drawImage(a, 0, 0);
  expect((cb as any).list.at(-1)[1].key).not.toBe(key);
});
