// `raea brand`'s reader: a brand's website, fetched on this machine (HTML and its linked CSS, no browser and no raea API),
// read into what brand.ts writes the kit from. The tokens come from the site's own CSS:
//   colours   custom properties named for a role (--primary, --background, --foreground ...), else the colours the
//             site's rules actually use, counted by role (backgrounds, text, buttons and links)
//   faces     font-family on body and p (body) and h1/h2 (heading), through var(); Google Fonts links kept as Google's
//   type      the font-size, weight, line height and tracking of h1-h3 and p (px, or rem/em at 16 px; clamp()'s max)
//   button    the first rule for a button, .btn or .cta with a background
//   logos     inline <svg> marked logo, <img> marked logo, apple-touch-icon, og:image, the favicon
// What a plain fetch can't see (a page drawn by JavaScript, styles set in script) comes back as `thin`: said in the
// output and BRAND.md, so the agent reads the live page in its own browser instead of trusting a near-empty kit.

/** One CSS rule: its selector (with any @media prelude dropped) and its declarations, last one winning per property */
type Rule = { sel: string; decl: Record<string, string>; dark?: boolean };

/** What the site gave: the kit's input (logos still as URLs), and why it may be too little to trust */
export type Site = {
  brand: { name: string; domain: string; description: string | null; colors: string[] };
  styleguide: any;
  logos: { url: string; type: string; mode?: string; svg?: string }[];
  thin: string[];
};

const NAMED: Record<string, string> = { white: "#ffffff", black: "#000000" };

/** A CSS colour as #rrggbb, or undefined (transparent, a var(), a keyword other than white/black) */
export function hexOf(v: string): string | undefined {
  const s = v.trim().toLowerCase();
  if (NAMED[s]) return NAMED[s];
  let m = /^#([0-9a-f]{3,8})$/.exec(s);
  if (m) {
    const h = m[1];
    if (h.length === 3 || h.length === 4) return "#" + h.slice(0, 3).split("").map((c) => c + c).join("");
    if (h.length === 8 && parseInt(h.slice(6), 16) < 0x80) return undefined;
    if (h.length === 6 || h.length === 8) return "#" + h.slice(0, 6);
    return undefined;
  }
  m = /^rgba?\(\s*([\d.]+)[\s,]+([\d.]+)[\s,]+([\d.]+)(?:\s*[,/]\s*([\d.]+%?))?\s*\)$/.exec(s);
  if (m) {
    if (m[4] !== undefined && (m[4].endsWith("%") ? parseFloat(m[4]) / 100 : parseFloat(m[4])) < 0.5) return undefined;
    return "#" + [m[1], m[2], m[3]].map((x) => Math.max(0, Math.min(255, Math.round(+x))).toString(16).padStart(2, "0")).join("");
  }
  m = /^hsla?\(\s*([\d.]+)(?:deg)?[\s,]+([\d.]+)%[\s,]+([\d.]+)%(?:\s*[,/]\s*([\d.]+%?))?\s*\)$/.exec(s);
  if (m) {
    if (m[4] !== undefined && (m[4].endsWith("%") ? parseFloat(m[4]) / 100 : parseFloat(m[4])) < 0.5) return undefined;
    const [h, sat, l] = [+m[1] / 360, +m[2] / 100, +m[3] / 100];
    const f = (n: number) => {
      const k = (n + h * 12) % 12;
      return l - sat * Math.min(l, 1 - l) * Math.max(-1, Math.min(k - 3, 9 - k, 1));
    };
    return "#" + [f(0), f(8), f(4)].map((x) => Math.round(x * 255).toString(16).padStart(2, "0")).join("");
  }
  m = /^okl(ch|ab)\(\s*([\d.]+)(%?)\s+(-?[\d.]+)%?\s+(-?[\d.]+)(?:deg)?(?:\s*\/\s*([\d.]+%?))?\s*\)$/.exec(s);
  if (m) {
    if (m[6] !== undefined && (m[6].endsWith("%") ? parseFloat(m[6]) / 100 : parseFloat(m[6])) < 0.5) return undefined;
    const L = +m[2] / (m[3] ? 100 : 1);
    const [a, b] = m[1] === "ch" ? [+m[4] * Math.cos((+m[5] * Math.PI) / 180), +m[4] * Math.sin((+m[5] * Math.PI) / 180)] : [+m[4], +m[5]];
    // OKLab to linear sRGB (Ottosson), then the sRGB curve
    const l = (L + 0.3963377774 * a + 0.2158037573 * b) ** 3, mm = (L - 0.1055613458 * a - 0.0638541728 * b) ** 3, ss = (L - 0.0894841775 * a - 1.291485548 * b) ** 3;
    const lin = [4.0767416621 * l - 3.3077115913 * mm + 0.2309699292 * ss, -1.2684380046 * l + 2.6097574011 * mm - 0.3413193965 * ss, -0.0041960863 * l - 0.7034186147 * mm + 1.707614701 * ss];
    return "#" + lin.map((x) => Math.round(Math.max(0, Math.min(1, x <= 0.0031308 ? 12.92 * x : 1.055 * x ** (1 / 2.4) - 0.055)) * 255).toString(16).padStart(2, "0")).join("");
  }
  return undefined;
}

