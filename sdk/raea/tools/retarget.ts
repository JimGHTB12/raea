// Motion capture retargeted onto a character (raea/clip): each BVH (a whole file, or a stretch of it) onto the
// skeleton projects/psx/export.py wrote (game_engine joints), all into one clip file the composition imports.
//
//   bun sdk/raea/tools/retarget.ts SKELETON.json OUT.json [--map MAP.json] NAME=FILE.bvh[@FROM-TO] ...
//
// --map: the capture's source map, { game_engine joint: BVH joint } (tools/maps/100style.json, cmu.json).
import { readFileSync, writeFileSync } from "node:fs";
import { readBvh, retarget, writeClips } from "raea/clip";
import type { Skeleton } from "raea/character";

const args = process.argv.slice(2);
const mi = args.indexOf("--map");
const map = mi >= 0 ? (JSON.parse(readFileSync(args[mi + 1], "utf8")) as Record<string, string>) : undefined;
if (mi >= 0) args.splice(mi, 2);
if (ci >= 0) args.splice(ci, 2);
const [skelPath, out, ...specs] = args;
if (!skelPath || !out || specs.length === 0) {
  console.error("usage: bun sdk/raea/tools/retarget.ts SKELETON.json OUT.json [--map MAP.json] NAME=FILE.bvh[@FROM-TO] ...");
  process.exit(2);
}
const skel = JSON.parse(readFileSync(skelPath, "utf8")) as Skeleton;
const one = (s: string) => {
  const m = /^([^=]+)=([^@]+)(?:@([\d.]+)-([\d.]+))?$/.exec(s);
  if (!m) throw new Error(`retarget: ${s} isn't NAME=FILE.bvh[@FROM-TO]`);
  const b = readBvh(readFileSync(m[2], "utf8"));
  const c = retarget(skel, b, { name: m[1], from: m[3] ? +m[3] : undefined, to: m[4] ? +m[4] : undefined, map });
  console.log(`${c.name}: ${c.frames} frames at ${c.fps} fps from ${m[2]}`);
  return c;
};
const clips = specs.map((s) => one(s));
const file = writeClips(skel, clips);
writeFileSync(out, JSON.stringify(file));
console.log(`${out}: ${clips.length} clips, ${(JSON.stringify(file).length / 1024).toFixed(0)} KB`);
