// An AmbientLight of brightness 0 turns the ambient off, as Bevy's does: it is not dropped for the camera's default 80
// (runtime/look/src/camera.rs). Runs where the release binary and ffmpeg are built and on PATH.
import { expect, test } from "bun:test";
import { existsSync, mkdtempSync, rmSync } from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";

const BIN = process.env.RAEA_BIN ?? join(import.meta.dir, "../../../target/release/raea");
const ffmpeg = Bun.which("ffmpeg");

/** the frame's centre pixel, RGB */
function centre(id: string, dir: string): number[] {
  const png = join(dir, `${id}.png`);
  const r = Bun.spawnSync([BIN, "render", join(import.meta.dir, "ambient-zero/scene.tsx"), "--composition", id, "--frame", "0", "-o", png], { stderr: "pipe" });
  expect(r.exitCode).toBe(0);
  const px = new Uint8Array(Bun.spawnSync([ffmpeg!, "-v", "error", "-i", png, "-f", "rawvideo", "-pix_fmt", "rgb24", "-"]).stdout);
  const i = (45 * 160 + 80) * 3;
  return [...px.subarray(i, i + 3)];
}

test.skipIf(!existsSync(BIN) || !ffmpeg)("AmbientLight brightness 0 leaves an unlit box black; 80 lights it", () => {
  const dir = mkdtempSync(join(tmpdir(), "ambient-zero-"));
  try {
    expect(Math.max(...centre("dark", dir))).toBeLessThan(8);
    expect(Math.min(...centre("lit", dir))).toBeGreaterThan(40);
  } finally {
    rmSync(dir, { recursive: true, force: true });
  }
}, 120_000);
