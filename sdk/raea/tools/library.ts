// A motion library for a character: free motion capture retargeted onto its skeleton, one clip file per source
// file, with an index of what each clip is. Sources are read straight from their archives over the network (HTTP
// ranges into 100STYLE's 1.5 GB zip, CMU's per-clip zips), so nothing but the retargeted clips lands on disk. Clips
// already written are skipped, so a run picks up where the last one stopped.
//
//   bun sdk/raea/tools/library.ts 100style SKELETON.json OUT [--moves FW,ID,...] [--styles Neutral,Old,...]
//   bun sdk/raea/tools/library.ts cmu SKELETON.json OUT [--match REGEX] [--not REGEX] [--ids 127_03,64_27,...] [--max N]
//   bun sdk/raea/tools/library.ts accad SKELETON.json OUT [--match REGEX]   ACCAD's BVHs (unzipped in ACCAD_DIR)
//   bun sdk/raea/tools/library.ts index OUT
//   bun sdk/raea/tools/library.ts tables SKELETON.json OUT    each clip file's per-frame table (FILE.table.json)
//   bun sdk/raea/tools/library.ts effects SKELETON.json OUT   each 100STYLE style's rig effect against Neutral, from
//                                                                their forward walks, and CMU's (CMU_EFFECTS) against
//                                                                the same actor's own walk (OUT/effects.json)
//   bun sdk/raea/tools/library.ts styles SKELETON.json OUT    every clip's style, a second every half second
//                                                                (OUT/styles.json), and each clip's mean (OUT/means.json)
//   bun sdk/raea/tools/library.ts db SKELETON.json OUT [--max N] [--to FILE]
//                                                                every clip in the index as one motion database
//                                                                (OUT/motion.db: docs/clips.feature, src/clips.ts)
//
// 100STYLE (CC BY 4.0): 100 styles of one actor, each walked forward (FW), back (BW), sideways (SW), run (FR, BR,
// SR), stood (ID) and turned (TR1-3), cut by its Frame_Cuts.csv. CMU (free for any use; cgspeed's BVH conversion):
// 2,505 clips, chosen by their descriptions in ~/.cache/raea-mocap/cmu/index/all.tsv. ACCAD's Open Motion Project
// (CC BY 3.0, the Ohio State University's ACCAD): three actors' takes (Male1, Male2, Female1), among them picking up,
// lifting, carrying and putting down a box from a stand, a walk and a run; each actor's turns measured from its own
// A1_Stand (retarget's `rest`), since its files' zero pose isn't a person standing.
import { existsSync, mkdirSync, readdirSync, readFileSync, writeFileSync } from "node:fs";
import { inflateRawSync } from "node:zlib";
import { readBvh, readClips, retarget, retargetIK, writeClips, type ClipFile } from "raea/clip";
import { libraryTable } from "raea/matching";
import { buildDb, type BuildClip } from "@raea/character/clips";
import { learnEffect, type RigEffect } from "raea/effects";
import { STYLE_KEYS, styleByLabel, styleTable, type StyleRow } from "raea/style";
import type { Skeleton } from "raea/character";

const STYLE_ZIP = "https://zenodo.org/api/records/8127870/files/100STYLE.zip/content";
const STYLE_MAP = JSON.parse(readFileSync(new URL("./maps/100style.json", import.meta.url), "utf8")) as Record<string, string>;
const CMU_MAP = JSON.parse(readFileSync(new URL("./maps/cmu.json", import.meta.url), "utf8")) as Record<string, string>;
/** 100STYLE unpacked on disk (read from there when it is; from Zenodo's zip by HTTP ranges when not) */
const STYLE_DIR = process.env.STYLE_DIR ?? `${process.env.HOME}/.cache/raea-mocap/100style-full`;
const ACCAD_MAP = JSON.parse(readFileSync(new URL("./maps/accad.json", import.meta.url), "utf8")) as Record<string, string>;
const ACCAD_DIR = process.env.ACCAD_DIR ?? `${process.env.HOME}/.cache/raea-mocap/accad`;
const CMU_INDEX = `${process.env.HOME}/.cache/raea-mocap/cmu/index/all.tsv`;
const MOVES = ["BR", "BW", "FR", "FW", "ID", "SR", "SW", "TR1", "TR2", "TR3"];

