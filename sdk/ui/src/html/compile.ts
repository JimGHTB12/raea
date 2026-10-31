// Portions ported from HyperFrames (Apache-2.0, https://github.com/heygen-com/hyperframes); see NOTICE.
// A composition's compile step (docs/ts-native.feature P27), on the parsed tree before anything is built, as engine/
// html's compile.rs did it after the composition format's compiler: sub-compositions (`data-composition-src`) inlined into their
// host, recursively, their relative URLs rebased onto the project root, their styles scoped to the host and put in
// <head>, their scripts at the end of <body> (a script of their own run with `document` and `gsap` scoped to their
// root); the root composition's data-width/height sizing it, <html> and <body>.

import { type HElement, type HNode, attr, elements, find, parse, setAttr, textOf } from "./parse";
import { blocks, splitTop, stripComments } from "./cascade";

export type Script = { src?: string; text: string; scope?: string };
export type Compiled = { styles: { href?: string; text?: string }[]; scripts: Script[]; warnings: string[]; aliases: Map<string, string> };

const URL_ATTRS = ["src", "href", "poster", "xlink:href", "data-composition-src", "data-src"];

function isRelative(u: string) {
  const s = u.trim();
  return !(!s || s.startsWith("#") || s.startsWith("/") || /^(data|blob|about|javascript|mailto):/i.test(s) || s.includes("://") || s.startsWith("//"));
}
/** `dir/../x.png` normalised against `file`'s folder. */
function sibling(file: string, url: string): string {
  const parts = file.split("/").slice(0, -1).filter((p) => p && p !== ".");
  const cut = url.search(/[?#]/);
  const path = cut < 0 ? url : url.slice(0, cut);
  const suffix = cut < 0 ? "" : url.slice(cut);
  for (const seg of path.split("/")) {
    if (seg === "" || seg === ".") continue;
    if (seg === "..") parts.pop();
    else parts.push(seg);
  }
  return parts.join("/") + suffix;
}
/** A sub-composition's relative URL as a project-root path (the composition format's rewriteAssetPath): `../` resolves against
 *  the file's folder, a plain relative path only when that sibling exists. */
function rebase(exists: (p: string) => boolean, file: string, url: string): string {
  if (!isRelative(url)) return url;
  const s = sibling(file, url);
  const path = s.split(/[?#]/)[0];
  return url.startsWith("../") || url === ".." || exists(path) ? s : url;
}
function rebaseCss(exists: (p: string) => boolean, css: string, file: string): string {
  return css.replace(/url\(\s*(['"]?)([^'")]+)\1\s*\)/g, (_m, q: string, u: string) => `url(${q}${rebase(exists, file, u.trim())}${q})`);
}

function authoredIdSelector(sel: string, domId: string): string {
  return sel.replace(new RegExp(`#${domId.replace(/[-/\\^$*+?.()|[\]{}]/g, "\\$&")}(?![-\\w])`, "g"), `[data-hf-authored-id="${domId}"]`).trim();
}
function scopeSelector(sel: string, scope: string, ids: string[]): string {
  const t = sel.trim();
  if (!t || t === "*") return sel;
  if (/^(html|body)$/i.test(t) || t === ":root") return scope;
  for (const id of ids) {
    for (const q of ['"', "'"]) {
      const pat = `[data-composition-id=${q}${id}${q}]`;
      if (t === pat) return `${scope} > [data-hf-inner-root]`;
      if (t.includes(pat)) return t.split(pat).join(scope);
    }
  }
  return `${scope} ${t}`;
}
/** The composition format's scopeCssToComposition: each top-level rule prefixed with the scope; @keyframes / @font-face global,
 *  @media and @supports scoped inside. */
export function scopeCss(css: string, scope: string, ids: string[], map?: (s: string) => string): string {
  let out = "";
  for (const [prelude, body] of blocks(stripComments(css))) {
    if (body === undefined) {
      out += `${prelude};`;
      continue;
    }
    if (prelude.startsWith("@")) {
      const name = (/^@([-\w]+)/.exec(prelude)?.[1] ?? "").toLowerCase();
      out += `${prelude}{${["media", "supports", "container", "layer", "document", "scope"].includes(name) ? scopeCss(body, scope, ids, map) : body}}`;
      continue;
    }
    out += `${splitTop(prelude).map((s) => scopeSelector(map ? map(s) : s, scope, ids)).join(", ")}{${body}}`;
  }
  return out;
}

type Sub = { fragment: HNode[]; styles: string[]; links: string[]; scripts: { src?: string; text: string }[]; authoredId?: string; innerW?: string; innerH?: string; authoredDomId?: string };

async function loadSub(read: (p: string) => Promise<string | undefined>, exists: (p: string) => boolean, src: string, hostId: string | undefined): Promise<Sub | string> {
  const html = await read(src.split(/[?#]/)[0]);
  if (html === undefined) return `${src}: not found`;
  const doc = parse(html);
  const all = elements(doc);
  const template = all.find((e) => e.tag === "template");
  const content: HElement = template?.content ?? find(doc, "body") ?? doc;
  const scope = elements(content);
  const roots = scope.filter((e) => e !== content && attr(e, "data-composition-id") !== undefined);
  const inner = hostId !== undefined ? roots.find((r) => attr(r, "data-composition-id") === hostId) : roots[0];
  const declared = attr(inner ?? roots[0] ?? content, "data-composition-id");
  const styles: string[] = [];
  const links: string[] = [];
  const scripts: { src?: string; text: string }[] = [];
  const drop = new Set<HElement>();
  for (const e of [...all, ...scope.filter((x) => !all.includes(x))]) {
    if (e.tag === "style") {
      styles.push(rebaseCss(exists, textOf(e), src));
      drop.add(e);
    } else if (e.tag === "link") {
      if ((attr(e, "rel") ?? "").toLowerCase().includes("stylesheet") && attr(e, "href")) links.push(rebase(exists, src, attr(e, "href")!));
      drop.add(e);
    } else if (e.tag === "script") {
      const ty = (attr(e, "type") ?? "").toLowerCase();
      if (["", "text/javascript", "application/javascript", "module"].includes(ty)) {
        const s = attr(e, "src");
        scripts.push({ src: s !== undefined ? rebase(exists, src, s) : undefined, text: textOf(e) });
      }
      drop.add(e);
    } else if (e.tag === "meta" || e.tag === "title") drop.add(e);
  }
  const prune = (n: HElement) => {
    n.kids = n.kids.filter((k) => k.type !== "el" || !drop.has(k));
    for (const k of n.kids) if (k.type === "el") prune(k);
  };
  prune(content);
  for (const e of elements(content)) {
    for (const a of e.attrs) {
      if (URL_ATTRS.includes(a[0])) a[1] = rebase(exists, src, a[1]);
      else if (a[0] === "style" && a[1].includes("url(")) a[1] = rebaseCss(exists, a[1], src);
    }
  }
  const innerW = inner ? attr(inner, "data-width") : undefined;
  const innerH = inner ? attr(inner, "data-height") : undefined;
  const authoredDomId = inner ? attr(inner, "id") : undefined;
  if (inner) {
    const style = attr(inner, "style") ?? "";
    const w = innerW ? `${innerW.trim().replace(/px$/, "")}px` : "100%";
    const h = innerH ? `${innerH.trim().replace(/px$/, "")}px` : "100%";
    for (const a of ["data-composition-id", "data-composition-file", "data-start", "data-duration", "data-end", "data-track-index", "data-track", "data-composition-src", "data-hf-authored-duration", "data-hf-authored-end"]) setAttr(inner, a, undefined);
    if (authoredDomId !== undefined) {
      setAttr(inner, "id", undefined);
      setAttr(inner, "data-hf-authored-id", authoredDomId);
    }
    setAttr(inner, "data-hf-inner-root", "true");
    setAttr(inner, "style", style.trim() ? `${style.trim().replace(/;$/, "")};width:${w};height:${h}` : `width:${w};height:${h}`);
    if (hostId === undefined && declared !== undefined) setAttr(inner, "data-composition-id", declared);
  }
  return { fragment: inner ? [inner] : content.kids, styles, links, scripts, authoredId: declared, innerW, innerH, authoredDomId };
}

/** The page compiled in place: sub-compositions inlined, the stylesheets in document order (inline text or a link's
 *  href, relative to the project), the scripts in order. `read` reads a project file's text, `exists` tests one. */
export async function compile(doc: HElement, read: (p: string) => Promise<string | undefined>, exists: (p: string) => boolean): Promise<Compiled> {
  const warnings: string[] = [];
  const aliases = new Map<string, string>();
  const pending: Script[] = [];
  const extraStyles: { href?: string; text?: string }[] = [];
  const seen: string[] = [];
  for (let round = 0; round < 16; round++) {
    const hosts = elements(doc).filter((e) => attr(e, "data-composition-src") !== undefined);
    if (!hosts.length) break;
    for (const host of hosts) {
      const src = attr(host, "data-composition-src")!;
      const hostId = attr(host, "data-composition-id");
      setAttr(host, "data-composition-src", undefined);
      setAttr(host, "data-composition-file", src);
      if (seen.includes(src)) warnings.push(`${src} is mounted more than once; its scripts run unscoped, so selectors may reach every instance`);
      seen.push(src);
      const sub = await loadSub(read, exists, src, hostId);
      if (typeof sub === "string") {
        warnings.push(`sub-composition ${src} not loaded: ${sub}`);
        continue;
      }
      host.kids = sub.fragment;
      if (sub.innerW && attr(host, "data-width") === undefined) setAttr(host, "data-width", sub.innerW);
      if (sub.innerH && attr(host, "data-height") === undefined) setAttr(host, "data-height", sub.innerH);
      if (sub.authoredId && hostId && sub.authoredId !== hostId) aliases.set(sub.authoredId, hostId);
      const scopeId = hostId ?? sub.authoredId;
      for (const href of sub.links) extraStyles.push({ href });
      for (const css of sub.styles) {
        if (scopeId === undefined) {
          extraStyles.push({ text: css });
          continue;
        }
        const ids = [scopeId, ...(sub.authoredId ? [sub.authoredId] : [])];
        const map = sub.authoredDomId ? (s: string) => authoredIdSelector(s, sub.authoredDomId!) : undefined;
        extraStyles.push({ text: scopeCss(css, `[data-composition-id="${scopeId}"]`, ids, map) });
      }
      for (const s of sub.scripts) pending.push(s.src === undefined && scopeId !== undefined ? { text: s.text, scope: scopeId } : s);
    }
  }
  // the page's own styles and scripts in document order (head, then body), then what the sub-compositions brought
  const styles: { href?: string; text?: string }[] = [];
  const scripts: Script[] = [];
  for (const e of elements(doc)) {
    if (e.tag === "style") styles.push({ text: textOf(e) });
    else if (e.tag === "link" && (attr(e, "rel") ?? "").toLowerCase().includes("stylesheet") && attr(e, "href")) styles.push({ href: attr(e, "href") });
    else if (e.tag === "script") {
      const ty = (attr(e, "type") ?? "").toLowerCase();
      if (["", "text/javascript", "application/javascript", "module"].includes(ty)) scripts.push({ src: attr(e, "src"), text: textOf(e) });
    }
  }
  styles.push(...extraStyles);
  scripts.push(...pending);
  // the root composition's data-width/height size it, html and body (the runtime's applyCompositionSizing)
  const comps = elements(doc).filter((e) => attr(e, "data-composition-id") !== undefined);
  const root = comps.find((e) => attr(e, "data-root") === "true") ?? comps[0];
  if (root) {
    const w = parseFloat(attr(root, "data-width") ?? "");
    const h = parseFloat(attr(root, "data-height") ?? "");
    const add = (e: HElement | undefined, decl: string) => e && setAttr(e, "style", `${(attr(e, "style") ?? "").trim().replace(/;?$/, ";")}${decl}`.replace(/^;/, ""));
    const html = find(doc, "html");
    const body = find(doc, "body");
    if (Number.isFinite(w)) for (const e of [root, html, body]) add(e, `width:${w}px${e === root ? `;--comp-width:${w}px` : ""}`);
    if (Number.isFinite(h)) for (const e of [root, html, body]) add(e, `height:${h}px${e === root ? `;--comp-height:${h}px` : ""}`);
  }
  return { styles, scripts, warnings, aliases };
}
