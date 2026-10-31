// Portions ported from HyperFrames (Apache-2.0, https://github.com/heygen-com/hyperframes); see NOTICE.
// The composition format's deterministic fonts (docs/ts-native.feature P27), as engine/html's fonts.rs did them: a page's
// `font-family` gets the same faces whatever machine renders it. A system family is swapped for a bundled stand-in
// (Helvetica, Arial, SF Pro, Avenir -> Inter; Georgia, Times -> EB Garamond; Menlo, Courier -> JetBrains Mono ...),
// the bundle's weights (core/fonts) with Google Fonts' for the weights it lacks; any other family comes from Google
// Fonts; a value whose first family is generic (sans-serif, system-ui) gets Inter put first. The result is @font-face
// text under the family names the page asks for, which raea-runtime-js's sheet registers.

import { type Decl, type Rule, blocks, coversLatin, declsOf, stripComments } from "./cascade";

const ALIASES: Record<string, string> = {
  inter: "inter", montserrat: "montserrat", outfit: "outfit", nunito: "nunito", oswald: "oswald",
  "league gothic": "league-gothic", "archivo black": "archivo-black", "space mono": "space-mono",
  "ibm plex mono": "ibm-plex-mono", "jetbrains mono": "jetbrains-mono", "eb garamond": "eb-garamond",
  "playfair display": "playfair-display", "source code pro": "source-code-pro", "noto sans jp": "noto-sans-jp",
  roboto: "roboto", "open sans": "open-sans", lato: "lato", poppins: "poppins",
  "helvetica neue": "inter", helvetica: "inter", arial: "inter", "helvetica bold": "inter",
  futura: "montserrat", "din alternate": "montserrat", "arial black": "montserrat",
  "bebas neue": "league-gothic", "courier new": "jetbrains-mono", courier: "jetbrains-mono",
  garamond: "eb-garamond", "noto sans japanese": "noto-sans-jp", "segoe ui": "roboto",
  "sf pro": "inter", "sf pro display": "inter", "sf pro text": "inter", "sf pro rounded": "inter",
  avenir: "inter", "avenir next": "inter", "lucida grande": "inter", geneva: "inter", optima: "inter",
  verdana: "inter", tahoma: "inter", "trebuchet ms": "inter", calibri: "inter", candara: "inter", corbel: "inter",
  "lucida sans": "inter", "lucida sans unicode": "inter", "noto sans": "inter", "dejavu sans": "inter",
  "liberation sans": "inter", "sf mono": "jetbrains-mono", menlo: "jetbrains-mono", monaco: "jetbrains-mono",
  consolas: "jetbrains-mono", "lucida console": "jetbrains-mono", "lucida sans typewriter": "jetbrains-mono",
  "andale mono": "jetbrains-mono", "dejavu sans mono": "jetbrains-mono", "liberation mono": "jetbrains-mono",
  georgia: "eb-garamond", palatino: "eb-garamond", "palatino linotype": "eb-garamond", "book antiqua": "eb-garamond",
  cambria: "eb-garamond", times: "eb-garamond", "times new roman": "eb-garamond", "dejavu serif": "eb-garamond",
  "liberation serif": "eb-garamond",
};
const DISPLAY: Record<string, string> = {
  inter: "Inter", montserrat: "Montserrat", outfit: "Outfit", nunito: "Nunito", oswald: "Oswald",
  "league-gothic": "League Gothic", "archivo-black": "Archivo Black", "space-mono": "Space Mono",
  "ibm-plex-mono": "IBM Plex Mono", "jetbrains-mono": "JetBrains Mono", "eb-garamond": "EB Garamond",
  "playfair-display": "Playfair Display", "source-code-pro": "Source Code Pro", "noto-sans-jp": "Noto Sans JP",
  roboto: "Roboto", "open-sans": "Open Sans", lato: "Lato", poppins: "Poppins",
};
const GENERIC = new Set([
  "sans-serif", "serif", "monospace", "cursive", "fantasy", "system-ui", "ui-sans-serif", "ui-serif", "ui-monospace",
  "ui-rounded", "emoji", "math", "fangsong", "-apple-system", "blinkmacsystemfont", "inherit", "initial", "unset",
  "revert", "revert-layer",
]);
/** The composition format's GENERIC_FAMILIES: a value starting with one of these gets Inter first. */
const HF_GENERIC = new Set([
  "sans-serif", "serif", "monospace", "cursive", "fantasy", "system-ui", "ui-sans-serif", "ui-serif", "ui-monospace",
  "emoji", "math", "fangsong", "-apple-system", "blinkmacsystemfont",
]);

