// An <Animate> that React renders once is still sampled on every frame by the engine (runtime/react animate.rs, which
// visits only the elements whose props changed and the animated ones). Runs where the release binary and ffmpeg are
// built and on PATH.
import { expect, test } from "bun:test";
import { existsSync, mkdtempSync, readdirSync, rmSync } from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";

const BIN = process.env.RAEA_BIN ?? join(import.meta.dir, "../../../target/release/raea");
const ffmpeg = Bun.which("ffmpeg");

test.skipIf(!existsSync(BIN) || !ffmpeg)("a box fading in over frames 0-6 is black at frame 0 and white at frame 12", () => {
  const dir = mkdtempSync(join(tmpdir(), "animate-held-"));
  try {
    const r = Bun.spawnSync([BIN, "render", join(import.meta.dir, "animate-held/fade.tsx"), "--composition", "main", "--at", "0,12", "-o", dir], { stderr: "pipe" });
    expect(r.exitCode).toBe(0);
    const grey = readdirSync(dir).filter((f) => f.endsWith(".png")).sort().map((f) => new Uint8Array(Bun.spawnSync([ffmpeg!, "-v", "error", "-i", join(dir, f), "-f", "rawvideo", "-pix_fmt", "gray", "-"]).stdout)[0]);
    expect(grey[0]).toBeLessThan(20);
    expect(grey[1]).toBeGreaterThan(235);
  } finally {
    rmSync(dir, { recursive: true, force: true });
  }
}, 120_000);
