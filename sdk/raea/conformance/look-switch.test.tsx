// A frame <Look> per scene (spec/motion.feature, "3D: the rest of a scene"): one composition whose Sequences are none,
// paint, cartoon and none again. Each frame emits only its own scene's Look, and the engine draws each frame in the
// look of its scene, rendered one after another in one process (raea render --at), so a look comes on, changes
// and goes back to plain mid-piece. The render part runs where the release binary and ffmpeg are built and on PATH.
import { describe, expect, test } from "bun:test";
import { existsSync, mkdtempSync, readdirSync, rmSync } from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";
import { renderFrame } from "../src/test";
import { CARTOON_SKY, PAINT_SKY, PLAIN_BG, SCENE, Scenes } from "./look-switch/scene";

const base = { fps: 30, width: 320, height: 180, durationInFrames: 4 * SCENE };
const looks = (frame: number) => [...renderFrame(<Scenes />, { ...base, frame }).matchAll(/data-3d="(Look|Outline)" data-props="([^"]*)"/g)].map((m) => ({ kind: m[1], ...JSON.parse(m[2].replace(/&quot;/g, '"')) }));

const BIN = process.env.RAEA_BIN ?? join(import.meta.dir, "../../../target/release/raea");
const ffmpeg = Bun.which("ffmpeg");

/** a PNG's pixels, RGB */
function pixels(png: string): { w: number; h: number; px: Uint8Array } {
  const probe = Bun.spawnSync([ffmpeg!, "-v", "error", "-i", png, "-f", "rawvideo", "-pix_fmt", "rgb24", "-"]);
  const px = new Uint8Array(probe.stdout);
  return { w: 320, h: px.length / (320 * 3), px };
}
const at = (p: ReturnType<typeof pixels>, x: number, y: number) => [...p.px.subarray((y * p.w + x) * 3, (y * p.w + x) * 3 + 3)];
const near = (a: number[], hex: string, tol = 24) => {
  const b = [1, 3, 5].map((i) => parseInt(hex.slice(i, i + 2), 16));
  return a.every((v, i) => Math.abs(v - b[i]) <= tol);
};
/** pixels near black: the cartoon's outline */
const inked = (p: ReturnType<typeof pixels>) => {
  let n = 0;
  for (let i = 0; i < p.px.length; i += 3) if (p.px[i] < 30 && p.px[i + 1] < 30 && p.px[i + 2] < 30) n++;
  return n;
};

describe("Look: a frame look per scene", () => {
  test("each Sequence's frames carry only its own Look", () => {
    expect(looks(5)).toEqual([]);
    expect(looks(15)).toMatchObject([{ kind: "Look", mode: "paint", sky: PAINT_SKY }]);
    const c = looks(25);
    expect(c.find((x) => x.kind === "Look")).toMatchObject({ mode: "paint", sky: CARTOON_SKY, bands: 2 });
    expect(c.find((x) => x.kind === "Outline")).toMatchObject({ color: "#000000", width: 8 });
    expect(looks(35)).toEqual([]);
  });

  test.skipIf(!existsSync(BIN) || !ffmpeg)("drawn in its own look, one frame after another: none, paint, cartoon, none", () => {
    const dir = mkdtempSync(join(tmpdir(), "look-switch-"));
    try {
      const r = Bun.spawnSync([BIN, "render", join(import.meta.dir, "look-switch/scene.tsx"), "--composition", "looks", "--at", "5,15,25,35", "-o", dir], { stderr: "pipe" });
      expect(r.exitCode).toBe(0);
      const pngs = readdirSync(dir).filter((f) => f.endsWith(".png")).sort();
      expect(pngs.length).toBe(4);
      const [none, paint, cartoon, after] = pngs.map((f) => pixels(join(dir, f)));
      // the corner is the scene's background: plain, the paint sky, the cartoon's sky, plain again
      expect(near(at(none, 4, 4), PLAIN_BG)).toBe(true);
      expect(near(at(paint, 4, 4), PAINT_SKY)).toBe(true);
      expect(near(at(cartoon, 4, 4), CARTOON_SKY)).toBe(true);
      expect(near(at(after, 4, 4), PLAIN_BG)).toBe(true);
      // the cartoon's outline is on in its scene only
      expect(inked(cartoon)).toBeGreaterThan(200);
      expect(inked(paint)).toBeLessThan(50);
      expect(inked(after)).toBeLessThan(50);
      // and the last scene is drawn as the first was
      const diff = none.px.reduce((s, v, i) => s + Math.abs(v - after.px[i]), 0) / none.px.length;
      expect(diff).toBeLessThan(3);
    } finally {
      rmSync(dir, { recursive: true, force: true });
    }
  }, 300_000);
});