/** How coloured a colour is (0 grey to 1 pure hue) */
function chroma(hex: string): number {
  const [r, g, b] = [1, 3, 5].map((i) => parseInt(hex.slice(i, i + 2), 16) / 255);
  return Math.max(r, g, b) - Math.min(r, g, b);
}

/** The rules of a stylesheet, innermost blocks only (an @media's rules are read as plain rules), each marked when it
 *  sits in a dark colour scheme's @media */
export function rulesOf(css: string): Rule[] {
  const text = css.replace(/\/\*[\s\S]*?\*\//g, "");
  const out: Rule[] = [];
  const stack: string[] = [];
  let from = 0;
  for (let i = 0; i < text.length; i++) {
    const c = text[i];
    if (c === "{") {
      stack.push(text.slice(from, i).trim());
      from = i + 1;
    } else if (c === "}") {
      const sel = stack.pop() ?? "";
      const body = text.slice(from, i);
      from = i + 1;
      if (!sel || sel.startsWith("@") || /^(from|to|[\d.]+%)(\s*,\s*(from|to|[\d.]+%))*$/.test(sel) || stack.some((p) => p.startsWith("@keyframes"))) continue;
      const decl: Record<string, string> = {};
      for (const d of body.split(/;(?![^(]*\))/)) {
        const k = d.indexOf(":");
        if (k > 0) decl[d.slice(0, k).trim().toLowerCase()] = d.slice(k + 1).replace(/!important/, "").trim();
      }
      out.push({ sel, decl, dark: stack.some((p) => /prefers-color-scheme\s*:\s*dark/.test(p)) });
    } else if (c === ";" && !stack.length) from = i + 1; // @import and @charset lines at the top
  }
  return out;
}

/** A length in px: px, rem/em at 16, clamp()/min()/max() by their last px-able value */
function pxOf(v: string | undefined): number | undefined {
  if (!v) return undefined;
  const parts = [...v.matchAll(/(-?[\d.]+)(px|rem|em)\b/g)].map((m) => +m[1] * (m[2] === "px" ? 1 : 16));
  if (!parts.length) return /^-?[\d.]+$/.test(v.trim()) ? +v : undefined;
  return Math.round((/^(clamp|max)\(/.test(v.trim()) ? Math.max(...parts) : parts[parts.length - 1]) * 10) / 10;
}

const attr = (tag: string, name: string) => new RegExp(`\\s${name}\\s*=\\s*("([^"]*)"|'([^']*)'|([^\\s>]+))`, "i").exec(tag)?.slice(2).find((x) => x !== undefined);
const decode = (s: string) => s.replace(/&amp;/g, "&").replace(/&quot;/g, '"').replace(/&#39;/g, "'").replace(/&lt;/g, "<").replace(/&gt;/g, ">");

/** The stylesheets a page links (absolute URLs) and its inline <style> blocks */
export function stylesOf(html: string, base: string): { links: string[]; inline: string[] } {
  const links: string[] = [];
  for (const m of html.matchAll(/<link\b[^>]*>/gi)) {
    const rel = attr(m[0], "rel") ?? "";
    const href = attr(m[0], "href");
    if (href && /stylesheet/i.test(rel)) {
      try {
        links.push(new URL(decode(href), base).href);
      } catch {}
    }
  }
  const inline = [...html.matchAll(/<style\b[^>]*>([\s\S]*?)<\/style>/gi)].map((m) => m[1]);
  // style="" attributes count as one more sheet on a made-up selector, so their colours are counted too
  const attrs = [...html.matchAll(/\sstyle\s*=\s*"([^"]*)"/gi)].map((m) => `[inline]{${decode(m[1])}}`).join("\n");
  return { links, inline: [...inline, attrs] };
}

/** The site read from its page and stylesheets. `url` is the page's final URL. */
export function readSite(html: string, sheets: string[], url: string): Site {
  const u = new URL(url);
  const domain = u.hostname.replace(/^www\./, "");
  const thin: string[] = [];
  let rules = sheets.flatMap(rulesOf);

  // ---- custom properties (from :root, html, body and [data-theme] scopes), resolved through var() -------------------
  // a scope is the page itself (:root, html, body), maybe narrowed to a theme ([data-theme=dark], .dark); the page's own
  // theme (its <html>'s data-theme or class) picks which theme's values win
  const htmlTag = /<html\b[^>]*>/i.exec(html)?.[0] ?? "";
  const dark = /dark/i.test(`${attr(htmlTag, "data-theme")} ${attr(htmlTag, "class")} ${attr(htmlTag, "data-color-mode")}`);
  // a dark scheme's @media rules are the page as a dark-mode visitor sees it; read the page as it is by default
  rules = rules.filter((r) => !r.dark || dark);
  const scope = /^(:root|html|body|:host)?((\[data-[\w-]+(=["']?[\w-]+["']?)?\])|\.(dark|light|theme-[\w-]+))*$/;
  const themed = (sel: string) => (/dark/i.test(sel) ? "dark" : /light/i.test(sel) ? "light" : "any");
  const props: Record<string, string> = {};
  for (const want of ["any", dark ? "dark" : "light"])
    for (const r of rules) {
      const parts = r.sel.split(",").map((x) => x.trim());
      if (!parts.every((x) => x && scope.test(x)) || !parts.some((x) => themed(x) === want)) continue;
      for (const [k, v] of Object.entries(r.decl)) if (k.startsWith("--")) props[k] = v;
    }
  // the page's own variables name roles; one set only in a narrower scope (a section, a card) can't, but still says what
  // a var() means: the first such value, below the page's
  const page = { ...props };
  for (const r of rules) for (const [k, v] of Object.entries(r.decl)) if (k.startsWith("--") && !(k in props) && !/dark/i.test(r.sel) === !dark) props[k] = v;
  for (const r of rules) for (const [k, v] of Object.entries(r.decl)) if (k.startsWith("--") && !(k in props)) props[k] = v;
  // variables the CSS uses for colours and faces but never sets: a theme set by script, which a fetch can't see
  const set = new Set(rules.flatMap((r) => Object.keys(r.decl).filter((k) => k.startsWith("--"))));
  const used = new Set(rules.flatMap((r) => ["color", "background", "background-color", "font-family"].flatMap((k) => [...(r.decl[k] ?? "").matchAll(/var\(\s*(--[\w-]+)/g)].map((m) => m[1]))));
  const unset = [...used].filter((v) => !set.has(v));
  if (unset.length >= 5 && unset.length >= used.size * 0.25) thin.push(`${unset.length} of the ${used.size} variables its colours and faces use are set by script, not in its CSS (${unset.slice(0, 4).join(", ")})`);
  const resolve = (v: string | undefined, depth = 0): string | undefined => {
    if (v === undefined || depth > 8) return v;
    return v.replace(/var\(\s*(--[\w-]+)\s*(?:,\s*([^()]*(?:\([^()]*\))?[^()]*))?\)/g, (_, name, fb) => resolve(props[name] ?? fb ?? "", depth + 1) ?? "");
  };
  // shadcn-style tokens are bare "h s% l%" triples: read them as hsl()
  const colourOf = (v: string | undefined) => {
    const s = resolve(v)?.trim();
    if (!s) return undefined;
    return hexOf(s) ?? (/^[\d.]+(deg)?\s+[\d.]+%\s+[\d.]+%$/.test(s) ? hexOf(`hsl(${s.replace(/\s+/g, ", ")})`) : undefined);
  };

  // ---- colours by role ----------------------------------------------------------------------------------------------
  const named = (re: RegExp) => Object.keys(page).filter((k) => re.test(k)).map((k) => colourOf(page[k])).find(Boolean);
  const count = { bg: new Map<string, number>(), text: new Map<string, number>(), action: new Map<string, number>() };
  const add = (m: Map<string, number>, c: string | undefined, w = 1) => c && m.set(c, (m.get(c) ?? 0) + w);
  let pageBg: string | undefined, pageText: string | undefined;
  for (const r of rules) {
    const bg = colourOf(r.decl["background-color"] ?? (r.decl.background && !/url\(|gradient/.test(r.decl.background) ? r.decl.background : undefined));
    const fg = colourOf(r.decl.color);
    const onPage = /(^|,)\s*(html|body)\s*(,|$)/.test(r.sel);
    if (onPage) {
      pageBg ??= bg;
      pageText ??= fg;
    }
    add(count.bg, bg, onPage ? 20 : 1);
    add(count.text, fg, onPage ? 20 : 1);
    if (/\b(button|btn|cta)\b|(^|[\s,>])a(\b|:)|\[role=["']?button/i.test(r.sel)) {
      add(count.action, bg, 2);
      add(count.action, fg);
    }
    for (const k of ["border-color", "fill", "outline-color"]) add(count.action, colourOf(r.decl[k]), 0.5);
  }
  const top = (m: Map<string, number>, ok: (c: string) => boolean = () => true) => [...m.entries()].filter(([c]) => ok(c)).sort((a, b) => b[1] - a[1]).map(([c]) => c);
  const background = named(/^--(color-)?(background|bg|surface|base)(-color)?$/) ?? pageBg ?? top(count.bg, (c) => chroma(c) <= 0.2)[0];
  const text = named(/^--(color-)?(foreground|text|fg|ink|body)(-color)?$/) ?? pageText ?? top(count.text, (c) => c !== background)[0];
  const coloured = (c: string) => chroma(c) > 0.2;
  const accent = named(/^--(color-)?(primary|brand|accent)(-color|-500|-600)?$/) ?? top(count.action, coloured)[0] ?? top(count.bg, coloured)[0] ?? top(count.text, coloured)[0];
  const all = new Map<string, number>();
  for (const m of Object.values(count)) for (const [c, n] of m) all.set(c, (all.get(c) ?? 0) + n);
  const palette = [...new Set([accent, ...top(all, coloured), ...top(all)].filter(Boolean) as string[])].filter((c) => c !== background && c !== text).slice(0, 6);
  if (!accent) thin.push("no brand colour in its CSS");

  // ---- type ---------------------------------------------------------------------------------------------------------
  const fontLinks: Record<string, any> = {};
  for (const m of html.matchAll(/fonts\.googleapis\.com\/css2?\?([^"'\s>]+)/g))
    for (const f of decode(m[1]).matchAll(/family=([^:&;]+)/g)) fontLinks[decodeURIComponent(f[1].replace(/\+/g, " "))] = { type: "google" };
  for (const s of sheets) for (const m of s.matchAll(/@import\s+url\(["']?[^)"']*fonts\.googleapis\.com\/css2?\?([^)"']+)/g))
    for (const f of decode(m[1]).matchAll(/family=([^:&;]+)/g)) fontLinks[decodeURIComponent(f[1].replace(/\+/g, " "))] = { type: "google" };
  const ruleFor = (re: RegExp, key: string) => rules.find((r) => re.test(r.sel) && r.decl[key] !== undefined)?.decl[key];
  const familyOf = (v: string | undefined) => {
    const s = resolve(v);
    if (!s) return undefined;
    const list = s.split(",").map((x) => x.trim().replace(/^["']|["']$/g, "")).filter(Boolean);
    const generic = ["serif", "sans-serif", "monospace", "cursive", "system-ui", "ui-sans-serif", "ui-serif", "-apple-system", "blinkmacsystemfont", "inherit"];
    const family = list.find((f) => !generic.includes(f.toLowerCase()));
    return family ? { fontFamily: family, fontFallbacks: list.filter((f) => generic.includes(f.toLowerCase())) } : undefined;
  };
  const sel = (tag: string) => new RegExp(`(^|,)\\s*${tag}\\s*(,|$)`);
  const bodyFace = familyOf(ruleFor(sel("body"), "font-family") ?? ruleFor(sel("html"), "font-family") ?? ruleFor(sel("p"), "font-family") ?? props["--font-sans"] ?? props["--font-body"]);
  const headFace = familyOf(ruleFor(/(^|,)\s*h1\b/, "font-family") ?? ruleFor(/(^|,)\s*h2\b/, "font-family") ?? props["--font-heading"] ?? props["--font-display"]) ?? bodyFace;
  // a site that sizes type by class: its biggest heading-like size for h1, the next steps down for h2 and h3
  const sized = (re: RegExp) => [...new Set(rules.filter((r) => re.test(r.sel)).map((r) => pxOf(resolve(r.decl["font-size"]))).filter((x): x is number => x !== undefined && x >= 12 && x <= 160))].sort((a, b) => b - a);
  const heads = sized(/\b(h[1-3]|hero|headline|heading|title|display)\b/i);
  const byClass: Record<string, number | undefined> = { h1: heads[0], h2: heads.find((x) => x < (heads[0] ?? 0) * 0.8), h3: heads.find((x) => x < (heads[0] ?? 0) * 0.6) };
  const level = (tag: string, face: any) => {
    // body text is sized on body as often as on p
    const r = (k: string) => resolve(ruleFor(new RegExp(`(^|,)\\s*${tag}\\s*(,|$)`), k) ?? (tag === "p" ? ruleFor(sel("body"), k) : undefined));
    const size = pxOf(r("font-size")) ?? byClass[tag];
    if (size === undefined && !face) return undefined;
    const line = r("line-height");
    const lh = line && /^[\d.]+$/.test(line) && size ? Math.round(+line * size * 10) / 10 : pxOf(line);
    return {
      ...face,
      fontSize: size !== undefined ? `${size}px` : undefined,
      fontWeight: r("font-weight") && /^\d+$/.test(r("font-weight")!) ? +r("font-weight")! : r("font-weight") === "bold" ? 700 : tag === "p" ? 400 : 700,
      lineHeight: lh !== undefined ? `${lh}px` : undefined,
      letterSpacing: r("letter-spacing") && pxOf(r("letter-spacing")) !== undefined ? `${pxOf(r("letter-spacing"))}px` : undefined,
    };
  };
  const headings: Record<string, any> = {};
  for (const h of ["h1", "h2", "h3"]) {
    const l = level(h, headFace);
    if (l?.fontFamily) headings[h] = l;
  }
  const p = level("p", bodyFace) ?? (bodyFace ? { ...bodyFace, fontWeight: 400 } : undefined);
  if (!bodyFace && !headFace) thin.push("no font-family in its CSS");
  if (!headings.h1?.fontSize) thin.push("no heading sizes in its CSS (brand.css has raea's defaults, marked)");

  // ---- the button and card ------------------------------------------------------------------------------------------
  const btn = rules.find((r) => /\b(button|btn|cta)\b/i.test(r.sel) && !/:(hover|focus|active|disabled)|\bclose\b|\bicon\b/i.test(r.sel) && colourOf(r.decl["background-color"] ?? r.decl.background));
  const keep = ["background-color", "color", "border-radius", "padding", "font-family", "font-size", "font-weight", "letter-spacing", "text-transform", "border"];
  const button = btn && {
    primary: {
      css: keep.filter((k) => btn.decl[k] !== undefined).map((k) => `${k}: ${k.endsWith("color") ? colourOf(btn.decl[k]) ?? resolve(btn.decl[k]) : resolve(btn.decl[k])};`).join(" "),
      backgroundColor: colourOf(btn.decl["background-color"] ?? btn.decl.background),
      color: colourOf(btn.decl.color),
      borderRadius: resolve(btn.decl["border-radius"]),
      ...familyOf(btn.decl["font-family"]),
      fontSize: pxOf(resolve(btn.decl["font-size"])),
      fontWeight: btn.decl["font-weight"] && /^\d+$/.test(resolve(btn.decl["font-weight"])!) ? +resolve(btn.decl["font-weight"])! : undefined,
    },
  };
  const radius = resolve(ruleFor(/\bcard\b/i, "border-radius") ?? props["--radius"]);

  // ---- name and logos -----------------------------------------------------------------------------------------------
  const meta = (key: string) => {
    for (const m of html.matchAll(/<meta\b[^>]*>/gi)) if ((attr(m[0], "property") ?? attr(m[0], "name"))?.toLowerCase() === key) return attr(m[0], "content") && decode(attr(m[0], "content")!);
    return undefined;
  };
  const title = decode(/<title[^>]*>([^<]*)<\/title>/i.exec(html)?.[1]?.trim() ?? "");
  const name = meta("og:site_name") ?? title.split(/\s+[|–—·:-]\s+/).sort((a, b) => a.length - b.length)[0] ?? domain;
  const abs = (h: string | undefined) => {
    try {
      return h ? new URL(decode(h), url).href : undefined;
    } catch {
      return undefined;
    }
  };
  const logos: Site["logos"] = [];
  const svgLogo = /<(?:a|div|span|header)\b[^>]*(?:logo|brand)[^>]*>\s*(?:<[^s][^>]*>\s*)*(<svg\b[\s\S]*?<\/svg>)/i.exec(html)?.[1] ?? /<svg\b[^>]*(?:logo|aria-label="[^"]*")[^>]*>[\s\S]*?<\/svg>/i.exec(html)?.[0];
  if (svgLogo && svgLogo.length < 200_000) logos.push({ url: url, type: "logo", svg: svgLogo });
  for (const m of html.matchAll(/<img\b[^>]*>/gi)) {
    if (!/logo/i.test(`${attr(m[0], "src")} ${attr(m[0], "alt")} ${attr(m[0], "class")}`)) continue;
    const src = abs(attr(m[0], "src"));
    if (src && !src.startsWith("data:")) {
      logos.push({ url: src, type: "logo" });
      break;
    }
  }
  for (const m of html.matchAll(/<link\b[^>]*>/gi)) {
    const rel = (attr(m[0], "rel") ?? "").toLowerCase();
    if (rel.includes("apple-touch-icon")) logos.push({ url: abs(attr(m[0], "href"))!, type: "icon" });
    else if (/(^|\s)icon\b/.test(rel) && (/\.svg|\.png/i.test(attr(m[0], "href") ?? "") || /svg|png/.test(attr(m[0], "type") ?? ""))) logos.push({ url: abs(attr(m[0], "href"))!, type: "favicon" });
  }
  const og = abs(meta("og:image"));
  if (og) logos.push({ url: og, type: "og-image" });
  if (!logos.some((l) => l.type === "logo")) thin.push("no logo marked in its HTML");

  // ---- is a plain fetch enough? -------------------------------------------------------------------------------------
  const visible = html.replace(/<(script|style|svg|noscript)\b[\s\S]*?<\/\1>/gi, "").replace(/<[^>]+>/g, " ").replace(/\s+/g, " ").trim();
  if (visible.length < 400) thin.push(`the page has ${visible.length} characters of text without JavaScript (drawn by script)`);
  if (rules.length < 30) thin.push(`its CSS has ${rules.length} rules`);

  const colors: Record<string, string> = {};
  if (accent) colors.accent = accent;
  if (background) colors.background = background;
  if (text) colors.text = text;
  const any = Object.keys(colors).length || p || Object.keys(headings).length;
  const styleguide = any ? {
    mode: background && chroma(background) < 0.2 && parseInt(background.slice(1, 3), 16) < 0x60 ? "dark" : "light",
    colors,
    typography: { headings, ...(p ? { p } : {}) },
    fontLinks,
    components: { ...(button ? { button } : {}), ...(radius ? { card: { borderRadius: radius } } : {}) },
    source: { fetched: url, stylesheets: sheets.length, rules: rules.length, custom_properties: Object.keys(props).length },
  } : null;
  return {
    brand: { name: name || domain, domain, description: meta("og:description") ?? meta("description") ?? null, colors: palette },
    styleguide,
    logos: logos.filter((l, i) => l.url && logos.findIndex((x) => x.url === l.url && !x.svg === !l.svg) === i).slice(0, 5),
    thin,
  };
}
