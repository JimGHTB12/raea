// A TSX entry bundled for raea-runtime-js: `bun build.ts ENTRY OUT.js UI_SRC FALLBACK_MODULES` makes one IIFE of the entry,
// @raea/ui (resolved to UI_SRC, sdk/ui/src) and React. Bare imports resolve from the entry's own node_modules first,
// then FALLBACK_MODULES (raea's sdk/react/js/node_modules), so an entry folder needs no install of its own.
import { dirname, join } from "node:path";

const [entry, out, ui, fallback] = process.argv.slice(2);
if (!entry || !out || !ui) {
  console.error("usage: bun build.ts ENTRY OUT.js UI_SRC [FALLBACK_MODULES]");
  process.exit(2);
}
const b = await Bun.build({
  entrypoints: [entry],
  target: "browser",
  format: "iife",
  minify: { syntax: true },
  define: { "process.env.NODE_ENV": '"production"' },
  plugins: [
    {
      name: "raea-runtime-js-modules",
      setup(b) {
        // @raea/ui's subpaths (./react, ./wire, ./platform ...) from UI_SRC
        b.onResolve({ filter: /^@raea\/ui(\/.*)?$/ }, (a) => ({ path: join(ui, (a.path.slice("@raea/ui/".length) || "index") + ".ts") }));
        // React and friends: the entry's own, else the fallback's
        b.onResolve({ filter: /^(react|react-dom|react-reconciler|scheduler)(\/.*)?$/ }, (a) => {
          try {
            return { path: Bun.resolveSync(a.path, dirname(a.importer || entry)) };
          } catch {
            if (!fallback) throw new Error(`${a.path}: not installed beside ${entry}`);
            return { path: Bun.resolveSync(a.path, fallback + "/") };
          }
        });
      },
    },
  ],
});
if (!b.success) {
  console.error(b.logs.map((l) => String(l)).join("\n"));
  process.exit(1);
}
await Bun.write(out, await b.outputs[0].text());
