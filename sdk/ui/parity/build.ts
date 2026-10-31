// The corpus's bundles: `bun build.ts OUT [FILTER]` bundles every case under cases/ (or those whose path contains
// FILTER) twice, for React DOM in Chrome (OUT/<area>/<case>/chrome.js beside index.html) and for @raea/ui's React
// adapter on raea-page-host (bevy.js), and its <iframe> page if it has one (frameOf); builds Tailwind once over all of them
// (OUT/tw.css, with the font Bevy draws text in as the page's every family), and writes OUT/manifest.json: each case's
// area, steps, `proves:` ids and whether it has a frame page. The Rust runner (src/main.rs) runs this before anything
// else.
import { Glob } from "bun";
import { join, relative, dirname } from "node:path";
import { existsSync, mkdirSync, readFileSync, writeFileSync } from "node:fs";

const here = import.meta.dir;
const repo = join(here, "../../..");
const mods = join(here, "node_modules");
const ui = join(here, "../src");
const out = process.argv[2];
const filter = process.argv[3] ?? "";
if (!out) {
  console.error("usage: bun build.ts OUT [FILTER]");
  process.exit(2);
}
mkdirSync(out, { recursive: true });

// Tailwind v4, the cases as its sources; every family is the face Bevy's text uses (bevy_text's default font)
const font = readFileSync(join(here, "fonts/FiraMono-subset.ttf")).toString("base64");
// the same face under the same name on Bevy's side (probe/entry-bevy.tsx): a case that names "Parity Mono" finds it
// there too, as Chrome's page does, rather than falling through its list to a system face
const face = `@font-face { font-family: "Parity Mono"; src: url(data:font/ttf;base64,${font}) format("truetype"); }`;
// both sides draw no caret: headless Chrome's blinks in real time (on or off at a screenshot by chance), raea-page-host's
// is steady, so the pixel compares leave it out; where it is stays checked by focus and value (src/main.rs, "pixels")
const hideCaret = ":root { caret-color: transparent; }";
const twIn = join(here, "out-tw.in.css");
writeFileSync(
  twIn,
  `@import "tailwindcss";
@source "./cases";
@font-face { font-family: "Parity Mono"; src: url(data:font/ttf;base64,${font}) format("truetype"); }
@theme { --font-sans: "Parity Mono"; --font-serif: "Parity Mono"; --font-mono: "Parity Mono"; }
${hideCaret}
`,
);
const tw = Bun.spawnSync([join(mods, ".bin/tailwindcss"), "-i", twIn, "-o", join(out, "tw.css")], { cwd: here, stderr: "pipe" });
if (tw.exitCode !== 0) {
  console.error(tw.stderr.toString());
  process.exit(1);
}

const react = (a: { path: string }) => ({ path: Bun.resolveSync(a.path, mods + "/") });
async function bundle(entry: string, file: string, side: "chrome" | "bevy", dest: string) {
  const b = await Bun.build({
    entrypoints: [entry],
    target: "browser",
    format: "iife",
    minify: { syntax: true },
    define: { "process.env.NODE_ENV": '"production"', PARITY_FACE: JSON.stringify(`${face}\n${hideCaret}`) },
    plugins: [
      {
        name: "parity",
        setup(b) {
          b.onResolve({ filter: /^CASE$/ }, () => ({ path: file }));
          b.onResolve({ filter: /^parity$/ }, () => ({ path: join(here, `probe/${side}.ts`) }));
          if (side === "bevy") {
            b.onResolve({ filter: /^react-dom(\/client)?$/ }, () => ({ path: join(here, "probe/react-dom-bevy.ts") }));
            b.onResolve({ filter: /^@raea\/ui(\/.*)?$/ }, (a) => ({ path: join(ui, (a.path.slice("@raea/ui/".length) || "index") + ".ts") }));
          }
          b.onResolve({ filter: /^(react|react-dom|react-reconciler|scheduler)(\/.*)?$/ }, react);
        },
      },
    ],
  });
  if (!b.success) throw new Error(b.logs.map((l) => String(l)).join("\n"));
  await Bun.write(dest, await b.outputs[0].text());
}

