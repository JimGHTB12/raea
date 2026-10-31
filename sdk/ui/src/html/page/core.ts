// Portions ported from HyperFrames (Apache-2.0, https://github.com/heygen-com/hyperframes); see NOTICE.
// The page clock and its files.
import * as wire from "../../wire";
import { type Rule } from "../cascade";
import { Timing } from "../runtime";
import { Video } from "./all";

export const g = globalThis as any;

// served: `dir` is a server's root and `file` a path under it, so a URL starting "/" is the root's (WPT's reftests)
type Page = { source: string; file: string; dir: string; files: string[]; fps: number; fonts?: string; fontFiles?: string[]; served?: boolean };

export const P: Page = g.__html;

export const warnings: string[] = [];

export const warn = (m: string) => {
  warnings.push(m);
  console.warn(`[html] ${m}`);
};

// ---------------------------------------------------------------- the page clock

/** the module's state its files assign (each field was a top-level let) */ export const S: { clockMs: number; rules: Rule[]; pageCss: string; facesCss: string; sheetKey: string | undefined; sheetEl: Element | undefined; styleDirty: boolean; timing: Timing | undefined; rootTl: any; nested: [any, number][]; videos: Video[]; duration: number; queue: Promise<void> } = { clockMs: 0, rules: [], pageCss: "", facesCss: "", sheetKey: undefined, sheetEl: undefined, styleDirty: true, timing: undefined, rootTl: undefined, nested: [], videos: [], duration: 0, queue: Promise.resolve() };

const realNow = Date.now;

Date.now = () => S.clockMs;

try {
  Object.defineProperty(g.performance ?? (g.performance = {}), "now", { value: () => S.clockMs, configurable: true, writable: true });
} catch {}

void realNow;

// ---------------------------------------------------------------- files

const files = new Set(P.files);

export const exists = (p: string) => files.has(p.replace(/^\.\//, ""));

export const isRemote = (u: string) => /^https?:\/\//i.test(u) || u.startsWith("//");

export const abs = (u: string) => (u.startsWith("//") ? `https:${u}` : u);

export async function read(url: string, binary = false): Promise<{ path: string; text?: string } | undefined> {
  try {
    return await wire.call<{ path: string; text?: string }>("html.read", { url: isRemote(url) ? abs(url) : url, binary });
  } catch (e) {
    warn(`${url}: ${(e as Error).message}`);
    return undefined;
  }
}

/** A URL as the host loads it: a project file's absolute path, a remote one fetched into the cache. */
export async function local(url: string, from = ""): Promise<string> {
  const u = url.trim();
  if (!u || /^(data|blob):/i.test(u) || u.startsWith("#")) return u;
  if (isRemote(u) || isRemote(from)) {
    const full = isRemote(u) ? abs(u) : new URL(u, abs(from)).href;
    return (await read(full, true))?.path ?? u;
  }
  return localPath(u, from);
}

/** A project file's URL as the host loads it, relative to `from`'s folder. */
export function localPath(u: string, from: string): string {
  // (served: a URL made local already, by a linked sheet's own pass, is the root's absolute path)
  if (u.startsWith("/") && (!P.served || u.startsWith(`${P.dir}/`))) return u;
  return `${P.dir}/${joinPath(from, u)}`;
}

/** A link's or script's URL as the host reads it: relative to the page (an absolute path stays one unless served). */
export const pagePath = (u: string) => (u.startsWith("/") && !P.served ? u : joinPath(P.file, u));

/** `u` relative to the project file `from`'s folder, normalised. */
export function joinPath(from: string, u: string): string {
  const parts = u.startsWith("/") && P.served ? [] : from.split("/").slice(0, -1).filter((p) => p && p !== ".");
  for (const seg of u.split(/[?#]/)[0].split("/")) {
    if (seg === "" || seg === ".") continue;
    // above the page's folder stays above it (`../media/x` from the folder's top), as Chrome resolves a file:// URL
    if (seg === "..") parts.length && parts[parts.length - 1] !== ".." ? parts.pop() : parts.push("..");
    else parts.push(seg);
  }
  return parts.join("/");
}
