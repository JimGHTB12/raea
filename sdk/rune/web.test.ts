// T4b: Rune's props on the web target: a CSS fallback for each that has one, the rest named as ignored.
import { expect, test } from "bun:test";
import { readFileSync } from "node:fs";
import { css } from "./gen-css.ts";
import { web } from "./src/web.ts";

// rune(p) is { "data-rune": JSON.stringify(p), ...web(p).attrs } (index.ts, which needs React to load)
const rune = (p: Parameters<typeof web>[0]) => ({ "data-rune": JSON.stringify(p), ...web(p).attrs });
const Spring = (duration: number, bounce = 0) => ({ duration, bounce });

test("presets, effects, blurs, glass and instant get the web's attributes", () => {
  const a = rune({ motion: "pop", transition: { effect: { push: "top" }, enter: "snap" }, blur: 8, backdropBlur: 0, glass: true, instant: true });
  expect(a["data-rune-motion"]).toBe("pop");
  expect(a["data-rune-enter"]).toBe("push-top");
  expect(a["data-rune-enter-motion"]).toBe("snap");
  expect(a["data-rune-blur"]).toBe("8");
  expect(a["data-rune-backdrop-blur"]).toBe("0");
  expect(a["data-rune-glass"]).toBe("");
  expect(a["data-rune-instant"]).toBe("");
  // the exit is the one thing here the web can't do
  expect(a["data-rune-web-ignored"]).toBe("transition's exit (React removes the element at once)");
  // the native host's prop is as it was
  expect(JSON.parse(a["data-rune"]).motion).toBe("pop");
});

test("what has no CSS is named, never dropped", () => {
  const a = rune({ motion: Spring(0.4, 0.2), transition: { effect: { scale: 0.8 } }, lead: { lead: "snap", trail: "settle" }, blur: 2.5 });
  expect(a["data-rune-motion"]).toBeUndefined();
  expect(a["data-rune-web-ignored"].split("; ")).toEqual([
    "motion (a spring of its own)",
    "transition (an effect with numbers of its own)",
    "transition's exit (React removes the element at once)",
    "lead",
    "blur (2.5: whole points from 0 to 64 only)",
  ]);
  // no transition at all is nothing to ignore
  expect(rune({ transition: "none" })["data-rune-web-ignored"]).toBeUndefined();
});

test("every attribute value rune() writes has a rule in web.css, which is what gen-css.ts writes", () => {
  const sheet = readFileSync(new URL("./src/web.css", import.meta.url), "utf8");
  expect(sheet).toBe(css());
  for (const v of ["instant", "snap", "settle", "reflow", "glide", "morph", "pop", "drift"]) {
    expect(sheet).toContain(`[data-rune-motion="${v}"]`);
    expect(sheet).toContain(`[data-rune-enter-motion="${v}"]`);
  }
  for (const e of ["opacity", "slide", "blurReplace", "move-leading", "push-bottom"]) expect(sheet).toContain(`@keyframes rune-enter-${e} `);
  expect(sheet).toContain(`[data-rune-blur="64"] { filter: blur(64px); }`);
});
