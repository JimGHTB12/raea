// The defaults and override test (docs/control.feature FC0c): render sdk/raea/bench/bevy-parity/defaults/parts.tsx
// (each Bevy-named part with no props) with the parity probe dumping its components, and hold every value against
// Bevy's own Default from the type registry (raea-bevy-sdk --schema). Each difference is a convention: a unit, a
// setting or a value raea chose that an author didn't write.
//
//   bun tools/bevy-defaults.ts [SCHEMA.json]     (default: makes it with target/release/raea-bevy-sdk)
// Prints one line a difference and the count; writes docs/control/defaults.tsv.
import { mkdirSync, readFileSync, writeFileSync } from "node:fs";
import { join } from "node:path";

const root = join(import.meta.dir, "..");
const TYPES = [
  "bevy_camera::camera::Camera",
  "bevy_camera::projection::Projection",
  "bevy_render::view::Tonemapping",
  "bevy_render::view::Msaa",
  "bevy_light::ambient_light::AmbientLight",
  "bevy_light::directional_light::DirectionalLight",
  "bevy_light::point_light::PointLight",
  "bevy_light::spot_light::SpotLight",
  "bevy_light::ambient_light::GlobalAmbientLight",
  "bevy_camera::clear_color::ClearColor",
  "bevy_camera::camera::Exposure",
  "bevy_render::view::ColorGrading",
];
// what the World works out itself, not a value an author or raea sets
const COMPUTED = ["computed", "near_clip_plane", "aspect_ratio"];
const tmp = join(process.env.TMPDIR ?? "/tmp", "raea-bevy-defaults");
mkdirSync(tmp, { recursive: true });

let schemaPath = process.argv[2];
if (!schemaPath) {
  schemaPath = join(tmp, "schema.json");
  const args = TYPES.flatMap((t) => ["--type", t]);
  const r = Bun.spawnSync([join(root, "target/release/raea-bevy-sdk"), ...args, "--schema", schemaPath, "-o", join(tmp, "schema.ts")], { stderr: "pipe" });
  if (r.exitCode !== 0) throw new Error(`raea-bevy-sdk: ${r.stderr}`);
}
const schema = JSON.parse(readFileSync(schemaPath, "utf8")).types as Record<string, { default?: unknown }>;

const out = join(tmp, "probe.jsonl");
const r = Bun.spawnSync([join(root, "target/release/raea"), "render", join(root, "sdk/raea/bench/bevy-parity/defaults/parts.tsx"), "--frame", "0", "-o", join(tmp, "parts.png")], {
  env: { ...process.env, RAEA_SIGN: "0", RAEA_MARK: "0", RAEA_PARITY: out, RAEA_PARITY_TYPES: TYPES.join(",") },
  stdout: "pipe",
  stderr: "pipe",
});
if (r.exitCode !== 0) throw new Error(`raea render failed: ${r.stderr.toString().split("\n").filter((l) => /error|refused/i.test(l)).slice(-2).join(" ")}`);
const lines = readFileSync(join(tmp, "probe.world.jsonl"), "utf8").trim().split("\n");
const world = JSON.parse(lines[lines.length - 1]).world as Record<string, unknown[]>;

// a colour as linear RGBA, so Srgba white and LinearRgba white are one value
const lin = (c: number) => (c <= 0.04045 ? c / 12.92 : ((c + 0.055) / 1.055) ** 2.4);
function norm(v: unknown): unknown {
  if (Array.isArray(v)) return v.map(norm);
  if (!v || typeof v !== "object") return v;
  const o = v as Record<string, unknown>;
  const k = Object.keys(o);
  if (k.length === 1 && k[0] === "Srgba") {
    const s = o.Srgba as Record<string, number>;
    return { LinearRgba: { red: lin(s.red), green: lin(s.green), blue: lin(s.blue), alpha: s.alpha } };
  }
  return Object.fromEntries(k.map((key) => [key, norm(o[key])]));
}
const near = (a: unknown, b: unknown) => typeof a === "number" && typeof b === "number" && Math.abs(a - b) <= 1e-4 * Math.max(1, Math.abs(b));
function diff(path: string, got: unknown, want: unknown, into: string[][]) {
  if (COMPUTED.some((c) => path.endsWith(`.${c}`))) return;
  if (got === want || near(got, want)) return;
  if (got && want && typeof got === "object" && typeof want === "object" && !Array.isArray(got) === !Array.isArray(want)) {
    const keys = new Set([...Object.keys(got as object), ...Object.keys(want as object)]);
    // an enum's variant changed: report the variant, not its fields
    if (!Array.isArray(want) && Object.keys(want as object).length === 1 && Object.keys(got as object).length === 1 && Object.keys(got as object)[0] !== Object.keys(want as object)[0]) {
      into.push([path, JSON.stringify(got), JSON.stringify(want)]);
      return;
    }
    for (const k of keys) diff(`${path}.${k}`, (got as Record<string, unknown>)[k], (want as Record<string, unknown>)[k], into);
    return;
  }
  into.push([path, JSON.stringify(got), JSON.stringify(want)]);
}

const rows: string[][] = [];
for (const t of TYPES) {
  const short = t.split("::").pop()!;
  const values = world[t];
  if (!Array.isArray(values)) {
    rows.push([short, "(the probe)", JSON.stringify(values), "-"]);
    continue;
  }
  if (values.length === 0) {
    rows.push([short, "(missing)", "no entity has it", "the part should give Bevy's"]);
    continue;
  }
  values.forEach((v, i) => {
    const d: string[][] = [];
    diff(values.length > 1 ? `${short}[${i}]` : short, norm(v), norm(schema[t]?.default), d);
    rows.push(...d.map(([p, g, w]) => [short, p, g, w]));
  });
}
mkdirSync(join(root, "docs/control"), { recursive: true });
writeFileSync(join(root, "docs/control/defaults.tsv"), ["# type\tpath\traea gives\tBevy's Default (tools/bevy-defaults.ts, docs/control.feature FC0c)", ...rows.map((r) => r.join("\t"))].join("\n") + "\n");
for (const [, p, g, w] of rows) console.log(`${p}: raea ${g}, Bevy ${w}`);
console.log(`${rows.length} differences from Bevy's defaults`);
