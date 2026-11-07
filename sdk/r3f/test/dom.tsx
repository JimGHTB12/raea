// Just enough DOM for the three.js host in a test: raea's components rendered to HTML (react-dom/server) and parsed
// into elements Scene3d reads (querySelectorAll, closest, children, attributes, inline style).
import type { ReactNode } from "react";
import { renderToStaticMarkup } from "react-dom/server";
import { Clock } from "@raea/app/clock";

const ENTITY: Record<string, string> = { quot: '"', amp: "&", lt: "<", gt: ">", apos: "'", "#x27": "'", "#39": "'" };
const decode = (s: string) => s.replace(/&(#x?[0-9a-f]+|[a-z]+);/gi, (m, e: string) => ENTITY[e.toLowerCase()] ?? (e.startsWith("#x") ? String.fromCodePoint(parseInt(e.slice(2), 16)) : e.startsWith("#") ? String.fromCodePoint(Number(e.slice(1))) : m));

export class El {
  attrs = new Map<string, string>();
  kids: El[] = [];
  parentElement: El | null = null;
  isConnected = false;
  style: Record<string, string> = {};
  constructor(public tagName: string) {}
  getAttribute(k: string) { return this.attrs.get(k) ?? null; }
  hasAttribute(k: string) { return this.attrs.has(k); }
  setAttribute(k: string, v: string) { this.attrs.set(k, v); }
  get children() { return this.kids; }
  private matches(sel: string): boolean {
    // [data-3d], [data-3d="Kind"], [data-3d$='Material']
    const m = /^\[([\w-]+)(?:([$^]?=)["']([^"']*)["'])?\]$/.exec(sel.trim());
    if (!m) throw new Error(`fake DOM: selector ${sel}`);
    const v = this.attrs.get(m[1]);
    if (v === undefined) return false;
    if (!m[2]) return true;
    return m[2] === "=" ? v === m[3] : m[2] === "$=" ? v.endsWith(m[3]) : v.startsWith(m[3]);
  }
  querySelectorAll(sel: string): El[] {
    const scope = sel.startsWith(":scope > ");
    const s = scope ? sel.slice(9) : sel;
    const out: El[] = [];
    const walk = (e: El) => {
      for (const k of e.kids) {
        if (k.matches(s)) out.push(k);
        if (!scope) walk(k);
      }
    };
    walk(this);
    return out;
  }
  querySelector(sel: string) { return this.querySelectorAll(sel)[0] ?? null; }
  closest(sel: string): El | null {
    for (let e: El | null = this; e; e = e.parentElement) if (e.matches(sel)) return e;
    return null;
  }
}

/** HTML (react-dom/server's: well formed, quoted attributes) as elements under a root */
export function parse(html: string): El {
  const root = new El("root");
  let at = root;
  const tag = /<(\/?)([a-zA-Z0-9]+)((?:\s+[\w-]+(?:="[^"]*")?)*)\s*(\/?)>/g;
  let m: RegExpExecArray | null;
  while ((m = tag.exec(html))) {
    if (m[1]) {
      at = at.parentElement ?? root;
      continue;
    }
    const e = new El(m[2]);
    for (const a of m[3].matchAll(/([\w-]+)(?:="([^"]*)")?/g)) {
      const v = decode(a[2] ?? "");
      e.attrs.set(a[1], v);
      if (a[1] === "style") for (const d of v.split(";")) {
        const [k, ...rest] = d.split(":");
        if (k?.trim()) e.style[k.trim().replace(/-(\w)/g, (_x, c: string) => c.toUpperCase())] = rest.join(":").trim();
      }
    }
    e.parentElement = at === root ? null : at;
    at.kids.push(e);
    const voids = /^(img|br|hr|input|meta|link)$/i.test(m[2]);
    if (!m[4] && !voids) at = e;
  }
  return root;
}

/** a raea tree rendered at a frame, as the elements a host reads */
export function page(node: ReactNode, frame = 0, size: [number, number] = [640, 360], fps = 24): El {
  const html = renderToStaticMarkup(
    <Clock frame={frame} fps={fps} width={size[0]} height={size[1]} durationInFrames={Math.max(1, frame + 1)}>
      {node}
    </Clock>,
  );
  return parse(html);
}
