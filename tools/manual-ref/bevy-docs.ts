// Bevy's own doc comments for the types the SDK names with @bevy, or whose whole doc is a Rust path (the types
// raea-bevy-sdk generates from raea's crates: `/** raea_core_animate::spec::AnimKey */`) (docs/manual.feature DM3): each type's doc and its
// fields' and variants' docs, read from the Rust source of Bevy 0.20 (the cargo registry) and of raea's own crates,
// kept in tools/manual-ref/bevy-docs.json so the SDK reference can be generated without the registry. sdk.ts gives a
// prop Bevy's field doc, marked as Bevy's, where the SDK's own comment is missing.
//   bun tools/manual-ref/bevy-docs.ts      rewrite tools/manual-ref/bevy-docs.json
import { readdirSync, readFileSync, writeFileSync, existsSync, statSync } from "node:fs"
import path from "node:path"
import os from "node:os"

const ROOT = path.resolve(import.meta.dir, "../..")
const REG = path.join(os.homedir(), ".cargo/registry/src")
const OUT = path.join(import.meta.dir, "bevy-docs.json")

function rsFiles(dir: string): string[] {
  if (!existsSync(dir)) return []
  return readdirSync(dir).flatMap((n) => {
    const p = path.join(dir, n)
    return statSync(p).isDirectory() ? rsFiles(p) : n.endsWith(".rs") ? [p] : []
  })
}

// a crate's source folder: Bevy's in the registry (0.20.0), raea's by its Cargo.toml name
const raeaCrates = new Map<string, string>()
function scanCrates(dir: string, depth = 0) {
  if (depth > 3 || /node_modules|target|\.git/.test(dir)) return
  for (const n of readdirSync(dir, { withFileTypes: true })) {
    const p = path.join(dir, n.name)
    if (n.isDirectory()) scanCrates(p, depth + 1)
    else if (n.name === "Cargo.toml") {
      const m = /^name\s*=\s*"([^"]+)"/m.exec(readFileSync(p, "utf8"))
      if (m) raeaCrates.set(m[1].replace(/-/g, "_"), path.join(dir, "src"))
    }
  }
}
for (const top of ["systems", "core", "runtime", "engine", "io"]) if (existsSync(path.join(ROOT, top))) scanCrates(path.join(ROOT, top))
function crateDir(krate: string): string | null {
  if (raeaCrates.has(krate)) return raeaCrates.get(krate)!
  if (!existsSync(REG)) return null
  for (const idx of readdirSync(REG)) {
    const d = path.join(REG, idx, `${krate}-0.20.0`, "src")
    if (existsSync(d)) return d
  }
  return null
}

