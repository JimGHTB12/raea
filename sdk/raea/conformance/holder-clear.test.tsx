// The 3D clears to its nearest holder's background, and a holder's new background is seen on the frame it changes:
// the holders are kept across frames and worked out again when a holder's props change (runtime/react holders.rs
// HolderCache). Runs where the release binary and ffmpeg are built and on PATH.
import { expect, test } from "bun:test";
import { existsSync, mkdtempSync, readdirSync, rmSync } from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";

const BIN = process.env.RAEA_BIN ?? join(import.meta.dir, "../../../target/release/raea");
const ffmpeg = Bun.which("ffmpeg");

/** a PNG's top-left pixel, RGB */
function corner(png: string): number[] {
  const px = new Uint8Array(Bun.spawnSync([ffmpeg!, "-v", "error", "-i", png, "-f", "rawvideo", "-pix_fmt", "rgb24", "-"]).stdout);
  return [...px.subarray(0, 3)];
}

/** frames 2 and 8 of one of holder-clear's scenes, each as its top-left pixel */
function corners(scene: string): number[][] {
  const dir = mkdtempSync(join(tmpdir(), "holder-clear-"));
  try {
    const r = Bun.spawnSync([BIN, "render", join(import.meta.dir, "holder-clear", scene), "--composition", "main", "--at", "2,8", "-o", dir], { stderr: "pipe" });
    expect(r.exitCode).toBe(0);
    return readdirSync(dir).filter((f) => f.endsWith(".png")).sort().map((f) => corner(join(dir, f)));
  } finally {
    rmSync(dir, { recursive: true, force: true });
  }
}

function redThenBlue([before, after]: number[][]) {
  expect(before[0]).toBeGreaterThan(200);
  expect(before[2]).toBeLessThan(40);
  expect(after[2]).toBeGreaterThan(200);
  expect(after[0]).toBeLessThan(40);
}

test.skipIf(!existsSync(BIN) || !ffmpeg)("a holder's background turning blue at frame 5 clears the 3D blue from frame 5", () => {
  redThenBlue(corners("scene.tsx"));
}, 120_000);

// the holders kept across frames: a 3D that moves to another holder clears to the new one (before, a commit's move
// carried the tick the watch had just read at, so it was never seen and frame 8 cleared red)
test.skipIf(!existsSync(BIN) || !ffmpeg)("a 3D moved at frame 5 from a red holder to a blue one clears blue from frame 5", () => {
  redThenBlue(corners("moved.tsx"));
}, 120_000);
