// The TS Transform/Quat helpers against glam's answers for the same inputs (docs/control.feature FC1d). glam.json comes
// from core/scene/tests/glam_fixture.rs: `cargo test --release -p raea-core-scene --test glam_fixture -- --ignored --nocapture`.
import { expect, test } from "bun:test";
import { Quat, Transform } from "@raea/render/bevy3d";
import fixture from "./glam.json";

type Q = [number, number, number, number];
type V = [number, number, number];
// q and -q are the same turn
const near = (got: number[], want: number[]) => {
  const sign = got.reduce((s, g, i) => s + g * want[i], 0) < 0 && got.length === 4 ? -1 : 1;
  for (let i = 0; i < want.length; i++) expect(Math.abs(sign * got[i] - want[i])).toBeLessThan(1e-6);
};

for (const [n, row] of (fixture as any[]).entries()) {
  test(`${n} ${row.op}${row.order ? " " + row.order : ""}`, () => {
    switch (row.op) {
      case "x": return near(Quat.fromRotationX(row.a), row.q);
      case "y": return near(Quat.fromRotationY(row.a), row.q);
      case "z": return near(Quat.fromRotationZ(row.a), row.q);
      case "axis": return near(Quat.fromAxisAngle(row.axis as V, row.a), row.q);
      case "euler": return near(Quat.fromEuler(row.order, ...(row.abc as V)), row.q);
      case "mul": return near(Quat.mul(row.a as Q, row.b as Q), row.q);
      case "mulVec3": return near(Quat.mulVec3(row.q as Q, row.v as V), row.out);
      case "lookingAt": return near(Transform.fromTranslation(row.eye as V).lookingAt(row.target as V, row.up as V).rotation, row.q);
      default: throw new Error(`unknown op ${row.op}`);
    }
  });
}

test("the fixture covers every op", () => {
  expect(new Set((fixture as any[]).map((r) => r.op))).toEqual(new Set(["x", "y", "z", "axis", "euler", "mul", "mulVec3", "lookingAt"]));
});