// a case's <iframe> page, when it has one (CASE.frame.ts beside CASE.tsx, plain DOM): frame.html beside index.html
// for Chrome, where the case's src="frame.html" finds it, and frame.html and frame.js under bevy-root/ for raea-page-host,
// which serves it as RAEA_PAGE_ROOT (src/main.rs), with @raea/ui's DOM installed before the frame's script
async function frameOf(file: string, dir: string): Promise<boolean> {
  const src = file.replace(/\.tsx$/, ".frame.ts");
  if (!existsSync(src)) return false;
  const page = (script: string) => `<!doctype html><html><head><meta charset="utf-8"><style>${face}\n${hideCaret}</style></head><body>${script}</body></html>`;
  const root = join(dir, "bevy-root");
  mkdirSync(root, { recursive: true });
  const entry = join(dir, "frame-bevy-entry.ts");
  writeFileSync(entry, `import "@raea/ui/dom-globals";\nimport ${JSON.stringify(src)};\n`);
  for (const [from, to, bevy] of [[src, dir, false], [entry, root, true]] as const) {
    const b = await Bun.build({
      entrypoints: [from],
      target: "browser",
      format: "iife",
      minify: { syntax: true },
      plugins: bevy ? [{ name: "parity-frame", setup(b) { b.onResolve({ filter: /^@raea\/ui(\/.*)?$/ }, (a) => ({ path: join(ui, (a.path.slice("@raea/ui/".length) || "index") + ".ts") })); } }] : [],
    });
    if (!b.success) throw new Error(b.logs.map((l) => String(l)).join("\n"));
    const js = await b.outputs[0].text();
    // Chrome's frame has its script inline: sandboxed, its origin is opaque, and an opaque origin loads no file: URL
    if (bevy) await Bun.write(join(to, "frame.js"), js);
    writeFileSync(join(to, "frame.html"), page(bevy ? `<script src="frame.js"></script>` : `<script>${js.replace(/<\//g, "<\\/")}</script>`));
  }
  return true;
}

// a case's steps: the case bundled for bun with `parity` as a probe that does nothing, then imported (nothing renders)
async function stepsOf(file: string, dir: string): Promise<unknown[]> {
  const entry = join(dir, "steps-entry.ts");
  writeFileSync(entry, `export { steps } from ${JSON.stringify(file)};\n`);
  const b = await Bun.build({
    entrypoints: [entry],
    target: "bun",
    format: "esm",
    plugins: [
      {
        name: "parity-steps",
        setup(b) {
          b.onResolve({ filter: /^parity$/ }, () => ({ path: join(here, "probe/steps.ts") }));
          b.onResolve({ filter: /^(react|react-dom|react-reconciler|scheduler)(\/.*)?$/ }, react);
        },
      },
    ],
  });
  if (!b.success) throw new Error(b.logs.map((l) => String(l)).join("\n"));
  const mod = join(dir, "steps.mjs");
  await Bun.write(mod, await b.outputs[0].text());
  return (await import(mod)).steps ?? [];
}

// the checklist's ids (docs/ts-native/checklist.json), when it's there: a `proves:` id it doesn't have is a typo
let known: Set<string> | undefined;
try {
  known = new Set((JSON.parse(readFileSync(join(repo, "docs/ts-native/checklist.json"), "utf8")).items as { id: string }[]).map((i) => i.id));
} catch {}
const unknown: string[] = [];

const manifest: unknown[] = [];
const files = [...new Glob("cases/**/*.tsx").scanSync(here)].sort().filter((f) => f.includes(filter));
for (const f of files) {
  const file = join(here, f);
  const src = readFileSync(file, "utf8");
  const proves = [...src.matchAll(/^\s*\/\/\s*proves:(.*)$/gm)].flatMap((m) => m[1].split(/[\s,]+/).filter(Boolean));
  const rel = relative(join(here, "cases"), file).replace(/\.tsx$/, "");
  for (const id of proves) if (known && !known.has(id)) unknown.push(`${rel}: ${id}`);
  const area = rel.split("/")[0].split("-")[0].toUpperCase();
  const dir = join(out, rel);
  mkdirSync(dir, { recursive: true });
  const entry = { case: relative(repo, file), name: rel, area, proves, steps: [] as unknown[], frame: false, error: undefined as string | undefined };
  try {
    entry.steps = await stepsOf(file, dir);
    await bundle(join(here, "probe/entry-chrome.tsx"), file, "chrome", join(dir, "chrome.js"));
    await bundle(join(here, "probe/entry-bevy.tsx"), file, "bevy", join(dir, "bevy.js"));
    writeFileSync(join(dir, "index.html"), readFileSync(join(here, "probe/page.html")));
    entry.frame = await frameOf(file, dir);
  } catch (e) {
    entry.error = String((e as Error)?.message ?? e);
  }
  manifest.push(entry);
}
writeFileSync(join(out, "manifest.json"), JSON.stringify(manifest, null, 1));
console.log(`built ${files.length} cases into ${out}`);
if (unknown.length) {
  console.error(`proves: ids not in docs/ts-native/checklist.json:\n  ${unknown.join("\n  ")}`);
  process.exit(1);
}
// done: a case's module, imported for its steps, may have left something that keeps bun alive (a top-level
// listener, a timer), and the runner waits for this to exit
process.exit(0);
