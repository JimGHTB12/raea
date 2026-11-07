// An HTML page whose script throws before its timeline registers fails to open (issue #15): rendered, it would be
// its first frame held for the whole duration, with exit 0. Each case loads on the stand-in host through
// html-run.ts in its own process (src/html.ts keeps the page in module state).
import { describe, expect, test } from "bun:test";
import { mkdtempSync, writeFileSync } from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";

// a timeline as GSAP's looks to the runtime: a length and a seek
const TL = "{ totalDuration: () => 2, totalTime() {} }";

async function ready(script: string, rootAttrs = ""): Promise<any> {
  const dir = mkdtempSync(join(tmpdir(), "raea-html-throw-"));
  writeFileSync(
    join(dir, "index.html"),
    `<!doctype html><html><body><div data-composition-id="main" data-start="0" data-duration="2" data-width="640" data-height="360"${rootAttrs}><div id="box"></div></div><script>${script}</script></body></html>`,
  );
  const run = Bun.spawn(["bun", join(import.meta.dir, "html-run.ts"), dir], { stdout: "pipe", stderr: "pipe" });
  const out = await new Response(run.stdout).text();
  await run.exited;
  // html-run prints html.ready's JSON first, then the host's tree
  return JSON.parse(out.slice(0, out.indexOf("\n}") + 2));
}

describe("a page script that throws (#15)", () => {
  test("before its timeline registers: the open fails, naming the error and the composition", async () => {
    const r = await ready(`throw new Error("REPRO: setup failed"); window.__timelines.main = ${TL};`);
    expect(r.error).toContain("REPRO: setup failed");
    expect(r.error).toContain("no timeline registered for the composition main");
  });

  test("after its timeline registers: the page opens", async () => {
    const r = await ready(`window.__timelines.main = ${TL}; throw new Error("late");`);
    expect(r.error).toBeUndefined();
    expect(r.duration).toBe(2);
    expect(r.warnings.join("\n")).toContain("late");
  });

  test("a page with no timeline and no error opens (still on purpose)", async () => {
    const r = await ready(`document.getElementById("box").textContent = "hi";`);
    expect(r.error).toBeUndefined();
  });

  test("data-no-timeline: a throw is a warning only", async () => {
    const r = await ready(`throw new Error("ignored");`, ` data-no-timeline`);
    expect(r.error).toBeUndefined();
    expect(r.warnings.join("\n")).toContain("ignored");
  });
});
