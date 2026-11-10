// `raea show`: what a session shows the person goes to the raea app, as an item they can use, not to Preview (Shane
// 11-09: "when using raea and raea app is installed new edits new sheets or examples or questions are pulled up in the
// raea app and not preview. we should see them as interactive too. the whole point of raea is interactive artifacts and
// things and recording them after making an answer"). The app's host makes the item (apps/app host/show.ts): stills a
// look (several, a storyboard), a video a watch, a sound a listen, a NAME.script.ts or .md a read, a .tsx an interactive
// artifact (or, when it's one of the project's compositions, that composition open in its editor), a question an ask.
// The window comes up on it (raea://item/...), and what the person sends on it comes back as lines in the session's
// inbox (~/.raea/inbox/<inbox>.jsonl). Without the app on this Mac, the files open in the Mac's own viewer and it says so.

type Ask = (op: string, args: Record<string, unknown>) => any;

export const SHOW_USAGE = `raea show FILE... [--title T] [--ask "what you want from the person"] [--project P] [--replaces ITEM]
raea show --question "Which look?" (--choice A --choice B ... | STILL.png STILL.png ...) [--project P]
  FILE: stills (several make a storyboard), a video, a sound, a NAME.script.ts or .md, or a .tsx (an interactive
  artifact; one of the project's compositions opens in its editor). P: the project's id, title or Workspace/project;
  else the one this folder is in. Who's showing it: --as NAME [--role ROLE] [--inbox NAME], else your harness's.
`;

const WITH_VALUE = new Set(["--title", "--ask", "--project", "--replaces", "--as", "--role", "--inbox", "--question", "--choice"]);

/** The harness an agent runs in, named as the feed shows it, from the program that started this command (native.ts
 *  walks up its parents): any harness, not only the ones that mark their shells. */
export const HARNESSES: [RegExp, string][] = [
  [/^claude$/, "Claude Code"], [/^codex/, "Codex"], [/^gemini$/, "Gemini"], [/^kimi$/, "Kimi"], [/^opencode$/, "OpenCode"],
  [/^goose$/, "Goose"], [/^cursor-agent$/, "Cursor"], [/^amp$/, "Amp"], [/^qwen$/, "Qwen Code"], [/^crush$/, "Crush"], [/^aider$/, "Aider"],
];

/** The session's name and inbox, from its harness when it doesn't say: its shell's marks, else the harness that started
 *  this command (`parent`, any harness), one inbox per session so its answers find it. */
export function posterOf(env: Record<string, string | undefined>, as?: string, role?: string, inbox?: string, cwd = "", parent?: { name: string; pid: number }) {
  const marked = env.CLAUDECODE ? "Claude Code" : env.CODEX_SANDBOX || env.CODEX_HOME || env.CODEX_THREAD_ID ? "Codex" : env.GEMINI_CLI ? "Gemini" : env.OPENCODE ? "OpenCode" : undefined;
  const name = as?.trim() || marked || parent?.name || env.RAEA_AGENT || "An agent";
  const session = env.CLAUDE_CODE_SESSION_ID || env.CODEX_THREAD_ID || env.CODEX_SESSION_ID || env.RAEA_SESSION || (parent ? `p${parent.pid}` : hash(cwd));
  const slug = name.toLowerCase().replace(/[^a-z0-9]+/g, "-").replace(/^-|-$/g, "") || "agent";
  return { name, ...(role?.trim() ? { role: role.trim() } : {}), inbox: inbox?.trim() || `${slug}-${session.replace(/[^\w.-]/g, "").slice(0, 8)}` };
}

const hash = (s: string) => {
  let h = 5381;
  for (const c of s) h = ((h << 5) + h + c.charCodeAt(0)) >>> 0;
  return h.toString(36);
};

export function parseShow(args: string[], cwd: string) {
  const one = (name: string) => {
    const i = args.lastIndexOf(name);
    return i >= 0 ? args[i + 1] : undefined;
  };
  const choices = args.flatMap((a, i) => (a === "--choice" && args[i + 1] !== undefined ? [args[i + 1]] : []));
  const unknown = args.find((a, i) => a.startsWith("--") && !WITH_VALUE.has(a) && !WITH_VALUE.has(args[i - 1] ?? ""));
  if (unknown) throw new Error(`no ${unknown} here\n${SHOW_USAGE}`);
  const files = args.filter((a, i) => !a.startsWith("--") && !WITH_VALUE.has(args[i - 1] ?? "")).map((f) => (f.startsWith("/") ? f : `${cwd.replace(/\/$/, "")}/${f}`));
  return { files, choices, title: one("--title"), ask: one("--ask"), project: one("--project"), replaces: one("--replaces"), question: one("--question"), as: one("--as"), role: one("--role"), inbox: one("--inbox") };
}

export function showVerb(ask: Ask, args: string[], cwd: string, env: Record<string, string | undefined>): string {
  if (!args.length || args[0] === "--help") return SHOW_USAGE;
  const a = parseShow(args, cwd);
  const parent = (): { name: string; pid: number } | undefined => {
    try {
      return ask("harness", {});
    } catch {
      return undefined; // a shell without the native raea: the show below says so
    }
  };
  const by = posterOf(env, a.as, a.role, a.inbox, cwd, a.as && a.inbox ? undefined : parent());
  const r = ask("show", { files: a.files, title: a.title, ask: a.ask, question: a.question, choices: a.choices, project: a.project, replaces: a.replaces, cwd, by }) as
    | { opened: string[] }
    | { kind: "item" | "composition"; id: string; type: string; title: string; project: string; workspace: string; inbox: string };
  if ("opened" in r) return `The raea app isn't on this Mac, so ${r.opened.length === 1 ? "it opened" : `${r.opened.length} files opened`} in the Mac's own viewer: nothing was posted and nothing comes back. Install raea (raea.io) to show work as items the person can answer.\n`;
  const home = env.HOME ?? "";
  const inbox = home && r.inbox.startsWith(home) ? `~${r.inbox.slice(home.length)}` : r.inbox;
  if (r.kind === "composition") return `Opened ${r.title} in the raea app's composition editor (project ${r.project}, ${r.workspace}): the person plays it and changes it there, and their changes save into its file.\n`;
  return [
    `Shown in the raea app: "${r.title}" (${r.type}, item ${r.id}, project ${r.project} in ${r.workspace}); the window is on it.`,
    `What the person sends on it (notes, edits, picks, answers, results) comes to ${inbox}, one JSON line each. Follow it (tail -n0 -F ${inbox}, or your harness's monitor) and answer each; a next version is raea show --replaces ${r.id} FILE.`,
    "",
  ].join("\n");
}
