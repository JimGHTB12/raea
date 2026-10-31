// Portions ported from HyperFrames (Apache-2.0, https://github.com/heygen-com/hyperframes); see NOTICE.
// Loading a page.
import * as wire from "../../wire";
import { adoptDocument, setPageUrl } from "../../dom";
import { type HElement, type HNode, attr, find, parse } from "../parse";
import { compile } from "../compile";
import { P, S, build, exists, g, gatherSheets, isRemote, localPath, namedAccess, read, restyle, runScript, seek, settleText, setupTime, videosAt, warn, warnings } from "./all";

// ---------------------------------------------------------------- load

async function load() {
  const tree = parse(P.source);
  const compiled = await compile(tree, async (p) => (await read(p))?.text, exists);
  warnings.push(...compiled.warnings);
  adoptDocument();
  // a script's `img.src = …`: a project file as local() makes it (a remote URL stays as written)
  setPageUrl((u) => {
    const t = u.trim();
    return !t || /^(data|blob):/i.test(t) || t.startsWith("#") || isRemote(t) || isRemote(P.file) ? t : localPath(t, P.file);
  });
  const htmlN = find(tree, "html");
  const headN = find(tree, "head");
  // (no <body> tag: what's in <html> is the body's, as a browser's parser implies one, not <html> again inside it)
  const bodyN = find(tree, "body") ?? htmlN ?? tree;
  for (const [a, v] of htmlN?.attrs ?? []) document.documentElement.setAttribute(a, v);
  for (const [a, v] of bodyN === tree || bodyN === htmlN ? [] : bodyN.attrs) document.body.setAttribute(a, v);
  // head's styles and links (sub-compositions' after the page's own), never laid out
  if (headN) await build(document.head, { ...headN, kids: headN.kids.filter((k) => k.type === "el" && k.tag !== "script") }, false);
  // a page with a <body> tag and no <head> (WPT's reftests often): its styles, links and metas before the body are
  // head content, as a browser's parser puts them there
  const loose: HNode[] = [];
  const outside = (n: HElement) => {
    for (const k of n.kids) {
      if (k.type !== "el" || k === bodyN || k === headN) continue;
      if (["style", "link", "meta", "title", "base"].includes(k.tag)) loose.push(k);
      else outside(k);
    }
  };
  if (bodyN !== tree) outside(tree);
  if (loose.length) await build(document.head, { ...tree, kids: loose }, false);
  await build(document.body, { ...bodyN, kids: bodyN.kids.filter((k) => k !== headN && !(k.type === "el" && k.tag === "script")) }, false);
  for (const s of compiled.styles.slice(countPageStyles(tree))) {
    const el = s.href ? document.createElement("link") : document.createElement("style");
    if (s.href) {
      el.setAttribute("rel", "stylesheet");
      el.setAttribute("href", s.href);
    } else el.textContent = s.text ?? "";
    document.head.appendChild(el);
  }
  // the cascade before any script runs (GSAP reads computed values as it builds tweens)
  await gatherSheets();
  restyle();
  settleText();
  S.styleDirty = false;
  new g.MutationObserver(() => (S.styleDirty = true)).observe(document.documentElement, { subtree: true, childList: true, attributes: true, characterData: true });
  // laid out once before the scripts run: what they measure (SplitText's lines, GSAP's percentages) is the page's
  wire.flush();
  // (a host that never lays out, as the test host, is waited for a moment only)
  for (let i = 0; i < 3; i++) await new Promise((r) => (g.requestAnimationFrame(r), setTimeout(r, 200)));
  namedAccess();
  for (let i = 0; i < compiled.scripts.length; i++) await runScript(compiled.scripts[i], i);
  try {
    document.dispatchEvent(new g.Event("DOMContentLoaded", { bubbles: true }));
    g.dispatchEvent(new g.Event("load"));
  } catch {}
  // a script's own promises (Tailwind's browser build, a font load) settle before the first frame
  for (let i = 0; i < 4; i++) await new Promise((r) => setTimeout(r, 0));
  const comps = Array.from(document.querySelectorAll("[data-composition-id]"));
  const root = comps.find((e) => e.getAttribute("data-root") === "true") ?? comps[0] ?? null;
  setupTime(root, compiled.aliases);
  await seek(0);
  const w = parseFloat(root?.getAttribute("data-width") ?? "") || g.innerWidth || 1920;
  const h = parseFloat(root?.getAttribute("data-height") ?? "") || g.innerHeight || 1080;
  wire.emit("html.ready", {
    w,
    h,
    duration: S.duration,
    videos: S.videos.filter((v) => !v.audio).map(({ el: _e, ...v }) => v),
    audio: S.videos.filter((v) => v.audio).map(({ el: _e, ...v }) => v),
    warnings,
  });
}

/** How many of compiled.styles are the page's own (built from its markup already): the rest the sub-compositions'. */
function countPageStyles(tree: HElement): number {
  let n = 0;
  const walk = (e: HElement) => {
    if (e.tag === "style" || (e.tag === "link" && (attr(e, "rel") ?? "").toLowerCase().includes("stylesheet") && attr(e, "href"))) n++;
    for (const k of e.kids) if (k.type === "el") walk(k);
  };
  walk(tree);
  return n;
}

wire.on<{ t: number }>("html.seek", ({ t }) => {
  S.queue = S.queue.then(async () => {
    await seek(t);
    wire.emit("html.seeked", { t, videos: videosAt(t) });
  });
});

S.queue = S.queue.then(load).catch((e) => {
  warn(`load: ${(e as Error)?.stack ?? e}`);
  wire.emit("html.ready", { error: String(e), warnings });
});
