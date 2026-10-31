// Portions ported from HyperFrames (Apache-2.0, https://github.com/heygen-com/hyperframes); see NOTICE.
// The cascade: the page's rules applied to its elements' styles.
import { effStyle, setSheet } from "../../dom";
import { type Decl, type Rule, camel, imports, LONGHANDS, parseSheet, registered, substitute } from "../cascade";
import { deterministic } from "../fonts";
import { P, S, abs, cssUrls, g, isRemote, joinPath, pagePath, read } from "./all";

// ---------------------------------------------------------------- the cascade

/** Linked stylesheets' text by href, fetched with their @imports inlined and their url()s made local. */
const linked = new Map<string, string>();

async function sheetText(css: string, from: string, depth = 0): Promise<string> {
  const { list, rest } = imports(css);
  let out = "";
  for (const im of list) {
    if (depth > 8) break;
    const url = isRemote(im.url) || isRemote(from) ? new URL(im.url, isRemote(from) ? abs(from) : undefined).href : joinPath(from, im.url);
    const got = await read(url);
    if (got?.text === undefined) continue;
    let inner = await sheetText(got.text, url, depth + 1);
    if (im.layer !== undefined) inner = `@layer ${im.layer}{${inner}}`;
    if (im.media) inner = `@media ${im.media}{${inner}}`;
    out += `${inner}\n`;
  }
  return out + (await cssUrls(rest, from));
}

async function loadLink(href: string): Promise<void> {
  if (linked.has(href)) return;
  const url = isRemote(href) ? abs(href) : pagePath(href);
  const got = await read(url);
  linked.set(href, got?.text !== undefined ? await sheetText(got.text, url) : "");
}

/** UA defaults an HTML page has that @raea/ui's elements don't (Tailwind's preflight is raea-runtime-js's base). */
const UA = `body{margin:8px} h1{font-size:2em;margin:.67em 0;font-weight:bold} h2{font-size:1.5em;margin:.83em 0;font-weight:bold}
h3{font-size:1.17em;margin:1em 0;font-weight:bold} h4{margin:1.33em 0;font-weight:bold} h5{font-size:.83em;margin:1.67em 0;font-weight:bold}
h6{font-size:.67em;margin:2.33em 0;font-weight:bold} p,blockquote,figure,dl,ul,ol,pre{margin:1em 0} ul,ol{padding-left:40px}
b,strong,th{font-weight:bold} i,em,cite,var,dfn{font-style:italic} small{font-size:smaller} code,kbd,samp,pre{font-family:monospace}
img,video,canvas,svg{display:inline} button{padding:1px 6px} a{color:#0000ee;text-decoration:underline} hr{border:1px inset;margin:.5em 0}
script,style,link,meta,title,head,template,noscript,base{display:none}
[dir=rtl]{direction:rtl} [dir=ltr]{direction:ltr}
*{box-sizing:content-box}`;

/** A text's direction by its first strong character (a letter, or a mark: U+200E LRM, U+200F RLM). */
function strongDir(t: string): "ltr" | "rtl" {
  const c = t.match(/[\p{L}\u200E\u200F]/u)?.[0];
  return c && /[\u200F\p{Script=Hebrew}\p{Script=Arabic}\p{Script=Syriac}\p{Script=Thaana}\p{Script=Nko}]/u.test(c) ? "rtl" : "ltr";
}

const isInherit = (v: unknown) => typeof v === "string" && v.trim().toLowerCase() === "inherit";

if (g.__htmlDebug) g.__htmlRules = () => S.rules;

// (stylo: the page's sheets as CSS for the host's cascade, in document order, and the deterministic faces)


// (undefined until the first gather: a page with no <style> or <link> has the key "" and still needs the UA sheet)



const layers = new Map<string, number>();

