// The motion database read from disk under bun (benches, tools, tests); src/clips.ts itself reads nothing, so it
// bundles into raea's V8, where the bytes and clip files come through native calls.
//
//   const lib = dbLibraryAt(skel, `${HOME}/.cache/raea-mocap/lib/kid`);
import { readFileSync } from "node:fs";
import type { Skeleton } from "raea/character";
import type { ClipFile } from "./clip";
import { dbLibrary, parseDb, type DbLibrary, type DbLibraryOptions, type MotionDb } from "./clips";

/** A database read in one go: its typed arrays view the file's bytes, no JSON. */
export function readDb(path: string): MotionDb {
  const b = readFileSync(path);
  // (aligned for the typed arrays: a Buffer may sit at any offset in its pool)
  const u = new Uint8Array(new ArrayBuffer(b.byteLength));
  u.set(b);
  return parseDb(u);
}

/** A library's database (DIR/motion.db, or `db`) as a matchable library, its clip files read from DIR when they play */
export function dbLibraryAt(skel: Skeleton, dir: string, o: Omit<DbLibraryOptions, "read"> & { db?: string } = {}): DbLibrary {
  return dbLibrary(skel, readDb(o.db ?? `${dir}/motion.db`), { ...o, read: (file) => JSON.parse(readFileSync(`${dir}/${file}`, "utf8")) as ClipFile });
}
