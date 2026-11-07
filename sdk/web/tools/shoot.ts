// raea-web parity, the browser side (tools/parity.sh): each case in test/parity/cases.ts built into a page that plays
// its composition with @raea/web, held at the case's frame, drawn by raea-web on WebGPU in headless Chrome
// (sdk/r3f/tools/chrome.ts, never the person's own), and screenshotted at the composition's size.
//   bun tools/shoot.ts OUT_DIR [case ...]
import { mkdirSync, mkdtempSync, rmSync, writeFileSync } from "node:fs";
import { tmpdir } from "node:os";
import { join, resolve } from "node:path";
import { openChrome } from "../../r3f/tools/chrome";
import { CASES } from "../test/parity/cases";

const here = resolve(import.meta.dir, "..");
const [out, ...only] = process.argv.slice(2);
if (!out) throw new Error("usage: bun tools/shoot.ts OUT_DIR [case ...]");
const cases = CASES.filter((c) => !only.length || only.includes(c.name));

/** the page for one case: its edit (registering "main") played and held, `window.__raea` saying how it went */
function entry(edit: string, frame: number): string {
  return `import ${JSON.stringify(join("..", edit))};
import { loadRaea, play } from "@raea/web";
const w = window as unknown as { __raea: string };
w.__raea = "loading";
const load = () => import("/wasm/raea_web.js" as string);
(async () => {
  if (!("gpu" in navigator)) return void (w.__raea = "error: no WebGPU");
  const m = await loadRaea(load as never);
  let shown = 0;
  play({ stage: document.getElementById("stage")!, Raea: m.Raea, composition: "main", hold: ${frame}, onFirstFrame: () => void (shown = performance.now()), onError: (e) => void (w.__raea = "error: " + String(e)) });
  // the held frame drawn a few times over (a compute pass's or a resource's frame settles), then shown
  const wait = () => (shown && performance.now() - shown > ${Number(process.env.SHOOT_SETTLE ?? 1500)} ? void (w.__raea = w.__raea.startsWith("error") ? w.__raea : "ready") : requestAnimationFrame(wait));
  wait();
})().catch((e) => void (w.__raea = "error: " + String(e)));
`;
}

// the entries sit in the package (test/parity/gen, not in git) so the SDK's paths resolve them; the bundles go to a temp dir
const gen = resolve(here, "test/parity/gen");
mkdirSync(gen, { recursive: true });
const dir = mkdtempSync(join(tmpdir(), "raea-web-parity-"));
try {
  for (const c of cases) {
    const src = join(gen, `${c.name}.tsx`);
    writeFileSync(src, entry(c.edit, c.frame));
    const built = await Bun.build({
      entrypoints: [src], outdir: join(dir, c.name), target: "browser", external: ["/wasm/*"],
      define: { "process.env.NODE_ENV": '"production"' },
      plugins: [{ name: "one-copy", setup(b) { b.onResolve({ filter: /^(react|react-dom|scheduler)(\/.*)?$/ }, (a) => ({ path: Bun.resolveSync(a.path, resolve(here, "../raea")) })); } }],
    });
    if (!built.success) throw new AggregateError(built.logs, `shoot: ${c.name} didn't build`);
  }
  const server = Bun.serve({
    port: 0,
    fetch(req) {
      const p = new URL(req.url).pathname;
      const name = p.split("/")[1];
      if (p.startsWith("/wasm/")) return new Response(Bun.file(resolve(here, "wasm", p.slice(6))), { headers: { "content-type": p.endsWith(".wasm") ? "application/wasm" : "text/javascript" } });
      if (p === `/${name}/`) return new Response(`<!doctype html><meta charset="utf-8"><body style="margin:0;background:#000"><div id="stage" style="position:absolute;inset:0"></div><script type="module" src="/${name}/${name}.js"></script>`, { headers: { "content-type": "text/html" } });
      if (p === "/favicon.ico") return new Response(null, { status: 204 });
      // a bundle's file, else one of the case's own files beside its edit (a glTF, a picture), as raea render finds them;
      // raea-web asks for them under assets/ (Bevy's asset folder on the web, beside the page)
      const built = Bun.file(join(dir, p.slice(1)));
      const c = CASES.find((x) => x.name === name);
      const own = c ? Bun.file(resolve(here, "test/parity", c.edit, "..", p.slice(name.length + 2).replace(/^assets\//, ""))) : built;
      return built.size ? new Response(built, { headers: { "content-type": p.endsWith(".js") ? "text/javascript" : "application/octet-stream" } }) : new Response(own);
    },
  });
  let failed = 0;
  for (const c of cases) {
    const { width, height } = await dims(c.edit);
    const chrome = await openChrome(`http://127.0.0.1:${server.port}/${c.name}/`, width, height, { scale: 1, mobile: false, webgpu: true });
    let state = "";
    for (let i = 0; i < 600; i++) {
      state = String(await chrome.value("window.__raea"));
      if (state === "ready" || state.startsWith("error")) break;
      await Bun.sleep(100);
    }
    if (state === "ready") await chrome.shot(join(out, `${c.name}-web.png`));
    else failed++;
    console.log(`${c.name}: ${state}${state === "ready" && !process.env.SHOOT_LOGS ? "" : `\n  ${[...chrome.logs, ...chrome.problems].slice(process.env.SHOOT_LOGS ? -40 : -8).join("\n  ")}`}`);
    chrome.close();
  }
  server.stop(true);
  if (failed) process.exit(1);
} finally {
  rmSync(dir, { recursive: true, force: true });
}

/** a case's composition size, read from its source (width={N} height={N} on its Composition) */
async function dims(edit: string): Promise<{ width: number; height: number }> {
  const s = await Bun.file(resolve(here, "test/parity", edit)).text();
  const w = /width=\{(\d+)\}/.exec(s), h = /height=\{(\d+)\}/.exec(s);
  if (!w || !h) throw new Error(`shoot: ${edit} names no width and height`);
  return { width: Number(w[1]), height: Number(h[1]) };
}
