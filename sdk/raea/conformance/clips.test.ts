// The motion database (docs/clips.feature): tags read from CMU's descriptions, a database built from a few clips
// whose mirror images are exact, the kernel's integer twin ranking frames as the exact cost does, and the linear top-k.
import { describe, expect, test } from "bun:test";
import { readClips, type ClipFile } from "raea/clip";
import type { Skeleton } from "raea/character";
import { nearest } from "raea/matching";
import { BLOCK, FS, MF_SIGN, MF_SRC, NAN16, blockFrames, blockMins, buildDb, dbLibrary, kernelQuery, kernelScore, kernelSearch, parseDb, rerank, tagsOf, topK, topKFast } from "@raea/character/clips";
import kidJson from "../../../examples/character/media/characters/kid/skeleton.json" with { type: "json" };
import clipsJson from "../../../examples/character/parkour.json" with { type: "json" };

describe("tags", () => {
  test("CMU's descriptions", () => {
    expect(tagsOf("carry 5.5lb suitcase")).toEqual(expect.arrayContaining(["carry", "suitcase", "heavy"]));
    expect(tagsOf("moving heavy box")).toEqual(expect.arrayContaining(["carry", "box", "heavy"]));
    expect(tagsOf("walk forward and pick up object, carry back object")).toEqual(expect.arrayContaining(["walk", "pick", "carry"]));
    expect(tagsOf("CarefulWalk LookingAround")).toEqual(expect.arrayContaining(["walk", "look-around"]));
    expect(tagsOf("run/jog, sudden stop")).toEqual(expect.arrayContaining(["run", "jog"]));
    expect(tagsOf("walk")).not.toContain("styled");
    expect(tagsOf("Old", "100style", "FW")).toEqual(expect.arrayContaining(["100style", "fw", "walk", "styled"]));
    expect(tagsOf("Neutral", "100style", "ID")).not.toContain("styled");
  });
});

describe("a database of a few clips", () => {
  const skel = kidJson as unknown as Skeleton, clips = readClips(clipsJson as ClipFile);
  const names = [...clips.keys()].filter((n) => /^c(16_15|35_01|127_05|127_19|77_02|111_28)$/.test(n));
  const db = parseDb(buildDb(skel, names.map((name) => ({ name, source: "cmu", file: "", label: "walk", clip: clips.get(name)! }))));
  const lib = dbLibrary(skel, db, { read: () => clipsJson as ClipFile });
  test("a mirror image's features are the frame's permuted and negated, exactly", () => {
    const n0 = db.header.frames;
    for (let k = 0; k < n0; k += 7) for (let d = 0; d < 27; d++) {
      const v = db.F[k * FS + MF_SRC[d]], m = lib.full.F[(n0 + k) * FS + d];
      expect(m).toBe(v === NAN16 ? NAN16 : (MF_SIGN[d] * v) | 0);
    }
  });
  test("the kernel's twin's best blocks hold the exact best", () => {
    const q = { z: lib.features.slice(500 * 27, 501 * 27).map((x) => (Number.isNaN(x) ? 0 : x + 0.1)), ws: 2 };
    const best = rerank(lib, q);
    const cands = blockFrames(lib, topK(blockMins(lib, kernelQuery(lib, q)), 4));
    expect(rerank(lib, q, cands).k).toBe(best.k);
    expect(kernelScore(lib, kernelQuery(lib, q), best.k)).toBeLessThan(2 ** 31 - 1);
    // and perform's search through the backend finds the same frame
    lib.search = kernelSearch(4);
    const raw = Float32Array.from(q.z, (z, d) => z / lib.scale[d] + lib.mean[d]);
    expect(nearest(lib, raw, undefined, undefined, 2).k).toBe(best.k);
    lib.search = undefined;
    expect(BLOCK).toBe(64);
  });
  test("topKFast is topK", () => {
    const s = Int32Array.from({ length: 5000 }, (_, i) => (Math.imul(i, 2654435761) >>> 12) % 100000);
    expect(Array.from(topKFast(s, 50))).toEqual(topK(s, 50));
  });
});