/** a zip's members by name, read through `range` (byte ranges of a file `size` long) */
async function readZip(size: number, range: (a: number, b: number) => Promise<Uint8Array>) {
  const tail = await range(Math.max(0, size - 65558), size);
  const dv = new DataView(tail.buffer);
  let e = tail.length - 22;
  while (e >= 0 && dv.getUint32(e, true) !== 0x06054b50) e--;
  if (e < 0) throw new Error("remoteZip: no end of central directory");
  const count = dv.getUint16(e + 10, true), cdSize = dv.getUint32(e + 12, true), cdAt = dv.getUint32(e + 16, true);
  const cd = await range(cdAt, cdAt + cdSize), cv = new DataView(cd.buffer);
  const members = new Map<string, { method: number; csize: number; local: number }>();
  for (let p = 0, i = 0; i < count; i++) {
    const nl = cv.getUint16(p + 28, true), xl = cv.getUint16(p + 30, true), cl = cv.getUint16(p + 32, true);
    const name = new TextDecoder().decode(cd.subarray(p + 46, p + 46 + nl));
    members.set(name, { method: cv.getUint16(p + 10, true), csize: cv.getUint32(p + 20, true), local: cv.getUint32(p + 42, true) });
    p += 46 + nl + xl + cl;
  }
  return {
    names: [...members.keys()],
    async read(name: string): Promise<string> {
      const m = members.get(name);
      if (!m) throw new Error(`remoteZip: no ${name}`);
      const lh = await range(m.local, m.local + 30), lv = new DataView(lh.buffer);
      const start = m.local + 30 + lv.getUint16(26, true) + lv.getUint16(28, true);
      const data = await range(start, start + m.csize);
      return new TextDecoder().decode(m.method === 8 ? inflateRawSync(data) : data);
    },
  };
}

/** a remote zip, read through HTTP ranges (a large one: only its directory and the members asked for come down) */
async function remoteZip(url: string) {
  const head = await fetch(url, { method: "HEAD", redirect: "follow" });
  const size = +head.headers.get("content-length")!;
  const at = head.url || url;
  return readZip(size, async (a, b) => {
    for (let tries = 0; ; tries++) {
      try {
        const r = await fetch(at, { headers: { Range: `bytes=${a}-${b - 1}` } });
        if (r.status !== 206) throw new Error(`range ${a}-${b}: HTTP ${r.status}`);
        return new Uint8Array(await r.arrayBuffer());
      } catch (e) {
        if (tries >= 3) throw e;
      }
    }
  });
}

/** a small zip, fetched whole into memory */
async function wholeZip(url: string) {
  let bytes: Uint8Array | undefined;
  for (let tries = 0; !bytes; tries++) {
    try {
      const r = await fetch(url);
      if (!r.ok) throw new Error(`${url}: HTTP ${r.status}`);
      bytes = new Uint8Array(await r.arrayBuffer());
    } catch (e) {
      if (tries >= 3) throw e;
    }
  }
  return readZip(bytes.length, async (a, b) => bytes!.slice(a, b));
}

function flag(args: string[], name: string): string | undefined {
  const i = args.indexOf(name);
  return i >= 0 ? args[i + 1] : undefined;
}

