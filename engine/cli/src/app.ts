// The raea app's host, as the native `raea` reaches it (native.ts): where the running app listens, and the verbs that
// post to it: `raea show` (and its --on tasks), `raea library`, and the agent harness a show names as its poster.
import { spawnSync } from "node:child_process";
import { existsSync, readFileSync } from "node:fs";
import { homedir } from "node:os";
import { basename } from "node:path";
import { HARNESSES } from "./show";

/** The running raea app's host: RAEA_FEED_HOST for an agent the app started, else the address the installed app leaves
 *  in ~/.raea/app-host.json while it runs (apps/app host/host-file.ts), if its process is still there. */
function appHost(): string | null {
  if (process.env.RAEA_FEED_HOST) return process.env.RAEA_FEED_HOST;
  const file = process.env.RAEA_HOST_FILE || `${homedir()}/.raea/app-host.json`;
  try {
    const h = JSON.parse(readFileSync(file, "utf8")) as { url?: string; pid?: number };
    if (h.url && h.pid && spawnSync("kill", ["-0", String(h.pid)]).status === 0) return h.url;
  } catch {
    // no app running, or a file left by one that stopped
  }
  return null;
}

/** POST a host route of the raea app at `at`, answering its JSON; its refusal is thrown as it says it. */
function postHost(at: string, route: string, a: Record<string, unknown>): any {
  const r = spawnSync("curl", ["-sS", "--max-time", "120", "-w", "\n%{http_code}", "-H", "content-type: application/json", "--data-binary", "@-", `${at.replace(/\/$/, "")}${route}`], { input: JSON.stringify(a), encoding: "utf8", maxBuffer: 64 << 20 });
  const text = r.stdout ?? "";
  const cut = text.lastIndexOf("\n");
  const [body, code] = [text.slice(0, cut), Number(text.slice(cut + 1))];
  if (r.status !== 0) throw new Error(`the raea app at ${at} didn't answer: ${(r.stderr ?? "").trim()}`);
  if (!(code >= 200 && code < 300)) throw Object.assign(new Error(body.trim() || `the raea app said ${code}`), { code });
  return JSON.parse(body);
}

/** `raea show` (show.ts): the item made by the raea app's host (apps/app host/show.ts), the app started first when it's
 *  installed and not running. The window is brought up on it by its raea:// link only with --open or when the app was
 *  just started; otherwise the page finds it and says so. Without the app, the files open in the Mac's own viewer. */
export function show(a: Record<string, unknown>): unknown {
  let at = appHost();
  let launched = false;
  if (!at) {
    const app = [process.env.RAEA_APP, "/Applications/raea.app", `${homedir()}/Applications/raea.app`].find((p) => p && existsSync(p));
    const files = (a.files as string[]) ?? [];
    if (!app) {
      if (a.question || a.questions) throw new Error("a question needs the raea app, and it isn't on this Mac (raea.io)");
      spawnSync("open", files);
      return { opened: files };
    }
    if (spawnSync("pgrep", ["-f", `${app}/Contents/MacOS/raea-window`]).status === 0)
      throw new Error("the raea app running here is older than raea show (it leaves no ~/.raea/app-host.json); quit it and open it again to update, then show it again");
    spawnSync("open", ["-a", app]);
    launched = true;
    // the window's host writes its address once it listens: a minute at most, in quarter seconds
    for (let i = 0; i < 240 && !(at = appHost()); i++) spawnSync("/bin/sleep", ["0.25"]);
    if (!at) throw new Error(`the raea app didn't start (no ~/.raea/app-host.json after a minute); open ${app} and show it again`);
  }
  let shown: { url: string };
  try {
    shown = postHost(at, "/host/show", a);
  } catch (e) {
    // the host's refusal names the verb, as the shell's report does too
    throw new Error(String((e as Error).message).replace(/^raea show: /, ""));
  }
  // the window comes up on it only when asked (--open) or when it was just started for it: a post otherwise lands
  // quietly, the page saying so in a notice (Shane 11-10: "any time you make an update to the feed it takes me out of
  // the thing im currently doing"); RAEA_NO_OPEN: a test's, which mustn't bring up the person's window
  // an app from before quiet posts (no /host/shown) never finds an item it isn't sent to: there it comes up, as before
  const finds = (() => {
    try {
      postHost(at!, "/host/shown", {});
      return true;
    } catch {
      return false;
    }
  })();
  const brought = !process.env.RAEA_NO_OPEN && (a.open === true || launched || !finds);
  if (brought) spawnSync("open", [shown.url]);
  return { ...shown, brought };
}

/** The agent harness that started this command (show.ts HARNESSES), found up its parent processes: a program's own
 *  name, or the script a runtime runs (`node .../gemini`). Nothing when no harness is up there. */
export function harness(): { name: string; pid: number } | undefined {
  let pid = process.ppid;
  for (let i = 0; i < 16 && pid > 1; i++) {
    const m = /^\s*(\d+)\s+(.*)$/.exec(spawnSync("ps", ["-o", "ppid=,args=", "-p", String(pid)], { encoding: "utf8" }).stdout ?? "");
    if (!m) return undefined;
    for (const w of m[2].split(/\s+/).slice(0, 2).map((w) => basename(w).toLowerCase())) {
      const hit = HARNESSES.find(([re]) => re.test(w));
      if (hit) return { name: hit[1], pid };
    }
    pid = Number(m[1]);
  }
  return undefined;
}

/** `raea show --on ITEM --working ID` and `--done ID` (show.ts): the person's comments marked on their item by the raea
 *  app's host (apps/app host/tasks.ts), which the window shows as who's on each and which are done. */
export function task(a: Record<string, unknown>): unknown {
  const at = appHost();
  if (!at) throw new Error("the raea app isn't running here: open it, then mark the comments again");
  try {
    return postHost(at, "/host/task", a);
  } catch (e) {
    const old = (e as { code?: number }).code === 404 && !/^raea show: /.test((e as Error).message); // an app from before --on: its host's bare "not found"
    throw new Error(old ? "the raea app running here is older than raea show --on; quit it and open it again to update, then mark the comments again" : String((e as Error).message).replace(/^raea show: /, ""));
  }
}

/** An op of the feed host's libraries (apps/app host/library/service.ts): POST <the app's host>/host/library. */
export function library(a: Record<string, unknown>): unknown {
  const at = appHost();
  if (!at) throw new Error("libraries live in the raea app: open it on this Mac (or set RAEA_FEED_HOST to its address)");
  return postHost(at, "/host/library", a);
}

