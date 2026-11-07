// What a live bundle's own JS allocates a frame (R1's JS layer, lightspeed Z3), without Bevy or a page: the bundle run
// in bun's JavaScriptCore with the bridge stubbed, frames handed to it as raea hands them, and the garbage
// counted from JSC's GC log (BUN_JSC_logGC, as jsheap.sh reads JSC_logGC) between the first and last measured frame.
// Quick and repeatable enough to split a frame's garbage by taking parts of an edit away.
//   bun bench/lightspeed/jsalloc.ts BUNDLE.js [--frames 3000] [--warm 240] [--from 600] [--live] [--span N] [--set NAME=JSON]
// BUNDLE is a lite bundle as raea bundles it (RAEA_BUNDLE_CACHE=DIR keeps one in DIR).
const args = process.argv.slice(2);
const opt = (k: string, d: number) => {
  const i = args.indexOf(k);
  return i >= 0 ? Number(args[i + 1]) : d;
};
const file = args[0];
const FRAMES = opt("--frames", 3000), WARM = opt("--warm", 240), FROM = opt("--from", 600);
const LIVE = args.includes("--live");
if (!file) {
  console.error("bun jsalloc.ts BUNDLE.js [--frames N] [--warm N] [--from F]");
  process.exit(2);
}

if (!args.includes("--child")) {
  // the run, with JSC's GC log on: what each eden collection freed, between the marks
  const p = Bun.spawnSync(["bun", import.meta.path, ...args, "--child"], { env: { ...process.env, BUN_JSC_logGC: "1" }, stderr: "pipe", stdout: "pipe" });
  const log = p.stderr.toString() + p.stdout.toString();
  const from = log.indexOf("jsalloc: measuring"), to = log.indexOf("jsalloc: done");
  if (from < 0 || to < 0) {
    console.error(log.split("\n").filter((l) => !l.startsWith("[GC")).slice(-12).join("\n"));
    process.exit(1);
  }
  const gc = /START \w (\d+)kb => \w+,.*?=> (\d+)kb, p=([0-9.]+)ms/g;
  let freed = 0, n = 0;
  for (const m of log.slice(from, to).matchAll(gc)) {
    freed += Number(m[1]) - Number(m[2]);
    n++;
  }
  const bridge = /bridge ([0-9.]+) KB/.exec(log.slice(to))?.[1];
  console.log(`${file.split("/").pop()}: ${(freed / FRAMES).toFixed(1)} KB a frame (${n} collections over ${FRAMES} frames); bridge ${bridge} KB a frame`);
  process.exit(0);
}

let wake: ((m: any) => void) | null = null;
let flushed = 0;
(globalThis as any).__bevyHost = {
  op_flush(batch: string) {
    flushed += batch.length;
  },
  op_next_event: () => new Promise((r) => (wake = r)),
  op_emit() {},
  op_request: () => Promise.resolve(null),
  op_animate() {},
};
// --set NAME=JSON: a global set before the bundle runs (globalThis.__raea_lite_bailout=false, say)
for (let i = 0; i < args.length; i++) if (args[i] === "--set") {
  const [k, v] = args[i + 1].split("=");
  (globalThis as any)[k] = JSON.parse(v);
}
(0, eval)(await Bun.file(file).text());

const tick = () => new Promise<void>((r) => setImmediate(r));
async function frame(f: number) {
  await tick();
  const w = wake;
  wake = null;
  if (!w) throw new Error("the bundle isn't waiting for an event");
  // --live: as live play sends a frame (live.rs step_inner), with this moment's input; else a timeline's frame number
  w({ t: "event", name: "raea-frame", value: LIVE ? { frame: f, input: { keys: [] } } : f });
  await tick();
}

for (let i = 0; i < WARM; i++) await frame(FROM + i);
Bun.gc(true);
console.error("jsalloc: measuring");
flushed = 0;
// --span N: the frames FROM .. FROM+N played over and over (live play's first seconds, measured long enough to count)
const SPAN = opt("--span", 0);
for (let i = 0; i < FRAMES; i++) await frame(SPAN > 0 ? FROM + ((WARM + i) % SPAN) : FROM + WARM + i);
console.error("jsalloc: done");
console.error(`bridge ${(flushed / FRAMES / 1024).toFixed(2)} KB a frame`);
process.exit(0);
