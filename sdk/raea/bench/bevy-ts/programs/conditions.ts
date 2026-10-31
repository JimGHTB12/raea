// gap: G16
// Run conditions, ordering, sets and one-shot systems for TypeScript systems (BevyApp.system options, configureSet,
// oneShot, conditions; docs/bevy-ts.feature G16c), on a headless App:
//   order      three Update systems added a, b (before a), c (after a) run b, a, c every frame; a cycle is refused
//   conditions runOnce runs one frame of four; onTimer(2 frames) runs on frames 2 and 4; resourceExists of a type
//              that isn't there never runs; not(...) of it always does
//   sets       a set ordered after "a" with a runOnce condition runs its two members after a, on the first frame only
//   one-shot   run() answers now; queue() from inside a system runs once that system returns, before the next one
// Prints one JSON line; a wrong answer prints FAIL and throws.
//   sdk/raea/bench/bevy-plugins/motion.sh run sdk/raea/bench/bevy-ts/programs/conditions.ts
import { BevyApp, conditions } from "raea/bevy";
import { print } from "raea/native";

function check(ok: boolean, what: string) {
  if (!ok) {
    print(`FAIL conditions: ${what}`);
    throw new Error(`conditions: ${what}`);
  }
}

export default function main() {
  try {
    run();
  } catch (e) {
    print(`FAIL: ${e instanceof Error ? e.message : String(e)}`);
    throw e;
  }
}

function run() {
  const fps = 60;
  const app = BevyApp.open({ plugins: ["tweening"], fps });
  const log: string[] = [];
  const mark = (s: string) => (_a: BevyApp, f: number) => void log.push(`${f}:${s}`);
  app.system("Update", mark("a"), { name: "a" });
  app.system("Update", mark("b"), { name: "b", before: "a" });
  app.system("Update", mark("c"), { name: "c", after: "a" });
  let cycle = "";
  try {
    app.system("Update", mark("d"), { name: "d", before: "b", after: "c" });
  } catch (e) {
    cycle = e instanceof Error ? e.message : String(e);
  }
  check(cycle.includes("cycle"), `a cycle: ${cycle || "not refused"}`);

  app.system("Update", mark("once"), { when: conditions.runOnce() });
  app.system("Update", mark("timer"), { when: conditions.onTimer(2 / fps, fps) });
  app.system("Update", mark("absent"), { when: conditions.resourceExists("no_such::Resource") });
  app.system("Update", mark("not-absent"), { when: conditions.not(conditions.resourceExists("no_such::Resource")) });

  app.configureSet("late", { when: conditions.runOnce(), after: "a" });
  app.system("Update", mark("s1"), { set: "late" });
  app.system("Update", mark("s2"), { set: "late" });

  const answer = app.oneShot((_a, f) => `shot at ${f}`);
  check(answer.run() === "shot at 0", `run() before any frame: ${answer.run()}`);
  const queued = app.oneShot((_a, f) => void log.push(`${f}:queued`));
  app.system("Update", (a, f) => {
    log.push(`${f}:asks`);
    if (f === 1) queued.queue();
  }, { name: "asks", after: "c" });
  app.system("Update", mark("next"), { after: "asks" });

  app.step(4);
  const at = (f: number) => log.filter((l) => l.startsWith(`${f}:`)).map((l) => l.slice(l.indexOf(":") + 1));
  const f1 = at(1);
  const idx = (s: string) => f1.indexOf(s);
  check(idx("b") < idx("a") && idx("a") < idx("c"), `frame 1 order b, a, c: ${f1.join(" ")}`);
  check(idx("s1") > idx("a") && idx("s2") > idx("a"), `the set runs after a: ${f1.join(" ")}`);
  check(idx("asks") >= 0 && idx("queued") === idx("asks") + 1 && idx("next") > idx("queued"), `the queued one-shot runs right after the system that queued it: ${f1.join(" ")}`);
  const count = (s: string) => log.filter((l) => l.endsWith(`:${s}`)).length;
  check(count("once") === 1 && at(1).includes("once"), `runOnce ran ${count("once")} times`);
  const timer = log.filter((l) => l.endsWith(":timer")).map((l) => Number(l.split(":")[0]));
  check(timer.join() === "2,4", `onTimer(2 frames) ran on frames ${timer.join()}, wants 2,4`);
  check(count("absent") === 0 && count("not-absent") === 4, `resourceExists: absent ${count("absent")}, not-absent ${count("not-absent")}`);
  check(count("s1") === 1 && count("s2") === 1, `the runOnce set ran s1 ${count("s1")} and s2 ${count("s2")} times`);
  check(count("queued") === 1, `the queued one-shot ran ${count("queued")} times`);
  app.close();
  print(JSON.stringify({ ok: true, frame1: f1, timer }));
}
