// Selectors: parsing and matching, :focus-visible, querySelector.
import { Control_, Element_, FOCUSABLE_TAGS, HTMLOptionElement_, Node_, S, disabledEl, doc, html, isField, misses } from "./all";

// ---------------------------------------------------------------- selectors

type Compound = { tag?: string; id?: string; classes: string[]; attrs: [string, string, string, boolean][]; pseudos: [string, any][] };

type Complex = { parts: Compound[]; combs: string[] };

const selCache = new Map<string, Complex[]>();

export function parseSelector(src: string): Complex[] {
  const hit_ = selCache.get(src);
  if (hit_) return hit_;
  let i = 0;
  const s = src;
  const ident = () => {
    const m = /^-?(?:[_a-zA-Z\u00a0-\uffff]|\\.)(?:[-_a-zA-Z0-9\u00a0-\uffff]|\\.)*/.exec(s.slice(i));
    if (!m) throw new SyntaxError(`'${src}' is not a valid selector`);
    i += m[0].length;
    return m[0].replace(/\\(.)/g, "$1");
  };
  const ws = () => {
    while (i < s.length && /\s/.test(s[i])) i++;
  };
  const list = (end: string): Complex[] => {
    const out: Complex[] = [];
    for (;;) {
      ws();
      out.push(complex(end));
      ws();
      if (s[i] === ",") {
        i++;
        continue;
      }
      return out;
    }
  };
  const complex = (end: string): Complex => {
    const parts: Compound[] = [];
    const combs: string[] = [];
    // a leading combinator (relative, as in :has(> x) or a :scope-less querySelector) is a descendant of scope
    ws();
    if (s[i] === ">" || s[i] === "+" || s[i] === "~") {
      parts.push({ classes: [], attrs: [], pseudos: [["scope", null]] });
      combs.push(s[i++]);
      ws();
    }
    parts.push(compound());
    for (;;) {
      const save = i;
      ws();
      if (i >= s.length || s[i] === "," || s[i] === end) {
        i = save;
        ws();
        return { parts, combs };
      }
      let c = " ";
      if (s[i] === ">" || s[i] === "+" || s[i] === "~") {
        c = s[i++];
        ws();
      }
      combs.push(c);
      parts.push(compound());
    }
  };
  const compound = (): Compound => {
    const c: Compound = { classes: [], attrs: [], pseudos: [] };
    if (s[i] === "*") i++;
    else if (/[a-zA-Z_]/.test(s[i] ?? "")) c.tag = ident().toLowerCase();
    for (;;) {
      const ch = s[i];
      if (ch === "#") {
        i++;
        c.id = ident();
      } else if (ch === ".") {
        i++;
        c.classes.push(ident());
      } else if (ch === "[") {
        i++;
        ws();
        const name = ident().toLowerCase();
        ws();
        let op = "";
        let val = "";
        let ci = false;
        if (s[i] !== "]") {
          const m = /^[~|^$*]?=/.exec(s.slice(i));
          if (!m) throw new SyntaxError(`'${src}' is not a valid selector`);
          op = m[0];
          i += op.length;
          ws();
          if (s[i] === '"' || s[i] === "'") {
            const q = s[i++];
            const j = s.indexOf(q, i);
            val = s.slice(i, j);
            i = j + 1;
          } else val = ident();
          ws();
          if (s[i] === "i" || s[i] === "I") {
            ci = true;
            i++;
            ws();
          } else if (s[i] === "s") {
            i++;
            ws();
          }
        }
        if (s[i] !== "]") throw new SyntaxError(`'${src}' is not a valid selector`);
        i++;
        c.attrs.push([name, op, val, ci]);
      } else if (ch === ":") {
        i++;
        if (s[i] === ":") i++;
        const name = ident().toLowerCase();
        let arg: any = null;
        if (s[i] === "(") {
          i++;
          if (["not", "is", "where", "has", "matches"].includes(name)) arg = list(")");
          else {
            const j = s.indexOf(")", i);
            arg = s.slice(i, j).trim();
            i = j;
          }
          ws();
          if (s[i] !== ")") throw new SyntaxError(`'${src}' is not a valid selector`);
          i++;
        }
        c.pseudos.push([name, arg]);
      } else break;
    }
    return c;
  };
  const out = list("");
  ws();
  if (i < s.length) throw new SyntaxError(`'${src}' is not a valid selector`);
  selCache.set(src, out);
  return out;
}

function nth(expr: string, idx: number): boolean {
  const e = expr.replace(/\s+/g, "").toLowerCase();
  if (e === "odd") return idx % 2 === 1;
  if (e === "even") return idx % 2 === 0;
  const m = /^([+-]?\d*)n([+-]\d+)?$/.exec(e);
  if (!m) return idx === Number(e);
  const a = m[1] === "" || m[1] === "+" ? 1 : m[1] === "-" ? -1 : Number(m[1]);
  const b = Number(m[2] ?? 0);
  if (a === 0) return idx === b;
  const n = (idx - b) / a;
  return Number.isInteger(n) && n >= 0;
}