async function style(skel: Skeleton, out: string, args: string[]) {
  const dir = `${out}/100style`;
  mkdirSync(dir, { recursive: true });
  const zip = existsSync(`${STYLE_DIR}/100STYLE/Frame_Cuts.csv`)
    ? { read: async (name: string) => readFileSync(`${STYLE_DIR}/${name}`, "utf8") }
    : await remoteZip(STYLE_ZIP);
  const cuts = (await zip.read("100STYLE/Frame_Cuts.csv")).trim().split("\n").map((l) => l.split(","));
  const head = cuts.shift()!;
  // (every move: 100STYLE is the locomotion, docs/skeleton.feature SK7)
  const moves = (flag(args, "--moves") ?? MOVES.join(",")).split(",");
  const only = flag(args, "--styles")?.split(",");
  // the actor's carriage, from their Neutral walk: every style's neck and head move about it, so a style's own
  // carriage (Old's hanging head) stays
  const nrow = cuts.find((r) => r[0] === "Neutral")!;
  const carriage = retarget(skel, readBvh(await zip.read("100STYLE/Neutral/Neutral_FW.bvh")), {
    name: "Neutral_FW", from: +nrow[head.indexOf("FW_START")] / 60, to: +nrow[head.indexOf("FW_STOP")] / 60, map: STYLE_MAP }).carriage;
  for (const row of cuts) {
    const name = row[0];
    if (only && !only.includes(name)) continue;
    const file = `${dir}/${name}.json`;
    const have = existsSync(file) ? (JSON.parse(readFileSync(file, "utf8")) as ClipFile) : undefined;
    const clips = have ? [...readClips(have).values()] : [];
    let added = 0;
    for (const mv of moves) {
      const a = row[head.indexOf(`${mv}_START`)], b = row[head.indexOf(`${mv}_STOP`)];
      if (!a || a === "N/A" || clips.some((c) => c.name === `${name}_${mv}`)) continue;
      const bvh = readBvh(await zip.read(`100STYLE/${name}/${name}_${mv}.bvh`));
      clips.push(retargetIK(skel, bvh, { name: `${name}_${mv}`, from: +a / 60, to: +b / 60, map: STYLE_MAP, carriage }));
      added++;
    }
    if (added) {
      writeFileSync(file, JSON.stringify(writeClips(skel, clips)));
      console.log(`${name}: ${clips.map((c) => `${c.name} ${c.frames}`).join(", ")}`);
    }
  }
}

async function cmu(skel: Skeleton, out: string, args: string[]) {
  const dir = `${out}/cmu`;
  mkdirSync(dir, { recursive: true });
  const match = new RegExp(flag(args, "--match") ?? ".", "i"), not = flag(args, "--not"), max = +(flag(args, "--max") ?? Infinity);
  const skip = not ? new RegExp(not, "i") : undefined, ids = flag(args, "--ids") ? new Set(flag(args, "--ids")!.split(",")) : undefined;
  let n = 0;
  for (const line of readFileSync(CMU_INDEX, "utf8").trim().split("\n")) {
    const f = line.split("|");
    const id = /\[([\d_]+)\]\(([^)]+)\)/.exec(f[1]);
    if (!id || !match.test(f[3]) || skip?.test(f[3]) || (ids && !ids.has(id[1]))) continue;
    if (n++ >= max) break;
    const file = `${dir}/${id[1]}.json`;
    if (existsSync(file)) continue;
    const zip = await wholeZip(id[2].replace("github.com/", "raw.githubusercontent.com/").replace("/raw/", "/"));
    const bvhName = zip.names.find((x) => x.endsWith(".bvh"));
    if (!bvhName) { console.log(`${id[1]}: no BVH in its zip`); continue; }
    const c = retargetIK(skel, readBvh(await zip.read(bvhName)), { name: `c${id[1]}`, map: CMU_MAP, unit: 0.056444 });
    writeFileSync(file, JSON.stringify(writeClips(skel, [c])));
    console.log(`c${id[1]}: ${c.frames} frames, ${f[3].trim()}`);
  }
}

/** ACCAD's takes: OUT/accad/<take>.json, clip `a<take>` (Male1_A5_PickUpBox: aMale1_A5_PickUpBox) */
async function accad(skel: Skeleton, out: string, args: string[]) {
  const dir = `${out}/accad`;
  mkdirSync(dir, { recursive: true });
  const match = new RegExp(flag(args, "--match") ?? ".", "i");
  for (const actor of ["Male1", "Male2", "Female1"]) {
    const src = `${ACCAD_DIR}/${actor}_bvh`;
    if (!existsSync(src)) { console.log(`${src}: not there (unzip ${actor}_bvh.zip from accad.osu.edu)`); continue; }
    const files = readdirSync(src).filter((x) => x.endsWith(".bvh")).sort();
    const stand = files.find((x) => /_A0?1_Stand\.bvh$/.test(x));
    if (!stand) throw new Error(`accad: ${actor} has no A1_Stand to measure from`);
    const standB = readBvh(readFileSync(`${src}/${stand}`, "utf8")), rest = standB.frames[0];
    const shape = (b: { joints: { name: string }[] }) => b.joints.map((j) => j.name).join(" ");
    for (const f of files.filter((x) => match.test(x))) {
      const take = f.replace(/\.bvh$/, ""), file = `${dir}/${take}.json`;
      if (existsSync(file)) continue;
      const b = readBvh(readFileSync(`${src}/${f}`, "utf8"));
      // (three of Male2's takes are another skeleton, with no stand of their own to measure from)
      if (shape(b) !== shape(standB)) { console.log(`a${take}: skipped, not ${stand}'s skeleton`); continue; }
      const c = retargetIK(skel, b, { name: `a${take}`, map: ACCAD_MAP, rest, unit: 0.01 });
      writeFileSync(file, JSON.stringify(writeClips(skel, [c])));
      console.log(`a${take}: ${c.frames} frames`);
    }
  }
}