export async function gatherSheets(): Promise<boolean> {
  const parts: string[] = [];
  for (const e of Array.from(document.querySelectorAll("style, link"))) {
    if (e.hasAttribute("data-raea-sheet")) continue;
    if (e.localName === "style") {
      const ty = (e.getAttribute("type") ?? "").toLowerCase();
      if (ty && ty !== "text/css") continue;
      // an XHTML page's sheet in a CDATA section (its markers are CSS's CDO/CDC-like garbage otherwise)
      parts.push((e.textContent ?? "").replace(/<!\[CDATA\[|\]\]>/g, ""));
    } else if ((e.getAttribute("rel") ?? "").toLowerCase().includes("stylesheet")) {
      const href = e.getAttribute("href");
      if (!href) continue;
      if (!linked.has(href)) await loadLink(href);
      parts.push(linked.get(href) ?? "");
    }
  }
  const key = parts.join("\n/*--*/\n");
  if (key === S.sheetKey) return false;
  S.sheetKey = key;
  layers.clear();
  const env = { w: g.innerWidth || 0, h: g.innerHeight || 0 };
  const ua = parseSheet(UA, env, layers, 0);
  for (const r of ua.rules) r.layer = -1;
  S.rules = ua.rules;
  registered.clear();
  const at: string[] = [];
  const css: string[] = [];
  let order = ua.rules.length;
  for (const p of parts) {
    // inline <style> text can @import too, and its url()s are the page's
    // (its url()s are the page's, made local as a linked sheet's are)
    const text = p.includes("@import") || p.includes("url(") ? await sheetText(p, P.file) : p;
    css.push(text);
    const s = parseSheet(text, env, layers, order);
    order += s.rules.length + 1;
    S.rules.push(...s.rules);
    at.push(...s.at);
    for (const [k, v] of s.props) registered.set(k, v);
  }
  // The composition format's deterministic fonts: the families the page names pinned to bundled and Google faces. A served page
  // (WPT's) is a plain web page and keeps the families it names, as Chrome draws them: `monospace` is Menlo, not
  // `Inter, monospace`
  if (!P.served) {
    const inline = Array.from(document.querySelectorAll("[style]")).map((e) => e.getAttribute("style") ?? "");
    const faces = await deterministic(S.rules, inline, at, {
      fetch: async (url) => (await read(url))?.text,
      dir: P.fonts,
      bundled: P.fontFiles ?? [],
    });
    if (faces) at.push(faces);
    S.facesCss = faces ? await cssUrls(faces, P.file) : "";
  }
  S.pageCss = css.join("\n");
  return true;
}

type Win = { el: Element; normal: Record<string, unknown>; important: Record<string, unknown> };

/** CSS's logical properties as the physical ones they are in a horizontal, left-to-right flow (the only writing mode
 *  the page lays out): `margin-block-end` is `margin-bottom`, `inline-size` is `width`. A two-sided one
 *  (`inset-inline: a b`) is its two sides. */
const SIDE: Record<string, string> = { BlockStart: "Top", BlockEnd: "Bottom", InlineStart: "Left", InlineEnd: "Right" };

function physical(k: string, v: string): [string, string][] {
  const size = /^(min|max)?(Block|Inline)Size$|^(block|inline)Size$/.exec(k);
  if (size) {
    const axis = (size[2] ?? size[3]).toLowerCase() === "block" ? "Height" : "Width";
    return [[size[1] ? `${size[1]}${axis}` : axis.toLowerCase(), v]];
  }
  const side = /^(margin|padding|border|inset)(Block|Inline)(Start|End)(.*)$/.exec(k);
  if (side) {
    const s = SIDE[side[2] + side[3]];
    return [[side[1] === "inset" ? s.toLowerCase() : `${side[1]}${s}${side[4]}`, v]];
  }
  const both = /^(inset|border)(Block|Inline)(.*)$/.exec(k);
  if (both) {
    const [a, b] = both[1] === "inset" ? v.trim().split(/\s+/) : [v, v];
    const [s0, s1] = both[2] === "Block" ? ["Top", "Bottom"] : ["Left", "Right"];
    const name = (s: string) => (both[1] === "inset" ? s.toLowerCase() : `border${s}${both[3]}`);
    return [[name(s0), a], [name(s1), b ?? a]];
  }
  const corner = /^border(Start|End)(Start|End)Radius$/.exec(k);
  if (corner) return [[`border${corner[1] === "Start" ? "Top" : "Bottom"}${corner[2] === "Start" ? "Left" : "Right"}Radius`, v]];
  return [[k, v]];
}

/** border-width's and outline-width's keywords as Chrome's lengths (thin 1px, medium 3px, thick 5px). */
const WIDTH_KEYWORDS = /\b(thin|medium|thick)\b/g;

const KEYWORD_PX: Record<string, string> = { thin: "1px", medium: "3px", thick: "5px" };