function matchCompound(el: Element_, c: Compound, scope: Node_): boolean {
  if (c.tag && el.localName.toLowerCase() !== c.tag) return false;
  if (c.id !== undefined && el.id !== c.id) return false;
  if (c.classes.length) {
    const cl = el.className.split(/\s+/);
    if (!c.classes.every((x) => cl.includes(x))) return false;
  }
  for (const [name, op, val, ci] of c.attrs) {
    let v = el.getAttribute(name);
    if (v === null) return false;
    let want = val;
    if (ci) {
      v = v.toLowerCase();
      want = want.toLowerCase();
    }
    if (op === "=" && v !== want) return false;
    if (op === "~=" && !v.split(/\s+/).includes(want)) return false;
    if (op === "|=" && v !== want && !v.startsWith(`${want}-`)) return false;
    if (op === "^=" && (!want || !v.startsWith(want))) return false;
    if (op === "$=" && (!want || !v.endsWith(want))) return false;
    if (op === "*=" && (!want || !v.includes(want))) return false;
  }
  for (const [name, arg] of c.pseudos) {
    const sibs = el.parentNode ? el.parentNode._kids.filter((k): k is Element_ => k instanceof Element_) : [el];
    const idx = sibs.indexOf(el) + 1;
    switch (name) {
      case "not":
        if ((arg as Complex[]).some((x) => matchComplex(el, x, scope))) return false;
        break;
      case "is":
      case "where":
      case "matches":
        if (!(arg as Complex[]).some((x) => matchComplex(el, x, scope))) return false;
        break;
      case "has":
        if (!(arg as Complex[]).some((x) => query(el, "", false, [x], el).length > 0)) return false;
        break;
      case "scope":
        if (el !== scope && !(scope === doc && el === html)) return false;
        break;
      case "root":
        if (el !== html) return false;
        break;
      case "first-child":
        if (idx !== 1) return false;
        break;
      case "last-child":
        if (idx !== sibs.length) return false;
        break;
      case "only-child":
        if (sibs.length !== 1) return false;
        break;
      case "nth-child":
        if (!nth(String(arg), idx)) return false;
        break;
      case "nth-last-child":
        if (!nth(String(arg), sibs.length - idx + 1)) return false;
        break;
      case "first-of-type":
        if (sibs.filter((s) => s.localName === el.localName)[0] !== el) return false;
        break;
      case "last-of-type": {
        const t = sibs.filter((s) => s.localName === el.localName);
        if (t[t.length - 1] !== el) return false;
        break;
      }
      case "empty":
        if (el._kids.length || el._inline) return false;
        break;
      case "disabled":
        if (!disabledEl(el)) return false;
        break;
      case "enabled":
        if (!FOCUSABLE_TAGS.has(el.localName) || disabledEl(el)) return false;
        break;
      case "checked":
        if (!((el as Control_).checked === true || (el as HTMLOptionElement_).selected === true)) return false;
        break;
      case "focus":
        if (S.active !== el) return false;
        break;
      case "focus-visible":
        if (S.active !== el || !S.focusVisible) return false;
        break;
      case "focus-within":
        if (!S.active || !el.contains(S.active)) return false;
        break;
      case "hover":
      case "active":
      case "target":
      case "visited":
      case "autofill":
      case "fullscreen":
      case "modal":
      case "popover-open":
        return false;
      case "link":
      case "any-link":
        if (!(el.localName === "a" && el.hasAttribute("href"))) return false;
        break;
      case "read-only":
        if (isField(el) && !el.hasAttribute("readonly")) return false;
        break;
      case "read-write":
        if (!isField(el) || el.hasAttribute("readonly")) return false;
        break;
      case "required":
        if (!el.hasAttribute("required")) return false;
        break;
      case "optional":
        if (el.hasAttribute("required")) return false;
        break;
      case "placeholder-shown":
        if (!(isField(el) && el.hasAttribute("placeholder") && !(el as Control_).value)) return false;
        break;
      default:
        misses.set(`selector.:${name}`, (misses.get(`selector.:${name}`) ?? 0) + 1);
        return false;
    }
  }
  return true;
}

function matchComplex(el: Element_, x: Complex, scope: Node_): boolean {
  const go = (n: Element_, k: number): boolean => {
    if (!matchCompound(n, x.parts[k], scope)) return false;
    if (k === 0) return true;
    const comb = x.combs[k - 1];
    if (comb === ">") {
      const p = n.parentNode;
      return p instanceof Element_ && go(p, k - 1);
    }
    if (comb === " ") {
      for (let p = n.parentNode; p instanceof Element_; p = p.parentNode) if (go(p, k - 1)) return true;
      return false;
    }
    if (comb === "+") {
      const p = n.previousElementSibling;
      return !!p && go(p, k - 1);
    }
    for (let p = n.previousElementSibling; p; p = p.previousElementSibling) if (go(p, k - 1)) return true;
    return false;
  };
  return go(el, x.parts.length - 1);
}

export function matchesSelector(el: Element_, sel: Complex[], scope: Node_): boolean {
  return sel.some((x) => matchComplex(el, x, scope));
}

export function query(root: Node_, s: string, first: boolean, parsed?: Complex[], scope?: Node_): Element_[] {
  const sel = parsed ?? parseSelector(s);
  const out: Element_[] = [];
  const sc = scope ?? root;
  const walk = (n: Node_): boolean => {
    for (const k of n._kids) {
      if (k instanceof Element_) {
        if (matchesSelector(k, sel, sc)) {
          out.push(k);
          if (first) return true;
        }
        if (walk(k)) return true;
      } else if (k._kids.length && walk(k)) return true;
    }
    return false;
  };
  walk(root);
  return out;
}