/** A family list split at top-level commas, each trimmed and unquoted. */
export function families(value: string): string[] {
  const out: string[] = [];
  let start = 0;
  let depth = 0;
  let quote = "";
  for (let i = 0; i < value.length; i++) {
    const c = value[i];
    if (c === "\\") {
      i++;
      continue;
    }
    if (quote) {
      if (c === quote) quote = "";
      continue;
    }
    if (c === "'" || c === '"') quote = c;
    else if (c === "(") depth++;
    else if (c === ")") depth = Math.max(0, depth - 1);
    else if (c === "," && depth === 0) {
      out.push(value.slice(start, i));
      start = i + 1;
    }
  }
  out.push(value.slice(start));
  return out.map((p) => p.trim().replace(/^['"]/, "").replace(/['"]$/, "").trim()).filter(Boolean);
}

/** `Inter, <value>` when the value's first family is generic; through a leading var() only when that property is
 *  never defined (its fallback then decides). */
function systemPrimary(value: string, defined: Set<string>): string | undefined {
  const t = value.trim();
  const m = /^var\(\s*(--[-\w]+)\s*(?:,([\s\S]*))?\)([\s\S]*)$/i.exec(t);
  if (m) {
    if (defined.has(m[1]) || m[2] === undefined) return undefined;
    const fb = systemPrimary(m[2], defined);
    return fb === undefined ? undefined : `var(${m[1]}, ${fb})${m[3]}`;
  }
  const first = families(t)[0]?.toLowerCase();
  return first !== undefined && HF_GENERIC.has(first) ? `Inter, ${t}` : undefined;
}

/** The families of a `font` shorthand: what follows its size (and line height). */
function shorthandFamilies(v: string): string | undefined {
  const m = /(?:^|\s)(?:[\d.]+(?:px|em|rem|%|pt|vw|vh|vmin|vmax|ch|ex)|xx-small|x-small|small|medium|large|x-large|xx-large|smaller|larger)(?:\s*\/\s*\S+)?\s+(.+)$/i.exec(v.trim());
  return m?.[1];
}

function expand(v: string, vars: Map<string, string>): string {
  let out = v;
  for (let i = 0; i < 8 && out.includes("var("); i++) {
    out = out.replace(/var\(\s*(--[-\w]+)\s*(?:,([^()]*))?\)/g, (_m, n: string, fb?: string) => vars.get(n) ?? fb?.trim() ?? "");
  }
  return out;
}

type Face = { weight: string; style: string; url: string; range?: string };
function faceRule(family: string, f: Face): string {
  return `@font-face{font-family:"${family.replace(/"/g, '\\"')}";src:url("${f.url}");font-weight:${f.weight};font-style:${f.style}}`;
}
/** The composition format's bundledFamilyFaceRules supplement: the faces collapsed into weight runs of one file (Google serves a
 *  variable family's weights as one file) that don't span a weight the bundle covers, each a `font-weight: lo hi`
 *  range, so the file's `wght` axis answers every weight in its run. */
function runs(faces: Face[], covered: Set<string>): Face[] {
  const groups = new Map<string, Face[]>();
  for (const f of faces) {
    const k = `${f.url}\n${f.style}\n${f.range ?? ""}`;
    const g = groups.get(k);
    if (g) g.push(f);
    else groups.set(k, [f]);
  }
  const spansCovered = (a: Face, b: Face) => {
    const [lo, hi] = [Math.min(+a.weight, +b.weight), Math.max(+a.weight, +b.weight)];
    return [...covered].some((c) => {
      const [w, st] = c.split(":");
      return st === a.style && +w > lo && +w < hi;
    });
  };
  const out: { at: number; face: Face }[] = [];
  for (const g of groups.values()) {
    const sorted = g.filter((f) => Number.isFinite(+f.weight)).sort((a, b) => +a.weight - +b.weight);
    let cur: Face[] = [];
    const flush = () => {
      if (!cur.length) return;
      const first = cur[0];
      const last = cur[cur.length - 1];
      out.push({ at: Math.min(...cur.map((f) => faces.indexOf(f))), face: { ...first, weight: cur.length > 1 ? `${first.weight} ${last.weight}` : first.weight } });
      cur = [];
    };
    for (const f of sorted) {
      if (cur.length && spansCovered(cur[cur.length - 1], f)) flush();
      cur.push(f);
    }
    flush();
    for (const f of g) if (!Number.isFinite(+f.weight)) out.push({ at: faces.indexOf(f), face: f });
  }
  return out.sort((a, b) => a.at - b.at).map((r) => r.face);
}
/** A Google Fonts stylesheet's WOFF2 faces of `family`, the subset with Basic Latin only (raea-runtime-js registers a face for
 *  every character, unicode-range aside). */
function googleFaces(css: string, family: string): Face[] {
  const out: Face[] = [];
  for (const [prelude, body] of blocks(stripComments(css))) {
    if (body === undefined || !/^@font-face/i.test(prelude)) continue;
    const ds = declsOf(body);
    const get = (k: string) => ds.find(([p]) => p === k)?.[1];
    const fam = get("font-family")?.replace(/^['"]|['"]$/g, "");
    const src = get("src") ?? "";
    const url = /url\(\s*['"]?([^'")]+)['"]?\s*\)/.exec(src)?.[1];
    const range = get("unicode-range");
    if (!url || !/format\(\s*['"]woff2['"]\s*\)/.test(src) || (fam && fam.toLowerCase() !== family.toLowerCase())) continue;
    if (range && !coversLatin(range)) continue;
    out.push({ weight: (get("font-weight") ?? "400").split(/\s+/)[0], style: get("font-style") ?? "normal", url, range });
  }
  return out;
}
const googleUrl = (family: string) =>
  `https://fonts.googleapis.com/css2?family=${encodeURIComponent(family)}:ital,wght@0,100;0,200;0,300;0,400;0,500;0,600;0,700;0,800;0,900;1,400;1,700`;

export type FontEnv = {
  /** a URL's text (Google's stylesheets come back as Chrome is served them) */
  fetch: (url: string) => Promise<string | undefined>;
  /** core/fonts' folder and its WOFF2 file names (`inter-700-normal.woff2`) */
  dir?: string;
  bundled: string[];
};

/** The page's rules with system-font primaries normalised in place, and the @font-face text that pins every family
 *  its `font-family` declarations name (and `font` shorthands, for families that aren't system stand-ins).
 *  `inline`: the elements' style attributes; `at`: the page's at-rules (its own @font-face families are left alone). */
export async function deterministic(rules: Rule[], inline: string[], at: string[], env: FontEnv): Promise<string> {
  const decls: Decl[] = [...rules.filter((r) => r.layer >= 0).flatMap((r) => r.decls), ...inline.flatMap((s) => declsOf(s))];
  const vars = new Map<string, string>();
  for (const [p, v] of decls) if (p.startsWith("--")) vars.set(p, v);
  const defined = new Set([...vars].filter(([, v]) => v.trim()).map(([k]) => k));
  for (const r of rules) {
    if (r.layer < 0) continue;
    if (!r.decls.some(([p]) => p === "font-family")) continue;
    r.decls = r.decls.map((d) => {
      if (d[0] !== "font-family") return d;
      const v = systemPrimary(d[1], defined);
      return v === undefined ? d : [d[0], v, d[2]];
    });
  }
  const declared = new Set<string>();
  for (const a of at) {
    if (!/^@font-face/i.test(a)) continue;
    const f = /font-family\s*:\s*([^;}]+)/i.exec(a)?.[1];
    if (f) declared.add(families(f)[0]?.toLowerCase() ?? "");
  }
  const wanted = new Map<string, { family: string; longhand: boolean }>();
  const add = (value: string, longhand: boolean) => {
    for (const fam of families(expand(value, vars))) {
      const lower = fam.toLowerCase();
      if (GENERIC.has(lower) || lower.startsWith("var(")) continue;
      const had = wanted.get(lower);
      if (had) had.longhand ||= longhand;
      else wanted.set(lower, { family: fam, longhand });
    }
  };
  // Inter is what a generic primary became
  for (const r of rules) for (const [p, v] of r.layer >= 0 ? r.decls : []) if (p === "font-family") add(v, true);
  for (const [p, v] of decls) {
    if (p === "font-family") add(v, true);
    else if (p === "font") {
      const f = shorthandFamilies(v);
      if (f) add(f, false);
    }
  }
  const out: string[] = [];
  for (const [lower, { family, longhand }] of wanted) {
    if (declared.has(lower)) continue;
    const canonical = ALIASES[lower];
    // The HTML composition format reads `font-family` declarations only: a stand-in named only in a `font` shorthand keeps the
    // installed font, as it does in Chrome
    if (!longhand && canonical && DISPLAY[canonical].toLowerCase() !== lower) continue;
    if (canonical) {
      const covered = new Set<string>();
      const bundled: string[] = [];
      for (const f of env.bundled) {
        const m = /^(.+)-(\d+)-(normal|italic)\.woff2$/.exec(f);
        if (!m || m[1] !== canonical || !env.dir) continue;
        bundled.push(faceRule(family, { weight: m[2], style: m[3], url: `${env.dir}/${f}` }));
        covered.add(`${m[2]}:${m[3]}`);
      }
      // the canonical family's Google faces for what the bundle lacks first, the bundle's last
      const css = await env.fetch(googleUrl(DISPLAY[canonical]));
      if (css) for (const f of runs(googleFaces(css, DISPLAY[canonical]).filter((f) => !covered.has(`${f.weight}:${f.style}`)), covered)) out.push(faceRule(family, f));
      out.push(...bundled);
      continue;
    }
    const css = await env.fetch(googleUrl(family));
    const faces = css ? googleFaces(css, family) : [];
    for (const f of runs(faces, new Set())) out.push(faceRule(family, f));
  }
  return out.join("\n");
}