function widthKeywords(k: string, v: string): string {
  if (!/^(border(Top|Right|Bottom|Left)?(Width)?|outline(Width)?|borderBlock.*|borderInline.*)$/.test(k) || !WIDTH_KEYWORDS.test(v)) return v;
  return v.replace(WIDTH_KEYWORDS, (w) => KEYWORD_PX[w]);
}

function put(o: Record<string, unknown>, k: string, v: string) {
  v = widthKeywords(k, v);
  const phys = physical(k, v);
  if (phys.length !== 1 || phys[0][0] !== k) {
    for (const [pk, pv] of phys) put(o, pk, pv);
    return;
  }
  for (const l of LONGHANDS[k] ?? []) delete o[l];
  delete o[k];
  o[k] = v;
}

const INHERIT_VARS = true;

/** Each element's winning declarations sent to the host (dom.ts setSheet); custom properties inherited and var()s
 *  substituted here. */
export function restyle() {
  const matched = new Map<Element, Rule[]>();
  for (const r of S.rules) {
    if (r.pseudo) continue;
    let list: Element[];
    try {
      list = Array.from(document.querySelectorAll(r.sel));
    } catch {
      continue;
    }
    for (const e of list) {
      let m = matched.get(e);
      if (!m) matched.set(e, (m = []));
      m.push(r);
    }
  }
  const html = document.documentElement;
  const walk = (el: Element, vars: Map<string, string>) => {
    const rs = (matched.get(el) ?? []).sort((a, b) => a.layer - b.layer || a.spec - b.spec || a.order - b.order);
    const decls: Decl[] = [];
    for (const r of rs) decls.push(...r.decls);
    const own = (el as any)._style as Record<string, unknown>;
    let mine = vars;
    const customs = decls.filter(([p]) => p.startsWith("--"));
    const ownCustoms = Object.keys(own).filter((k) => k.startsWith("--"));
    if (customs.length || ownCustoms.length) {
      mine = INHERIT_VARS ? new Map(vars) : new Map();
      for (const [p, v, imp] of customs) if (!imp) mine.set(p, v.trim());
      for (const k of ownCustoms) mine.set(k, String(own[k]).trim());
      for (const [p, v, imp] of customs) if (imp) mine.set(p, v.trim());
    }
    const normal: Record<string, unknown> = {};
    const important: Record<string, unknown> = {};
    for (const [p, v, imp] of decls) {
      if (p.startsWith("--")) continue;
      const val = substitute(v, mine);
      if (val === undefined) continue;
      put(imp ? important : normal, camel(p), val);
    }
    // the element's own style may use var() too
    for (const k of Object.keys(own)) {
      const v = own[k];
      if (typeof v === "string" && v.includes("var(") && !k.startsWith("--")) {
        const val = substitute(v, mine);
        if (val !== undefined) put(important, k, val);
      }
    }
    // dir=auto: the direction of the element's first strong character (none: ltr), under any the author sets
    if (el.getAttribute("dir")?.toLowerCase() === "auto") normal.direction ??= strongDir(el.textContent ?? "");
    // `inherit`: the parent's value as it came out (resolved before its children are), else none (the initial value,
    // or the inherited one for a property that inherits anyway)
    const from = el.parentElement ? effStyle(el.parentElement as any) : {};
    const inherit = (to: Record<string, unknown>, k: string) => {
      if (from[k] === undefined) delete to[k];
      else to[k] = from[k];
    };
    for (const to of [normal, important]) for (const k of Object.keys(to)) if (isInherit(to[k])) inherit(to, k);
    for (const k of Object.keys(own)) if (isInherit(own[k]) && from[k] !== undefined) important[k] = from[k];
    setSheet(el as any, normal, important);
    for (const k of Array.from(el.children)) walk(k, mine);
  };
  walk(html, new Map());
  // the host's own sheet: the page's sheets, in document order, and the deterministic faces; stylo cascades them
  const text = `${S.pageCss}\n${S.facesCss}`;
  if (!S.sheetEl) {
    S.sheetEl = document.createElement("style");
    S.sheetEl.setAttribute("data-raea-sheet", "");
    S.sheetEl.textContent = text;
    document.body.appendChild(S.sheetEl);
  } else if (S.sheetEl.textContent !== text) S.sheetEl.textContent = text;
}
