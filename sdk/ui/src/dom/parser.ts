// Markup into nodes (docs/ts-native.feature P14a): innerHTML's setter and DOMParser, from the composition parser
// (src/html/parse.ts: not the HTML5 tree builder, but the tags, attributes, entities and implied end tags a page or an
// editor's content has). Tiptap parses its content with DOMParser; ProseMirror's clipboard reads pasted HTML through
// innerHTML.
import { parse, type HElement, type HNode } from "../html/parse";
import { DocumentFragment_, Element_, Node_, doc, makeElement, query } from "./all";

const SVG_NS = "http://www.w3.org/2000/svg";

function build(n: HNode, svg: boolean): Node_ {
  if (n.type === "text") return doc.createTextNode(n.text);
  const inSvg = svg || n.tag === "svg";
  const el = inSvg ? makeElement(n.tag, SVG_NS) : makeElement(n.tag);
  for (const [k, v] of n.attrs) el.setAttribute(k, v);
  for (const k of (n.content ?? n).kids) el.appendChild(build(k, inSvg && n.tag !== "foreignObject"));
  return el;
}

/** markup as a fragment of new nodes */
export function fragmentOf(html: string): DocumentFragment_ {
  const f = new DocumentFragment_();
  for (const k of parse(String(html)).kids) f.appendChild(build(k, false));
  return f;
}

/** an element's children replaced by the markup's (innerHTML's setter) */
export function setInnerHtml(el: Node_, html: string) {
  for (const k of el._kids.slice()) el.removeChild(k);
  el.appendChild(fragmentOf(html));
}

/** a parsed document: its own html, head and body, apart from the page's */
class ParsedDocument_ {
  documentElement: Element_;
  head: Element_;
  body: Element_;
  constructor(root: HElement) {
    const find = (tag: string, from: HElement): HElement | undefined => {
      for (const k of from.kids) {
        if (k.type !== "el") continue;
        if (k.tag === tag) return k;
        const f = find(tag, k);
        if (f) return f;
      }
      return undefined;
    };
    this.documentElement = makeElement("html");
    this.head = makeElement("head");
    this.body = makeElement("body");
    this.documentElement.appendChild(this.head);
    this.documentElement.appendChild(this.body);
    const h = find("head", root);
    const b = find("body", root);
    if (h) for (const k of h.kids) this.head.appendChild(build(k, false));
    // what isn't in head goes in body (a fragment's markup, as the HTML parser puts it)
    const kids = b ? b.kids : root.kids.filter((k) => !(k.type === "el" && (k.tag === "html" || k.tag === "head")));
    const htmlEl = find("html", root);
    const bodyKids = b ? kids : htmlEl ? htmlEl.kids.filter((k) => !(k.type === "el" && k.tag === "head")) : kids;
    for (const k of bodyKids) this.body.appendChild(build(k, false));
  }
  get nodeType() {
    return 9;
  }
  get firstChild() {
    return this.documentElement;
  }
  querySelector(s: string) {
    return query(this.documentElement, s, true)[0] ?? null;
  }
  querySelectorAll(s: string) {
    return query(this.documentElement, s, false);
  }
  getElementsByTagName(t: string) {
    return this.querySelectorAll(t);
  }
  createElement(t: string) {
    return doc.createElement(t);
  }
  createTextNode(t: string) {
    return doc.createTextNode(t);
  }
  createDocumentFragment() {
    return doc.createDocumentFragment();
  }
}

export class DOMParser_ {
  parseFromString(s: string, _type = "text/html") {
    return new ParsedDocument_(parse(String(s)));
  }
}

/** document.implementation: a detached document to parse into (ProseMirror's clipboard) */
export const implementation = {
  createHTMLDocument(title = "") {
    const d = new ParsedDocument_(parse(`<html><head><title>${title}</title></head><body></body></html>`));
    return d;
  },
  hasFeature: () => true,
};
