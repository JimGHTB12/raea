// Serves the parity page (test/parity/page.tsx) for tools/shoot.ts: bundled by Bun with one React, one three.js and one
// React Three Fiber (sdk/r3f's own), the shape module from sdk/r3f/wasm at /wasm/. tools/shoot.ts calls serve();
// `bun tools/serve.ts [PORT]` serves it by hand, printing its URL, until stopped.
import { mkdirSync, mkdtempSync, rmSync, writeFileSync } from "node:fs";
import { tmpdir } from "node:os";
import { join, resolve } from "node:path";

const here = resolve(import.meta.dir, "..");

/** each output case written as R3F source by the source export (test/parity/gen, not in git), its frames up to the
 * case's: the page's dir=src draws them */
async function generate() {
  const { OUTPUTS, SIZE } = await import("../test/parity/fixtures");
  const { toR3FSource } = await import("../src/export");
  const gen = resolve(here, "test/parity/gen");
  mkdirSync(gen, { recursive: true });
  const names = Object.keys(OUTPUTS);
  for (const n of names) {
    const c = OUTPUTS[n];
    const { source, effects } = await toR3FSource(c.component as never, { ...SIZE, from: 0, frames: c.frame + 1, title: `out-${n}` });
    writeFileSync(resolve(gen, `${n}.tsx`), source);
    if (effects) writeFileSync(resolve(gen, "raea-effects.ts"), effects);
  }
  // the page's entry: the page, handed the sources
  writeFileSync(resolve(gen, "main.tsx"), [
    `import { createRoot } from "react-dom/client";`,
    `import Page from "../page";`,
    ...names.map((n, i) => `import { Scene as S${i} } from "./${n}";`),
    `const sources = { ${names.map((n, i) => `${JSON.stringify(n)}: S${i}`).join(", ")} };`,
    `createRoot(document.getElementById("root")!).render(<Page sources={sources} />);`,
  ].join("\n") + "\n");
}

/** the page served on `port` (0: any free one): its URL, and stop() */
export async function serve(port = 0): Promise<{ url: string; stop: () => void }> {
  await generate();
  const out = mkdtempSync(join(tmpdir(), "raea-r3f-parity-"));
  const result = await Bun.build({
    entrypoints: [resolve(here, "test/parity/gen/main.tsx")],
    outdir: out,
    target: "browser",
    define: { "process.env.NODE_ENV": '"production"' },
    plugins: [
      {
        name: "one-copy",
        setup(b) {
          b.onResolve({ filter: /^(react|react-dom|scheduler|three|@react-three\/fiber)(\/.*)?$/ }, (a) => ({ path: Bun.resolveSync(a.path, here) }));
        },
      },
    ],
  });
  if (!result.success) throw new AggregateError(result.logs, "serve: the parity page didn't build");
  const page = `<!doctype html><meta charset="utf-8"><body style="margin:0;background:#000"><div id="root"></div><script type="module" src="/main.js"></script>`;
  const server = Bun.serve({
    port,
    fetch(req) {
      const p = new URL(req.url).pathname;
      if (p === "/" || p === "/index.html") return new Response(page, { headers: { "content-type": "text/html" } });
      if (p.startsWith("/wasm/")) {
        const f = Bun.file(resolve(here, "wasm", p.slice(6)));
        return new Response(f, { headers: { "content-type": p.endsWith(".wasm") ? "application/wasm" : "text/javascript" } });
      }
      const f = Bun.file(join(out, p.slice(1)));
      return new Response(f, { headers: { "content-type": p.endsWith(".js") ? "text/javascript" : "application/octet-stream" } });
    },
    error: () => new Response("not found", { status: 404 }),
  });
  return {
    url: `http://127.0.0.1:${server.port}`,
    stop: () => {
      server.stop(true);
      rmSync(out, { recursive: true, force: true });
    },
  };
}

if (import.meta.main) {
  const s = await serve(Number(process.argv[2] ?? 0));
  console.log(s.url);
  const stop = () => (s.stop(), process.exit(0));
  process.on("SIGTERM", stop);
  process.on("SIGINT", stop);
}