/** what each clip is: its source, file, move, label and length */
function index(out: string) {
  const desc = new Map<string, string>();
  for (const line of readFileSync(CMU_INDEX, "utf8").trim().split("\n")) {
    const f = line.split("|"), id = /\[([\d_]+)\]/.exec(f[1]);
    if (id) desc.set(`c${id[1]}`, f[3].trim());
  }
  const rows: { name: string; source: string; file: string; label: string; move?: string; frames: number; fps: number }[] = [];
  for (const src of ["100style", "cmu", "accad"]) {
    if (!existsSync(`${out}/${src}`)) continue;
    for (const f of readdirSync(`${out}/${src}`).filter((x: string) => x.endsWith(".json") && !x.endsWith(".table.json")).sort()) {
      const file = JSON.parse(readFileSync(`${out}/${src}/${f}`, "utf8")) as ClipFile;
      for (const c of file.clips) {
        const [label, move] = src === "100style" ? c.name.split("_")
          : src === "accad" ? [c.name.slice(1).split("_").slice(2).join(" ").replace(/([a-z])([A-Z])/g, "$1 $2").toLowerCase(), undefined]
          : [desc.get(c.name) ?? "", undefined];
        rows.push({ name: c.name, source: src, file: `${src}/${f}`, label, move, frames: c.frames, fps: c.fps });
      }
    }
  }
  writeFileSync(`${out}/index.json`, JSON.stringify({ clips: rows }, null, 1));
  const frames = rows.reduce((a, r) => a + r.frames, 0);
  console.log(`${out}/index.json: ${rows.length} clips, ${(frames / 30 / 3600).toFixed(2)} h`);
}

function clipFiles(out: string): string[] {
  return ["100style", "cmu", "accad"].filter((s) => existsSync(`${out}/${s}`)).flatMap((s) =>
    readdirSync(`${out}/${s}`).filter((x) => x.endsWith(".json") && !x.endsWith(".table.json")).sort().map((x) => `${out}/${s}/${x}`));
}

function tables(skel: Skeleton, out: string) {
  for (const f of clipFiles(out)) {
    const t = f.replace(/\.json$/, ".table.json");
    if (existsSync(t)) continue;
    const clips = readClips(JSON.parse(readFileSync(f, "utf8")) as ClipFile);
    writeFileSync(t, JSON.stringify(libraryTable(skel, clips)));
    console.log(`${t}: ${[...clips.values()].reduce((a, c) => a + c.frames, 0)} frames`);
  }
}

function styles(skel: Skeleton, out: string) {
  const ix = JSON.parse(readFileSync(`${out}/index.json`, "utf8")) as { clips: { name: string; label: string; file: string }[] };
  const labels = Object.fromEntries(ix.clips.map((c) => [c.name, c.label]));
  const rows: StyleRow[] = [];
  for (const f of clipFiles(out)) {
    const t = styleTable(skel, readClips(JSON.parse(readFileSync(f, "utf8")) as ClipFile), { labels });
    rows.push(...t.rows);
  }
  const round = (r: StyleRow) => ({ ...r, v: Object.fromEntries(STYLE_KEYS.map((k) => [k, +r.v[k].toFixed(4)])) });
  writeFileSync(`${out}/styles.json`, JSON.stringify({ keys: STYLE_KEYS, rows: rows.map(round) }));
  // each label's mean, per move (100STYLE's Old_FW and Old_ID apart: a walk and a stand differ more than styles do)
  const byMove = rows.map((r) => ({ ...r, label: r.clip }));
  const means = styleByLabel({ rows: byMove, mean: {} as any, sd: {} as any });
  writeFileSync(`${out}/means.json`, JSON.stringify(Object.fromEntries([...means].map(([k, v]) => [k, { label: labels[k], v: Object.fromEntries(STYLE_KEYS.map((s) => [s, +v[s].toFixed(4)])) }])), null, 1));
  console.log(`${out}/styles.json: ${rows.length} windows; ${out}/means.json: ${means.size} clips`);
}