// the /// lines above a line, joined, its attributes skipped
function docAbove(lines: string[], i: number): string {
  const out: string[] = []
  for (let k = i - 1; k >= 0; k--) {
    const t = lines[k].trim()
    if (t.startsWith("///")) { out.unshift(t.replace(/^\/\/\/\s?/, "")); continue }
    // above the docs, or code: stop; an attribute (with its continuation lines) or a blank line under it: go on
    if (out.length || /[;{}]$/.test(t) || t.startsWith("//")) break
  }
  // the first paragraph, without rustdoc's link brackets
  const para = out.join("\n").split(/\n\s*\n/)[0] ?? ""
  return para.replace(/\[`([^`\]]+)`\](\([^)]*\)|\[[^\]]*\])?/g, "`$1`").replace(/\[([^\]]+)\]\([^)]*\)/g, "$1").replace(/\s*\n\s*/g, " ").trim()
}

type Entry = { doc: string; fields: Record<string, string> }
function find(krate: string, name: string): Entry | null {
  const dir = crateDir(krate)
  if (!dir) return null
  for (const f of rsFiles(dir)) {
    const lines = readFileSync(f, "utf8").split("\n")
    const at = lines.findIndex((l) => new RegExp(`^\\s*pub (struct|enum) ${name}\\b`).test(l))
    if (at < 0) continue
    const entry: Entry = { doc: docAbove(lines, at), fields: {} }
    if (!lines[at].includes("{") && !lines[at + 1]?.trim().startsWith("{")) return entry
    // fields (pub name: T) and variants (Name, Name(..), Name { .. }) one level inside the braces
    let depth = 0
    for (let k = at; k < lines.length; k++) {
      const l = lines[k]
      const before = depth
      depth += (l.match(/\{/g) ?? []).length - (l.match(/\}/g) ?? []).length
      if (before === 1) {
        const field = /^\s*pub\s+([a-z_][a-z0-9_]*)\s*:/.exec(l)
        const variant = /^\s*([A-Z][A-Za-z0-9_]*)\s*(\(|\{|,|=|$)/.exec(l)
        const key = field?.[1] ?? variant?.[1]
        if (key) { const d = docAbove(lines, k); if (d) entry.fields[key] = d }
      }
      if (k > at && depth <= 0) break
    }
    return entry
  }
  return null
}

const tags = new Set<string>()
const walk = (d: string): void => {
  for (const n of readdirSync(d, { withFileTypes: true })) {
    const p = path.join(d, n.name)
    if (n.isDirectory()) { if (n.name !== "node_modules") walk(p) }
    else if (/\.tsx?$/.test(n.name)) for (const m of readFileSync(p, "utf8").matchAll(/(?:@bevy |\/\*\* )([a-z_0-9]+(?:::[A-Za-z_0-9]+)+)(?= |\*\/|$)/gm)) tags.add(m[1])
  }
}
for (const pkg of readdirSync(path.join(ROOT, "sdk"))) if (existsSync(path.join(ROOT, "sdk", pkg, "src"))) walk(path.join(ROOT, "sdk", pkg, "src"))

const docs: Record<string, Entry> = {}
let missing = 0
for (const t of [...tags].sort()) {
  const parts = t.split("::")
  const e = find(parts[0], parts[parts.length - 1])
  if (e && (e.doc || Object.keys(e.fields).length)) docs[t] = e
  else missing++
}
// an export with no @bevy tag that has a Bevy type's name (raea mirrors Bevy's names): that type, found in these crates
// and kept as "name:<Name>" with the crate it came from
const CRATES = ["bevy_shape", "bevy_math", "bevy_pbr", "bevy_light", "bevy_camera", "bevy_sprite", "bevy_sprite_render", "bevy_ui", "bevy_text", "bevy_core_pipeline", "bevy_post_process", "bevy_anti_alias", "bevy_transform", "bevy_color", "bevy_image", "bevy_mesh", "bevy_animation", "bevy_audio", "bevy_input", "bevy_picking", "bevy_gizmos", "bevy_time", "bevy_render"]
const scan = new Bun.Transpiler({ loader: "tsx" })
const names = new Set<string>()
for (const f of readdirSync(path.join(ROOT, "sdk/raea/src"))) if (/\.tsx?$/.test(f)) for (const n of scan.scan(readFileSync(path.join(ROOT, "sdk/raea/src", f), "utf8")).exports) if (/^[A-Z]/.test(n)) names.add(n)
let byName = 0
for (const n of [...names].sort()) {
  if ([...tags].some((t) => t.endsWith("::" + n))) continue
  for (const c of CRATES) {
    const e = find(c, n)
    if (e && (e.doc || Object.keys(e.fields).length)) { docs[`name:${n}`] = { ...e, doc: e.doc, from: `${c}::${n}` } as Entry; byName++; break }
  }
}
console.log(`bevy-docs: ${byName} more by name`)
writeFileSync(OUT, JSON.stringify(docs, null, 1) + "\n")
console.log(`bevy-docs: ${Object.keys(docs).length} types documented, ${missing} not found, in ${path.relative(ROOT, OUT)}`)
