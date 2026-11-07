// A layered Canvas3D's clearColor (docs/control.feature FC0a, 2d/transparency_2d): the 3D clears to it instead of
// transparent, so its translucent parts blend over it as Bevy's do over its ClearColor, not over the page (runtime/react
// holders.rs scene_clear). Runs where the release binary and ffmpeg are built.
import { expect, test } from "bun:test";
import { existsSync, mkdtempSync, rmSync } from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";

const BIN = process.env.RAEA_BIN ?? join(import.meta.dir, "../../../target/release/raea");
const ffmpeg = Bun.which("ffmpeg");

/** frame 1 of a layered-clear scene: its corner pixel and its middle one, RGB */
function pixels(scene: string): { corner: number[]; middle: number[] } {
  const dir = mkdtempSync(join(tmpdir(), "layered-clear-"));
  try {
    const png = join(dir, "1.png");
    const r = Bun.spawnSync([BIN, "render", join(import.meta.dir, "layered-clear", scene), "--frame", "1", "-o", png], { stderr: "pipe" });
    expect(r.exitCode).toBe(0);
    const px = new Uint8Array(Bun.spawnSync([ffmpeg!, "-v", "error", "-i", png, "-f", "rawvideo", "-pix_fmt", "rgb24", "-"]).stdout);
    const at = (x: number, y: number) => [...px.subarray((y * 160 + x) * 3, (y * 160 + x) * 3 + 3)];
    return { corner: at(2, 2), middle: at(80, 45) };
  } finally {
    rmSync(dir, { recursive: true, force: true });
  }
}

test.skipIf(!existsSync(BIN) || !ffmpeg)("with clearColor the 3D clears to it and a half-white square blends over it, not the page", () => {
  const { corner, middle } = pixels("scene.tsx");
  expect(corner).toEqual([0, 0, 0]);
  // grey: white over black, none of the page's red under it
  expect(middle[1]).toBeGreaterThan(100);
  expect(Math.abs(middle[0] - middle[1])).toBeLessThan(4);
}, 120_000);

test.skipIf(!existsSync(BIN) || !ffmpeg)("without clearColor the 3D clears transparent and the page shows under it", () => {
  const { corner, middle } = pixels("none.tsx");
  expect(corner).toEqual([255, 0, 0]);
  expect(middle[0]).toBe(255);
}, 120_000);