/** CMU's styled walks with the same actor's normal walk beside them, as effects by name: 100STYLE's Crouched is a bent
 * walk (its hips at 0.96 of the actor's Neutral walk's, in its own BVH), subject 136's crouched walk a real one (0.87) */
const CMU_EFFECTS: Record<string, [string, string]> = { CrouchWalk: ["136_20", "136_09"] };

function effects(skel: Skeleton, out: string) {
  const fw = (s: string) => readClips(JSON.parse(readFileSync(`${out}/100style/${s}.json`, "utf8")) as ClipFile).get(`${s}_FW`);
  const neutral = fw("Neutral")!, all: Record<string, RigEffect> = {};
  const r4 = (v: number[]) => v.map((x) => +x.toFixed(4));
  for (const f of readdirSync(`${out}/100style`).filter((x) => x.endsWith(".json") && !x.endsWith(".table.json")).sort()) {
    const s = f.slice(0, -5), c = fw(s);
    if (!c || s === "Neutral") continue;
    const e = learnEffect(skel, neutral, c);
    all[s] = { posture: e.posture.map(r4) as RigEffect["posture"], amplitude: e.amplitude.map(r4) as RigEffect["amplitude"], rate: +e.rate.toFixed(4), speed: +(e.speed ?? 1).toFixed(4) };
  }
  const cmu = (id: string) => readClips(JSON.parse(readFileSync(`${out}/cmu/${id}.json`, "utf8")) as ClipFile).get(`c${id}`)!;
  for (const [name, [walk, styled]] of Object.entries(CMU_EFFECTS)) {
    const e = learnEffect(skel, cmu(walk), cmu(styled));
    all[name] = { posture: e.posture.map(r4) as RigEffect["posture"], amplitude: e.amplitude.map(r4) as RigEffect["amplitude"], rate: +e.rate.toFixed(4), speed: +(e.speed ?? 1).toFixed(4) };
  }
  writeFileSync(`${out}/effects.json`, JSON.stringify(all));
  console.log(`${out}/effects.json: ${Object.keys(all).length} effects`);
}

function db(skel: Skeleton, out: string, args: string[]) {
  const ix = JSON.parse(readFileSync(`${out}/index.json`, "utf8")) as { clips: { name: string; source: string; file: string; label: string; move?: string }[] };
  const max = +(flag(args, "--max") ?? Infinity), to = flag(args, "--to") ?? `${out}/motion.db`;
  const rows = ix.clips.slice(0, max);
  // clip file by clip file, each read once
  function* clips(): Generator<BuildClip> {
    const byFile = new Map<string, typeof rows>();
    for (const r of rows) byFile.set(r.file, [...(byFile.get(r.file) ?? []), r]);
    for (const [file, rs] of byFile) {
      const cs = readClips(JSON.parse(readFileSync(`${out}/${file}`, "utf8")) as ClipFile);
      for (const r of rs) { const clip = cs.get(r.name); if (clip) yield { ...r, clip }; else console.log(`${r.name}: not in ${file}`); }
    }
  }
  const bytes = buildDb(skel, clips(), (s) => console.log(s));
  writeFileSync(to, bytes);
  console.log(`${to}: ${(bytes.length / 1e6).toFixed(1)} MB`);
}

const [cmd, ...rest] = process.argv.slice(2);
if (cmd === "index") index(rest[0]);
else if (cmd === "db") db(JSON.parse(readFileSync(rest[0], "utf8")) as Skeleton, rest[1], rest.slice(2));
else if (cmd === "tables" || cmd === "styles" || cmd === "effects") {
  const skel = JSON.parse(readFileSync(rest[0], "utf8")) as Skeleton;
  ({ tables, styles, effects })[cmd](skel, rest[1]);
}
else if (cmd === "100style" || cmd === "cmu" || cmd === "accad") {
  const [skelPath, out] = rest;
  const skel = JSON.parse(readFileSync(skelPath, "utf8")) as Skeleton;
  await ({ "100style": style, cmu, accad })[cmd](skel, out, rest.slice(2));
  index(out);
} else {
  console.error("usage: library.ts 100style|cmu|accad|tables|styles|effects SKELETON.json OUT [...] | library.ts index OUT");
  process.exit(2);
}
