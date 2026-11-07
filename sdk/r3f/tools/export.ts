// raea → React Three Fiber source (docs/r3f.feature R3F9): a composition baked frame by frame and written as an R3F
// module, and raea-effects.ts beside it when the scene has frame effects or Tony.
// `bun tools/export.ts EDIT.tsx COMPOSITION OUT.tsx [--from N] [--frames N | --all]`; a still scene by default (one
// frame), --all for the composition's whole length.
import { writeFileSync } from "node:fs";
import { dirname, resolve } from "node:path";
import { getCompositions } from "raea";
import { toR3FSource } from "../src/export";

const args = process.argv.slice(2);
const flag = (k: string) => {
  const i = args.indexOf(k);
  return i >= 0 ? args.splice(i, 2)[1] : undefined;
};
const all = args.includes("--all") && !!args.splice(args.indexOf("--all"), 1);
const from = Number(flag("--from") ?? 0), framesArg = flag("--frames");
const [edit, id, out] = args;
if (!edit || !id || !out) throw new Error("usage: bun tools/export.ts EDIT.tsx COMPOSITION OUT.tsx [--from N] [--frames N | --all]");
await import(resolve(edit));
const comp = getCompositions().find((c) => c.id === id);
if (!comp) throw new Error(`export: no composition ${id} in ${edit} (it has ${getCompositions().map((c) => c.id).join(", ")})`);
const frames = all ? comp.durationInFrames - from : Number(framesArg ?? 1);
const name = id.replace(/(^|[^A-Za-z0-9])([a-z0-9])/g, (_m, _a, c: string) => c.toUpperCase()).replace(/^[0-9]/, "S$&");
const { source: src, effects } = await toR3FSource(comp.component as never, { width: comp.width, height: comp.height, fps: comp.fps, from, frames, props: comp.defaultProps as Record<string, unknown> | undefined, name, title: `${id} from ${edit}` });
writeFileSync(out, src);
if (effects) writeFileSync(resolve(dirname(out), "raea-effects.ts"), effects);
console.log(`${out}: ${frames} frame${frames > 1 ? "s" : ""} of ${id}, ${(src.length / 1024).toFixed(0)} KB`);
