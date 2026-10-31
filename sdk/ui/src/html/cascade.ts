// The cascade for HTML compositions (docs/ts-native.feature P27). An HTML page's stylesheets select by id, tag,
// attribute and combinator, with specificity, @media, @layer and custom properties that inherit. raea-runtime-js's own sheet
// (runtime/js/src/sheet.rs) matches a single class or a class inside a class only, so the page's rules are
// matched here, on dom.ts's elements with dom.ts's selector engine, and each element's winning declarations go to the
// host as its style, under what its style attribute and its scripts set. What sheet.rs does that this doesn't
// (@font-face, @keyframes) goes to it as a <style> of those rules alone (html.ts).
//
// Not here: :hover and the other state pseudo-classes (a render has no pointer), pseudo-elements other than ::before
// and ::after with `content` (made as a first or last child span), @container queries (taken as matching), @scope.

export type Decl = [prop: string, value: string, important: boolean];
export type Rule = { sel: string; spec: number; order: number; layer: number; decls: Decl[]; pseudo?: "before" | "after" };
export type Sheet = { rules: Rule[]; at: string[]; props: Map<string, string> };

const UNLAYERED = 1e6;

export function stripComments(s: string): string {
  return s.replace(/\/\*[\s\S]*?\*\//g, "");
}

/** The blocks of a stylesheet's level: [prelude, body | undefined] (undefined for a statement ending in `;`). */
export function blocks(css: string): [string, string | undefined][] {
  const out: [string, string | undefined][] = [];
  let i = 0;
  const n = css.length;
  while (i < n) {
    const start = i;
    let depth = 0;
    let q = "";
    while (i < n) {
      const c = css[i];
      if (q) {
        if (c === "\\") i++;
        else if (c === q) q = "";
      } else if (c === '"' || c === "'") q = c;
      else if (c === "(" || c === "[") depth++;
      else if (c === ")" || c === "]") depth--;
      else if (depth <= 0 && (c === "{" || c === ";" || c === "}")) break;
      i++;
    }
    const prelude = css.slice(start, i).trim();
    if (i >= n) {
      if (prelude) out.push([prelude, undefined]);
      break;
    }
    if (css[i] === ";" || css[i] === "}") {
      if (prelude) out.push([prelude, undefined]);
      i++;
      continue;
    }
    // a block: to its matching brace
    let d = 1;
    let j = i + 1;
    q = "";
    while (j < n && d > 0) {
      const c = css[j];
      if (q) {
        if (c === "\\") j++;
        else if (c === q) q = "";
      } else if (c === '"' || c === "'") q = c;
      else if (c === "{") d++;
      else if (c === "}") d--;
      j++;
    }
    out.push([prelude, css.slice(i + 1, d === 0 ? j - 1 : j)]);
    i = j;
  }
  return out;
}

/** Split on commas outside parentheses, brackets and quotes. */
export function splitTop(s: string, sep = ","): string[] {
  const out: string[] = [];
  let depth = 0;
  let q = "";
  let last = 0;
  for (let i = 0; i < s.length; i++) {
    const c = s[i];
    if (q) {
      if (c === "\\") i++;
      else if (c === q) q = "";
    } else if (c === '"' || c === "'") q = c;
    else if (c === "(" || c === "[") depth++;
    else if (c === ")" || c === "]") depth--;
    else if (c === sep && depth === 0) {
      out.push(s.slice(last, i));
      last = i + 1;
    }
  }
  out.push(s.slice(last));
  return out.map((x) => x.trim()).filter(Boolean);
}

/** A declaration block's declarations (nested rules left out: `rulesIn` takes those). */
export function declsOf(body: string): Decl[] {
  const out: Decl[] = [];
  for (const [p, b] of blocks(body)) {
    if (b !== undefined) continue;
    const c = p.indexOf(":");
    if (c <= 0) continue;
    const prop = p.slice(0, c).trim();
    if (!/^(--)?[-a-zA-Z_][-a-zA-Z0-9_]*$/.test(prop)) continue;
    let value = p.slice(c + 1).trim();
    let important = false;
    const m = /!\s*important\s*$/i.exec(value);
    if (m) {
      important = true;
      value = value.slice(0, m.index).trim();
    }
    out.push([prop.startsWith("--") ? prop : prop.toLowerCase(), value, important]);
  }
  return out;
}

/** CSS specificity as one number: ids * 1e6 + classes * 1e3 + types. */
export function specificity(sel: string): number {
  let a = 0;
  let b = 0;
  let c = 0;
  let i = 0;
  const s = sel;
  while (i < s.length) {
    const ch = s[i];
    if (ch === "\\") {
      i += 2;
      continue;
    }
    if (ch === "#") {
      a++;
      i++;
      while (i < s.length && /[-\w -￿\\]/.test(s[i])) i++;
      continue;
    }
    if (ch === ".") {
      b++;
      i++;
      while (i < s.length && /[-\w -￿\\]/.test(s[i])) i++;
      continue;
    }
    if (ch === "[") {
      b++;
      let d = 1;
      i++;
      while (i < s.length && d > 0) {
        if (s[i] === "[") d++;
        else if (s[i] === "]") d--;
        i++;
      }
      continue;
    }
    if (ch === ":") {
      const el = s[i + 1] === ":";
      i += el ? 2 : 1;
      const m = /^[-\w]+/.exec(s.slice(i));
      const name = (m?.[0] ?? "").toLowerCase();
      i += name.length;
      let arg = "";
      if (s[i] === "(") {
        let d = 1;
        const st = ++i;
        while (i < s.length && d > 0) {
          if (s[i] === "(") d++;
          else if (s[i] === ")") d--;
          i++;
        }
        arg = s.slice(st, i - 1);
      }
      if (el || name === "before" || name === "after" || name === "first-line" || name === "first-letter") c++;
      else if (name === "where") {
      } else if (name === "is" || name === "not" || name === "has" || name === "matches") {
        const best = Math.max(0, ...splitTop(arg).map(specificity));
        a += Math.floor(best / 1e6);
        b += Math.floor((best % 1e6) / 1e3);
        c += best % 1e3;
      } else if (name === "nth-child" || name === "nth-last-child") {
        b++;
        const of = /\bof\b(.*)$/.exec(arg);
        if (of) {
          const best = Math.max(0, ...splitTop(of[1]).map(specificity));
          a += Math.floor(best / 1e6);
          b += Math.floor((best % 1e6) / 1e3);
          c += best % 1e3;
        }
      } else b++;
      continue;
    }
    if (/[a-zA-Z_ -￿]/.test(ch) && (i === 0 || /[\s>+~(,]/.test(s[i - 1]))) {
      c++;
      while (i < s.length && /[-\w -￿]/.test(s[i])) i++;
      continue;
    }
    i++;
  }
  return a * 1e6 + b * 1e3 + c;
}

export type Env = { w: number; h: number; dpr?: number };

/** Whether a media query list matches a viewport (width, height, orientation, resolution, the prefers- features at
 *  their defaults, print false). */
export function mediaMatches(q: string, env: Env): boolean {
  const list = splitTop(q.toLowerCase());
  if (!list.length) return true;
  return list.some((one) => {
    let s = one.trim();
    let neg = false;
    if (s.startsWith("not ")) {
      neg = true;
      s = s.slice(4);
    }
    s = s.replace(/^only\s+/, "");
    let ok = true;
    for (const part of s.split(/\s+and\s+/)) {
      const p = part.trim();
      if (!p) continue;
      if (p === "all" || p === "screen") continue;
      if (p === "print" || p === "speech") {
        ok = false;
        continue;
      }
      const m = /^\(\s*([-a-z]+)\s*(?::\s*([^)]+))?\)$/.exec(p);
      if (!m) {
        // range syntax: (width >= 600px), (400px <= width <= 700px)
        const r = /^\(\s*(.+)\s*\)$/.exec(p);
        if (r && /[<>=]/.test(r[1])) {
          ok &&= range(r[1], env);
          continue;
        }
        ok = false;
        continue;
      }
      const [, f, v] = m;
      const px = (x: string) => {
        const n = parseFloat(x);
        return x.endsWith("em") || x.endsWith("rem") ? n * 16 : n;
      };
      switch (f) {
        case "min-width":
          ok &&= env.w >= px(v);
          break;
        case "max-width":
          ok &&= env.w <= px(v);
          break;
        case "min-height":
          ok &&= env.h >= px(v);
          break;
        case "max-height":
          ok &&= env.h <= px(v);
          break;
        case "orientation":
          ok &&= v.trim() === (env.h >= env.w ? "portrait" : "landscape");
          break;
        case "prefers-reduced-motion":
          ok &&= v === undefined ? false : v.trim() === "no-preference";
          break;
        case "prefers-color-scheme":
          ok &&= v?.trim() === "light";
          break;
        case "hover":
        case "any-hover":
          ok &&= v === undefined || v.trim() === "hover";
          break;
        case "pointer":
        case "any-pointer":
          ok &&= v === undefined || v.trim() === "fine";
          break;
        case "min-resolution":
        case "-webkit-min-device-pixel-ratio":
          ok &&= (env.dpr ?? 1) >= parseFloat(v);
          break;
        default:
          break;
      }
    }
    return neg ? !ok : ok;
  });
}
function range(expr: string, env: Env): boolean {
  const toks = expr.split(/\s*(<=|>=|<|>|=)\s*/);
  const val = (t: string) => (t === "width" ? env.w : t === "height" ? env.h : t === "aspect-ratio" ? env.w / env.h : parseFloat(t) * (/(r?em)$/.test(t) ? 16 : 1));
  for (let i = 0; i + 2 < toks.length; i += 2) {
    const a = val(toks[i]);
    const op = toks[i + 1];
    const b = val(toks[i + 2]);
    const ok = op === "<" ? a < b : op === "<=" ? a <= b : op === ">" ? a > b : op === ">=" ? a >= b : a === b;
    if (!ok) return false;
  }
  return true;
}

/** A stylesheet's rules, flattened: conditional groups that match taken in, layers ranked, nesting resolved.
 *  `order` continues from `start`; `layers` is shared across a page's sheets (a layer's rank is its first mention). */
export function parseSheet(css: string, env: Env, layers: Map<string, number>, start = 0): Sheet {
  const sheet: Sheet = { rules: [], at: [], props: new Map() };
  let order = start;
  const rank = (name: string) => {
    let r = layers.get(name);
    if (r === undefined) layers.set(name, (r = layers.size + 1));
    return r;
  };
  let anon = 0;
  const walk = (text: string, parents: string[] | null, layer: number, prefix: string) => {
    for (const [prelude, body] of blocks(text)) {
      if (prelude.startsWith("@")) {
        const name = (/^@([-\w]+)/.exec(prelude)?.[1] ?? "").toLowerCase();
        const rest = prelude.slice(name.length + 1).trim();
        if (body === undefined) {
          if (name === "layer") for (const l of splitTop(rest)) rank(prefix + l);
          continue;
        }
        switch (name) {
          case "media":
            if (mediaMatches(rest, env)) walk(body, parents, layer, prefix);
            break;
          case "supports":
            if (!/^not\b/i.test(rest)) walk(body, parents, layer, prefix);
            break;
          case "container":
          case "document":
            walk(body, parents, layer, prefix);
            break;
          case "layer": {
            const ln = rest ? prefix + rest : `${prefix}#anon${anon++}`;
            walk(body, parents, rank(ln), `${ln}.`);
            break;
          }
          case "property": {
            // a registered custom property's initial value: what var() reads where nothing set it (Tailwind 4's shadows)
            const iv = declsOf(body).find(([p]) => p === "initial-value");
            if (iv) sheet.props.set(rest.trim(), iv[1]);
            break;
          }
          case "font-face": {
            // of a family's unicode-range subsets (Google Fonts' latin, latin-ext, cyrillic...) only the one with Basic
            // Latin: bevy_text takes the first face of a family, whatever it covers
            const ur = declsOf(body).find(([p]) => p === "unicode-range");
            if (ur && !coversLatin(ur[1])) break;
            sheet.at.push(`${prelude}{${body}}`);
            break;
          }
          case "keyframes":
          case "-webkit-keyframes":
            sheet.at.push(`${prelude}{${body}}`);
            break;
          default:
            break;
        }
        continue;
      }
      if (body === undefined) continue;
      // a style rule: its selectors (resolved against the rule it's nested in), its declarations, its nested rules
      let sels = splitTop(prelude);
      if (parents) sels = parents.flatMap((p) => sels.map((s) => (s.includes("&") ? s.replace(/&/g, p) : `${p} ${s}`)));
      const decls = declsOf(body);
      if (decls.length) {
        for (const s of sels) {
          const pm = /::?(before|after)\s*$/i.exec(s);
          const base = pm ? s.slice(0, pm.index).trim() || "*" : s;
          if (/::?[-\w]+/.test(base) && /::/.test(base)) continue;
          sheet.rules.push({ sel: base, spec: specificity(s), order: order++, layer, decls, pseudo: pm ? (pm[1].toLowerCase() as "before" | "after") : undefined });
        }
      }
      if (blocks(body).some(([, b]) => b !== undefined)) walk(body, sels, layer, prefix);
    }
  };
  walk(stripComments(css), null, UNLAYERED, "");
  return sheet;
}

/** `@import` statements of a stylesheet's top: [url, media, layer] each, and the text without them. */
export function imports(css: string): { list: { url: string; media: string; layer?: string }[]; rest: string } {
  const list: { url: string; media: string; layer?: string }[] = [];
  let rest = "";
  for (const [prelude, body] of blocks(stripComments(css))) {
    if (body !== undefined) {
      rest += `${prelude}{${body}}\n`;
      continue;
    }
    if (!/^@import\b/i.test(prelude)) {
      rest += `${prelude};\n`;
      continue;
    }
    const u = /^(?:url\(\s*(['"]?)([^'")]+)\1\s*\)|(['"])([^'"]+)\3)\s*([\s\S]*)$/i.exec(prelude.slice(7).trim());
    if (!u) continue;
    let tail = u[5] ?? "";
    let layer: string | undefined;
    const l = /^layer(?:\(([^)]*)\))?\s*/i.exec(tail);
    if (l) {
      layer = l[1] ?? "";
      tail = tail.slice(l[0].length);
    }
    tail = tail.replace(/^supports\([^)]*\)\s*/i, "");
    list.push({ url: u[2] ?? u[4], media: tail.trim(), layer });
  }
  return { list, rest };
}

// ---------------------------------------------------------------- values

/** `var(--x[, fallback])` replaced from `vars` (recursively); undefined when one can't be (the declaration is then
 *  invalid at computed-value time). */
export function coversLatin(range: string): boolean {
  return range.split(",").some((r) => {
    const m = /u\+([0-9a-f?]+)(?:-([0-9a-f]+))?/i.exec(r.trim());
    if (!m) return false;
    const lo = parseInt(m[1].replace(/\?/g, "0"), 16);
    const hi = m[2] ? parseInt(m[2], 16) : parseInt(m[1].replace(/\?/g, "f"), 16);
    return lo <= 0x41 && hi >= 0x41;
  });
}

/** Registered custom properties' initial values (@property), read where an element has no value for one. */
export const registered = new Map<string, string>();

export function substitute(v: string, vars: Map<string, string>, depth = 0): string | undefined {
  if (!v.includes("var(")) return v;
  if (depth > 16) return undefined;
  let out = "";
  let i = 0;
  while (i < v.length) {
    const k = v.indexOf("var(", i);
    if (k < 0) {
      out += v.slice(i);
      break;
    }
    out += v.slice(i, k);
    let d = 1;
    let j = k + 4;
    while (j < v.length && d > 0) {
      if (v[j] === "(") d++;
      else if (v[j] === ")") d--;
      j++;
    }
    const inner = v.slice(k + 4, j - 1);
    const c = inner.indexOf(",");
    const name = (c < 0 ? inner : inner.slice(0, c)).trim();
    const fb = c < 0 ? undefined : inner.slice(c + 1).trim();
    // `initial` (or nothing set) is the registered initial value, where @property gave one
    let got = vars.get(name);
    if (got === undefined || got === "initial") got = registered.get(name);
    let r: string | undefined;
    if (got !== undefined) r = substitute(got, vars, depth + 1);
    else if (fb !== undefined) r = substitute(fb, vars, depth + 1);
    if (r === undefined) return undefined;
    out += r;
    i = j;
  }
  return out;
}

/** A property's name as dom.ts's style object keys it (camelCase; custom properties as written). */
export function camel(name: string): string {
  if (name.startsWith("--")) return name;
  return name.replace(/^-(webkit|moz|ms)-/, (_m, p) => (p === "ms" ? "ms-" : `${p[0].toUpperCase()}${p.slice(1)}-`)).replace(/-([a-z])/g, (_m, c) => c.toUpperCase());
}

/** Longhands a shorthand resets (a shorthand that wins drops the longhands that lost to it). */
export const LONGHANDS: Record<string, string[]> = {
  margin: ["marginTop", "marginRight", "marginBottom", "marginLeft", "marginInline", "marginBlock"],
  padding: ["paddingTop", "paddingRight", "paddingBottom", "paddingLeft", "paddingInline", "paddingBlock"],
  inset: ["top", "right", "bottom", "left"],
  border: ["borderWidth", "borderStyle", "borderColor", "borderTop", "borderRight", "borderBottom", "borderLeft", "borderTopWidth", "borderRightWidth", "borderBottomWidth", "borderLeftWidth", "borderTopColor", "borderRightColor", "borderBottomColor", "borderLeftColor", "borderTopStyle", "borderRightStyle", "borderBottomStyle", "borderLeftStyle", "borderImage", "borderImageSource", "borderImageSlice", "borderImageWidth", "borderImageOutset", "borderImageRepeat"],
  borderImage: ["borderImageSource", "borderImageSlice", "borderImageWidth", "borderImageOutset", "borderImageRepeat"],
  borderRadius: ["borderTopLeftRadius", "borderTopRightRadius", "borderBottomRightRadius", "borderBottomLeftRadius"],
  background: ["backgroundColor", "backgroundImage", "backgroundSize", "backgroundPosition", "backgroundRepeat", "backgroundClip", "backgroundOrigin", "backgroundAttachment"],
  font: ["fontFamily", "fontSize", "fontWeight", "fontStyle", "lineHeight", "fontVariant", "fontStretch"],
  flex: ["flexGrow", "flexShrink", "flexBasis"],
  overflow: ["overflowX", "overflowY"],
  gap: ["rowGap", "columnGap"],
  transition: ["transitionProperty", "transitionDuration", "transitionTimingFunction", "transitionDelay"],
  animation: ["animationName", "animationDuration", "animationTimingFunction", "animationDelay", "animationIterationCount", "animationDirection", "animationFillMode", "animationPlayState"],
  textDecoration: ["textDecorationLine", "textDecorationColor", "textDecorationStyle", "textDecorationThickness"],
  outline: ["outlineWidth", "outlineStyle", "outlineColor"],
  gridArea: ["gridRow", "gridColumn", "gridRowStart", "gridRowEnd", "gridColumnStart", "gridColumnEnd"],
  grid: ["gridTemplate", "gridTemplateRows", "gridTemplateColumns", "gridTemplateAreas", "gridAutoRows", "gridAutoColumns", "gridAutoFlow"],
  gridTemplate: ["gridTemplateRows", "gridTemplateColumns", "gridTemplateAreas"],
  placeItems: ["alignItems", "justifyItems"],
  placeContent: ["alignContent", "justifyContent"],
};
