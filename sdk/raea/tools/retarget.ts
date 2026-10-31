// Motion capture retargeted onto a character (raea/clip): each BVH (a whole file, or a stretch of it) onto the
// skeleton character.py wrote, all into one clip file the composition imports.
//
//   bun sdk/raea/tools/retarget.ts SKELETON.json OUT.json [--map MAP.json] NAME=FILE.bvh[@FROM-TO] ...
//
// --map: a BVH whose joints aren't named as the skeleton's (100STYLE's), as { skeleton's joint: BVH's joint }; see
// tools/maps/.
// --carriage NAME: every clip's neck and head move about NAME's average (one actor's neutral walk) rather than their
// own, so a style's own carriage stays.
import { readFileSync, writeFileSync } from "node:fs";
import { readBvh, retarget, writeClips, type Clip } from "raea/clip";
import type { Skeleton } from "raea/character";

const args = process.argv.slice(2);
const mi = args.indexOf("--map");
const map = mi >= 0 ? (JSON.parse(readFileSync(args[mi + 1], "utf8")) as Record<string, string>) : undefined;
if (mi >= 0) args.splice(mi, 2);
const ci = args.indexOf("--carriage");
const carriageFrom = ci >= 0 ? args[ci + 1] : undefined;
if (ci >= 0) args.splice(ci, 2);
const [skelPath, out, ...specs] = args;
if (!skelPath || !out || specs.length === 0) {
  console.error("usage: bun sdk/raea/tools/retarget.ts SKELETON.json OUT.json [--map MAP.json] NAME=FILE.bvh[@FROM-TO] ...");
  process.exit(2);
}
const skel = JSON.parse(readFileSync(skelPath, "utf8")) as Skeleton;
const one = (s: string, carriage?: Clip["carriage"]) => {
  const m = /^([^=]+)=([^@]+)(?:@([\d.]+)-([\d.]+))?$/.exec(s);
  if (!m) throw new Error(`retarget: ${s} isn't NAME=FILE.bvh[@FROM-TO]`);
  const b = readBvh(readFileSync(m[2], "utf8"));
  const c = retarget(skel, b, { name: m[1], from: m[3] ? +m[3] : undefined, to: m[4] ? +m[4] : undefined, map, carriage });
  console.log(`${c.name}: ${c.frames} frames at ${c.fps} fps from ${m[2]}`);
  return c;
};
const ref = carriageFrom ? specs.find((s) => s.startsWith(`${carriageFrom}=`)) : undefined;
if (carriageFrom && !ref) throw new Error(`retarget: --carriage ${carriageFrom} isn't one of the clips`);
const carriage = ref ? one(ref).carriage : undefined;
const clips = specs.map((s) => one(s, carriage));
const file = writeClips(skel, clips);
writeFileSync(out, JSON.stringify(file));
console.log(`${out}: ${clips.length} clips, ${(JSON.stringify(file).length / 1024).toFixed(0)} KB`);
