// Portions ported from HyperFrames (Apache-2.0, https://github.com/heygen-com/hyperframes); see NOTICE.
// The page's scripts: their scope (document, gsap) and named access.
import { type Script } from "../compile";
import { abs, g, isRemote, pagePath, read, warn } from "./all";

// ---------------------------------------------------------------- scripts

/** The original runtime's scoped `document` and `gsap` for a sub-composition's own script: selectors find its elements first. */
function scope(compId: string): { document: Document; gsap: any } {
  const root = Array.from(document.querySelectorAll("[data-composition-id]")).find((e) => e.getAttribute("data-composition-id") === compId);
  if (!root) return { document, gsap: g.gsap };
  const scopedAll = (sel: string): Element[] => {
    const out: Element[] = [];
    try {
      if (root.matches(sel)) out.push(root);
    } catch {}
    try {
      out.push(...Array.from(root.querySelectorAll(sel)));
    } catch {}
    return out.length ? out : Array.from(document.querySelectorAll(sel));
  };
  const doc = new Proxy(document, {
    get(t, p) {
      if (p === "querySelector") return (s: string) => scopedAll(s)[0] ?? null;
      if (p === "querySelectorAll") return (s: string) => scopedAll(s);
      if (p === "getElementsByClassName") return (c: string) => scopedAll(`.${String(c).trim().split(/\s+/).join(".")}`);
      if (p === "getElementById")
        return (id: string) => {
          const found = document.getElementById(id);
          for (let a: Node | null = found; a; a = a.parentNode) if (a === root) return found;
          if (root.getAttribute("data-hf-authored-id") === id || root.id === id) return root;
          const esc = String(id).replace(/["\\]/g, "\\$&");
          return root.querySelector(`[data-hf-authored-id="${esc}"]`) ?? root.querySelector(`[id="${esc}"]`);
        };
      if (p === "currentScript") return root;
      const v = (t as any)[p];
      return typeof v === "function" ? v.bind(t) : v;
    },
  });
  const gs = g.gsap;
  if (!gs) return { document: doc, gsap: gs };
  const targets = (t: unknown) => (typeof t === "string" ? scopedAll(t) : t);
  const wrapTl = (tl: any) => {
    for (const k of ["to", "from", "fromTo", "set"]) {
      const f = tl[k];
      tl[k] = (t: unknown, ...a: unknown[]) => f.call(tl, targets(t), ...a);
    }
    return tl;
  };
  const scoped = Object.create(gs);
  for (const k of ["to", "from", "fromTo", "set", "getProperty", "quickSetter", "quickTo"]) scoped[k] = (t: unknown, ...a: unknown[]) => gs[k](targets(t), ...a);
  scoped.timeline = (...a: unknown[]) => wrapTl(gs.timeline(...a));
  scoped.utils = Object.create(gs.utils);
  scoped.utils.toArray = (t: unknown) => (typeof t === "string" ? scopedAll(t) : gs.utils.toArray(t));
  return { document: doc, gsap: scoped };
}

/** A browser's named access on window: an element's id is a global naming it (`target.style.rotate = "0deg"`, as
 *  WPT's scripts write), unless a global of that name exists; a script's own `var` of the name takes it over. */
export function namedAccess() {
  for (const el of Array.from(document.querySelectorAll("[id]"))) {
    const id = el.getAttribute("id") ?? "";
    if (!/^[A-Za-z_$][\w$]*$/.test(id) || id in g) continue;
    Object.defineProperty(g, id, {
      configurable: true,
      enumerable: false,
      get: () => document.getElementById(id),
      set: (v) => Object.defineProperty(g, id, { value: v, writable: true, configurable: true, enumerable: true }),
    });
  }
}

export async function runScript(s: Script, i: number) {
  let code = s.text;
  let origin = `inline #${i}`;
  if (s.src) {
    origin = s.src;
    const got = await read(isRemote(s.src) ? abs(s.src) : pagePath(s.src));
    if (got?.text === undefined) return warn(`script ${s.src} not loaded`);
    code = got.text;
  }
  if (!code.trim()) return;
  try {
    if (s.scope !== undefined) {
      g.__hfScope = scope;
      (0, eval)(`(function(){var __s=__hfScope(${JSON.stringify(s.scope)});(function(document,gsap){\n${code}\n}).call(window,__s.document,__s.gsap);})();`);
    } else (0, eval)(code);
  } catch (e) {
    warn(`script ${origin} threw: ${(e as Error)?.stack ?? e}`);
  }
  // GSAP's ticker on the page clock, its jumps taken whole
  if (g.gsap?.ticker && !g.__hfLag) {
    g.__hfLag = true;
    g.gsap.ticker.lagSmoothing(0);
  }
}
