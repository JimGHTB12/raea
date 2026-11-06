// `raea library`: an agent's way into the person's libraries (raea docs/libraries.feature), through the raea app's
// host (apps/app host/library/service.ts, asked as host op "library"). An agent can list libraries, make one, ask to
// add a folder or drive (the person allows it in the app; a folder they already granted is added at once), search by
// meaning, list files and tag them. Every command prints what a person would read.

type Ask = (op: string, args: Record<string, unknown>) => any;
type Lib = { id: string; name: string; files: number; indexed: number; sources: { kind: string; label: string; here?: boolean; online: boolean }[] };

export const LIBRARY_USAGE = `raea library   (the workspace's libraries, their sources, and folders waiting for the person)
raea library new NAME
raea library add LIBRARY PATH   (a folder or a drive: the person allows it in the app unless it's already granted)
raea library search "WORDS" [--library LIBRARY] [--kind video|image|audio] [--limit N] [--sound]   (--sound: a described sound, ranked by sound alone)
raea library files LIBRARY
raea library tag SHA256 TAG[,TAG...]
raea library status | raea library scan
LIBRARY is a library's name or id.
`;

const flagOf = (args: string[], name: string) => {
  const i = args.indexOf(name);
  return i >= 0 ? args[i + 1] : undefined;
};
const t = (s: number) => `${Math.floor(s / 60)}:${String(Math.floor(s % 60)).padStart(2, "0")}`;

export function libraryVerb(ask: Ask, args: string[], env: Record<string, string | undefined>): string {
  const workspace = env.RAEA_WORKSPACE ?? "w-default";
  const by = env.RAEA_LIBRARY_BY ?? `agent:${env.RAEA_AGENT ?? "cli"}`;
  const q = (op: string, a: Record<string, unknown> = {}) => ask("library", { op, ...a });
  const all = (): Lib[] => q("list", { workspace }).libraries;
  const find = (key?: string): Lib => {
    if (!key) throw new Error(`which library?\n${LIBRARY_USAGE}`);
    const ls = all();
    const l = ls.find((x) => x.id === key) ?? ls.find((x) => x.name.toLowerCase() === key.toLowerCase());
    if (!l) throw new Error(`no library "${key}"; there are: ${ls.map((x) => x.name).join(", ") || "none yet (raea library new NAME)"}`);
    return l;
  };
  const [sub, ...rest] = args;
  switch (sub) {
    case undefined:
    case "list": {
      const ls = all();
      const waiting: { library: string; path: string; by: string }[] = q("requests").requests;
      if (!ls.length) return "No libraries yet. raea library new NAME makes one.\n";
      return (
        ls
          .map((l) => {
            const src = l.sources.map((s) => `    ${s.kind} ${s.label}${s.here ? "" : s.online ? " (another machine)" : " (offline)"}`);
            const ask = waiting.filter((w) => w.library === l.id).map((w) => `    waiting for the person: ${w.path} (${w.by})`);
            return [`${l.name}  [${l.id}]  ${l.files} files, ${l.indexed} indexed`, ...src, ...ask].join("\n");
          })
          .join("\n") + "\n"
      );
    }
    case "new": {
      // on this Mac unless --cloud (Pro: searchable from the web and other devices)
      const name = rest.filter((w) => w !== "--cloud").join(" ").trim();
      if (!name) throw new Error("raea library new NAME [--cloud]");
      const l = q("create", { workspace, name, by, where: rest.includes("--cloud") ? "cloud" : "local" });
      return `made ${l.name} [${l.id}]${l.id.startsWith("loc_") ? " on this Mac" : " in raea cloud"}. raea library add "${l.name}" PATH asks to add a folder or drive.\n`;
    }
    case "add": {
      const [key, path] = rest;
      if (!path) throw new Error("raea library add LIBRARY PATH");
      const l = find(key);
      const r = q("add", { library: l.id, path, by });
      return r.state === "needs-approval"
        ? `asked: ${r.path} waits for the person to allow it in the raea app (Libraries > ${l.name}). Its files are indexed once they do.\n`
        : `added ${r.label} to ${l.name}; its files are being scanned and indexed on this machine.\n`;
    }
    case "search": {
      // --sound takes no value; the other flags take the word after them
      const valued = (a?: string) => !!a?.startsWith("--") && a !== "--sound";
      const words = rest.filter((a, i) => !a.startsWith("--") && !valued(rest[i - 1])).join(" ");
      if (!words) throw new Error('raea library search "WORDS"');
      const lib = flagOf(rest, "--library");
      const r = q("search", { workspace, words, libraries: lib ? [find(lib).id] : undefined, kind: flagOf(rest, "--kind"), limit: Number(flagOf(rest, "--limit")) || 10, sound: rest.includes("--sound") });
      if (!r.hits.length) return `nothing found for "${words}"\n`;
      const lines = r.hits.map((h: any) => {
        const span = h.t1 > 0 ? ` ${t(h.t0)}-${t(h.t1)}` : "";
        const where = h.local ?? (h.file?.locations?.[0] ? `${h.file.locations[0].label}/${h.file.locations[0].path} (not on this machine)` : h.sha256);
        return `${where}${span}  (${h.matched.join(", ")})${h.text ? `  "${h.text.slice(0, 80)}"` : ""}`;
      });
      const note = r.by === "words" ? (r.rebuilding ? "(by words only: the cloud index is being rebuilt for a new model)\n" : "(by words only: the search model is still loading)\n") : r.by === "sound" ? "(by sound only)\n" : "";
      return `${note}${lines.join("\n")}\n`;
    }
    case "files": {
      const l = find(rest[0]);
      const r = q("files", { library: l.id });
      return r.files.map((f: any) => `${f.path}  ${f.kind}  ${f.sha256.slice(0, 12)}${f.labels.length ? `  labels: ${f.labels.join(", ")}` : ""}${f.tags.length ? `  tags: ${f.tags.join(", ")}` : ""}`).join("\n") + (r.next ? "\n(more: the app lists the rest)\n" : "\n");
    }
    case "tag": {
      const [sha, tags] = rest;
      if (!tags) throw new Error("raea library tag SHA256 TAG[,TAG...]");
      const r = q("tag", { sha256: sha, tags: tags.split(",").map((x) => x.trim()).filter(Boolean) });
      return `tagged ${sha.slice(0, 12)}: ${r.tags.join(", ")}\n`;
    }
    case "scan":
      return `${q("scan").queued} new files to index\n`;
    case "status": {
      const s = q("status");
      return `model ${s.model.phase}; ${s.indexing ? `indexing ${s.indexing.file} (${s.indexing.step}), ` : ""}${s.queue} waiting, ${s.done} done, ${s.failed} failed${s.errors[0] ? `\nlast error: ${s.errors[0].file}: ${s.errors[0].error}` : ""}\n`;
    }
    default:
      throw new Error(LIBRARY_USAGE);
  }
}
