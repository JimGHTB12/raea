// Portions ported from HyperFrames (Apache-2.0, https://github.com/heygen-com/hyperframes); see NOTICE.
// Building the page: text runs settled into elements.
import { effStyle } from "../../dom";
import { type HElement, type HNode } from "../parse";
import { P, abs, g, isRemote, joinPath, local, read } from "./all";

// ---------------------------------------------------------------- building

const SVG_NS = "http://www.w3.org/2000/svg";

const INLINE = new Set(["span", "a", "b", "i", "em", "strong", "code", "small", "label", "abbr", "sub", "sup", "mark", "u", "s", "q", "cite", "kbd", "samp", "var", "time", "br", "img", "button", "input", "select", "textarea", "svg", "wbr", "font", "del", "ins"]);

const PRE = new Set(["pre", "textarea", "script", "style"]);

/** Text as it renders in normal flow: runs of white space as one space, a run at a block's start or end dropped. */
function textFor(parent: HElement, i: number): string | undefined {
  const n = parent.kids[i] as { text: string };
  if (PRE.has(parent.tag)) return n.text;
  let t = n.text.replace(/[ \t\n\r\f]+/g, " ");
  const inlineSib = (k: HNode | undefined) => k !== undefined && (k.type === "text" ? /\S/.test(k.text) : INLINE.has(k.tag));
  const prev = parent.kids[i - 1];
  const next = parent.kids[i + 1];
  // (a space beside a <br> ends or starts a line, and CSS removes it there: a line full up to the space doesn't wrap
  // it onto a line of its own)
  const br = (k: HNode | undefined) => k?.type === "el" && k.tag === "br";
  if (t === " ") return inlineSib(prev) && inlineSib(next) && !br(prev) && !br(next) ? " " : undefined;
  if (!inlineSib(prev) || br(prev)) t = t.replace(/^ /, "");
  if (!inlineSib(next) || br(next)) t = t.replace(/ $/, "");
  return t;
}

/** The texts built as written, to be collapsed (`settleText`) once the cascade says their white-space collapses. */
const rawTexts: { node: Text; parent: HElement; i: number }[] = [];

/** Each text `build` kept as written, collapsed as `textFor` has it (or dropped) unless the white-space it inherits
 *  keeps its spaces (pre, pre-wrap, break-spaces) or its line breaks (pre-line): a `<div style="white-space: pre">`'s
 *  newlines break its lines, as a `<pre>`'s do. */
export function settleText() {
  const keeps = (el: Element | null): string | undefined => {
    for (let e = el; e; e = e.parentElement) {
      const ws = String(effStyle(e as any).whiteSpace ?? "").trim();
      if (ws && ws !== "inherit") return /^(pre|pre-wrap|pre-line|break-spaces)$/.test(ws) ? ws : undefined;
    }
    return undefined;
  };
  for (const { node, parent, i } of rawTexts.splice(0)) {
    const ws = node.isConnected ? keeps(node.parentNode as Element | null) : undefined;
    if (ws === "pre-line") node.textContent = node.data.replace(/[ \t\r\f]*\n[ \t\r\f]*/g, "\n").replace(/[ \t\r\f]+/g, " ");
    if (ws) continue;
    const t = textFor(parent, i);
    if (t) node.textContent = t;
    else node.remove();
  }
}

export async function build(into: Element, n: HElement, svg: boolean): Promise<void> {
  for (let i = 0; i < n.kids.length; i++) {
    const k = n.kids[i];
    if (k.type === "text") {
      if (PRE.has(n.tag) || !k.text) {
        if (k.text) into.appendChild(document.createTextNode(k.text));
        continue;
      }
      const node = document.createTextNode(k.text);
      into.appendChild(node);
      rawTexts.push({ node, parent: n, i });
      continue;
    }
    // scripts run from compile's list, not from the tree
    if (k.tag === "script") continue;
    const inSvg = svg || k.tag === "svg";
    const el = inSvg ? document.createElementNS(SVG_NS, k.tag) : document.createElement(k.tag);
    for (const [a, v] of k.attrs) {
      let val = v;
      if (a === "src" && k.tag === "img" && /\.svg([?#]|$)/i.test(v) && !/^data:/i.test(v)) val = await svgData(v);
      else if ((a === "src" || a === "poster") && !inSvg && (k.tag === "img" || k.tag === "video" || k.tag === "audio" || k.tag === "source")) val = await local(v, P.file);
      else if (a === "style" && v.includes("url(")) val = await cssUrls(v, P.file);
      try {
        el.setAttribute(a, val);
      } catch {}
    }
    if (k.tag === "template") {
      // its content stays out of the document (a script clones it)
      if (k.content) await build((el as any).content ?? el, k.content, false);
    } else await build(el, k, inSvg);
    into.appendChild(el);
  }
}

/** An SVG picture as a data: URL (raea-runtime-js draws `image/svg+xml` data with resvg; its asset loader has no SVG). */
async function svgData(url: string): Promise<string> {
  const got = await read(isRemote(url) ? abs(url) : joinPath(P.file, url));
  if (got?.text === undefined) return url;
  const bytes = new TextEncoder().encode(got.text);
  let bin = "";
  for (let i = 0; i < bytes.length; i++) bin += String.fromCharCode(bytes[i]);
  return `data:image/svg+xml;base64,${g.btoa(bin)}`;
}

export async function cssUrls(css: string, from: string): Promise<string> {
  const found = [...css.matchAll(/url\(\s*(['"]?)([^'")]+)\1\s*\)/g)];
  if (!found.length) return css;
  // (an SVG as a data: URL, which raea-runtime-js draws with resvg: its asset loader reads no .svg file)
  const subs = await Promise.all(found.map(async (m) => {
    const l = await local(m[2], from);
    if (!/\.svg$/i.test(l.split(/[?#]/)[0]) || isRemote(l)) return l;
    const got = await read(l);
    if (got?.text === undefined) return l;
    const bytes = new TextEncoder().encode(got.text);
    let bin = "";
    for (let i = 0; i < bytes.length; i++) bin += String.fromCharCode(bytes[i]);
    return `data:image/svg+xml;base64,${g.btoa(bin)}`;
  }));
  let i = 0;
  return css.replace(/url\(\s*(['"]?)([^'")]+)\1\s*\)/g, () => `url("${subs[i++]}")`);
}
