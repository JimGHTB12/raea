// bun sdk/ui/test/html-run.ts CASE_DIR [t ...]: an HTML composition loaded by src/html.ts on the stand-in host
// (test/host.ts), html.read answered here as runtime/js's html_page.rs answers it (project files, and http(s)
// through raea's cache), then seeked to each t; prints what it reported and the host's tree at the last t.
import { Host } from "./host";
import { readdirSync, readFileSync, existsSync, statSync, mkdirSync, writeFileSync } from "node:fs";
import { join, relative } from "node:path";

const dir = process.argv[2];
const times = process.argv.slice(3).map(Number);
const g = globalThis as any;
const host = new Host();
const listing: string[] = [];
const walk = (d: string) => {
  for (const f of readdirSync(d)) {
    const p = join(d, f);
    if (statSync(p).isDirectory()) walk(p);
    else listing.push(relative(dir, p));
  }
};
walk(dir);
const CACHE = join(process.env.HOME!, "Library/Caches/raea");
const CHROME_UA = "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/131.0.0.0 Safari/537.36";
function fnv(s: string) {
  let h = 0xcbf29ce484222325n;
  for (const b of new TextEncoder().encode(s)) {
    h ^= BigInt(b);
    h = (h * 0x100000001b3n) & 0xffffffffffffffffn;
  }
  return h.toString(16).padStart(16, "0");
}
async function answer(args: { url: string; binary?: boolean }) {
  const u = args.url;
  if (/^https?:/.test(u)) {
    const google = new URL(u).host === "fonts.googleapis.com";
    const key = google ? `${u}#ua=${CHROME_UA.length}` : u;
    const p = join(CACHE, fnv(key));
    if (!existsSync(p) || statSync(p).size === 0) {
      mkdirSync(CACHE, { recursive: true });
      const r = await fetch(u, { headers: google ? { "user-agent": CHROME_UA } : {} });
      if (!r.ok) throw new Error(`${u}: ${r.status}`);
      const body = new Uint8Array(await r.arrayBuffer());
      if (body.length === 0) throw new Error(`${u}: an empty answer (not cached)`);
      writeFileSync(p, body);
    }
    return { path: p, text: args.binary ? undefined : readFileSync(p, "utf8") };
  }
  const p = join(dir, u);
  if (!existsSync(p)) throw new Error(`${u}: not found`);
  return { path: p, text: args.binary ? undefined : readFileSync(p, "utf8") };
}
g.__ui_call = (id: number, name: string, json: string) => {
  const args = JSON.parse(json);
  (name === "html.read" ? answer(args) : Promise.reject(new Error(`no call ${name}`))).then(
    (value) => g.__ui_deliver(JSON.stringify({ t: "answer", id, ok: true, value })),
    (e) => g.__ui_deliver(JSON.stringify({ t: "answer", id, ok: false, error: String(e?.message ?? e) })),
  );
};
const emitted: Record<string, (v: any) => void> = {};
g.__ui_emit = (name: string, json: string) => {
  host.emits.push([name, JSON.parse(json)]);
  emitted[name]?.(JSON.parse(json));
};
g.__ui_viewport = { w: 1920, h: 1080, dpr: 1 };
const fonts = join(import.meta.dir, "../../../core/fonts");
g.__htmlDebug = true;
g.__html = { source: readFileSync(join(dir, "index.html"), "utf8"), file: "index.html", dir, files: listing, fps: 30, fonts, fontFiles: readdirSync(fonts).filter((f) => f.endsWith(".woff2")) };
const ready = new Promise<any>((r) => (emitted["html.ready"] = r));
await import("../src/html");
const info = await ready;
console.log(JSON.stringify(info, null, 1).slice(0, 2000));
for (const t of times) {
  const done = new Promise<any>((r) => (emitted["html.seeked"] = r));
  g.__ui_deliver(JSON.stringify({ t: "named", name: "html.seek", value: { t } }));
  await done;
}
const show = (id: number, depth: number) => {
  const n = host.nodes.get(id)!;
  if (depth > 14) return;
  const st = n.props.style ? JSON.stringify(n.props.style) : "";
  console.log(`${"  ".repeat(depth)}${n.kind}#${id}${n.props.id ? ` id=${n.props.id}` : ""}${n.text ? ` "${String(n.text).slice(0, 40)}"` : ""} ${st.slice(0, 300)}`);
  for (const c of n.children) show(c, depth + 1);
};
show(0, 0);
if (process.env.FACES) for (const n of host.nodes.values()) if (n.kind === "style") for (const c of n.children) console.log(String(host.nodes.get(c)?.text).replace(/}/g, "}\n").slice(0, 8000));
if (process.env.RULES) for (const r of g.__htmlRules()) if (JSON.stringify(r).includes(process.env.RULES)) console.log(JSON.stringify(r).slice(0, 400));
