// Elements and context.
import { children, useContext } from "./all";

// ---- elements

export const Fragment = Symbol.for("react.fragment");

const ELEMENT = Symbol.for("react.transitional.element");

export const PROVIDER = Symbol.for("raea.lite.provider");

export interface Element {
  $$typeof: symbol;
  type: any;
  key: string | null;
  props: any;
  ref?: any;
  /** a host element's output, the place and text context it was made for (render: an element rendered again at the
   * same place gives the same output, when nothing under it is a component) */
  _o: any;
  _s: any;
  _t: boolean;
}

export function jsx(type: any, props: any, key?: any): Element {
  const p = props ?? {};
  let ref;
  if (p.ref !== undefined && typeof type === "string") ref = p.ref;
  return { $$typeof: ELEMENT, type, key: key === undefined || key === null ? (p.key != null ? String(p.key) : null) : String(key), props: p, ref, _o: null, _s: null, _t: false };
}

export const jsxs = jsx;

export const jsxDEV = jsx;

export function createElement(type: any, config: any, ...children: any[]): Element {
  const props: any = {};
  let key: string | null = null;
  if (config) {
    for (const k in config) {
      if (k === "key") key = config.key != null ? String(config.key) : null;
      else props[k] = config[k];
    }
  }
  if (children.length === 1) props.children = children[0];
  else if (children.length > 1) props.children = children;
  return jsx(type, props, key);
}

export function isValidElement(v: any): v is Element {
  return typeof v === "object" && v !== null && v.$$typeof === ELEMENT;
}

export function cloneElement(el: Element, config?: any, ...children: any[]): Element {
  const props = { ...el.props, ...(config ?? {}) };
  delete props.key;
  if (children.length === 1) props.children = children[0];
  else if (children.length > 1) props.children = children;
  return jsx(el.type, props, config && config.key != null ? config.key : el.key);
}

function flat(children: any, out: any[] = []): any[] {
  if (Array.isArray(children)) {
    for (const c of children) flat(c, out);
  } else if (children !== undefined && children !== null && typeof children !== "boolean") {
    out.push(children);
  }
  return out;
}

export const Children = {
  toArray: (c: any) => flat(c),
  map: (c: any, f: (x: any, i: number) => any) => (c === undefined || c === null ? c : flat(c).map(f)),
  forEach: (c: any, f: (x: any, i: number) => void) => {
    flat(c).forEach(f);
  },
  count: (c: any) => flat(c).length,
  only: (c: any) => {
    const a = flat(c);
    if (a.length !== 1 || !isValidElement(a[0])) throw new Error("Children.only expected one element");
    return a[0];
  },
};

// ---- context

export interface Context<T> {
  Provider: any;
  Consumer: any;
  _default: T;
  _id: symbol;
}

export function createContext<T>(defaultValue: T): Context<T> {
  const ctx: any = { _default: defaultValue, _id: Symbol("context") };
  ctx.Provider = { $$provider: PROVIDER, ctx };
  ctx.Consumer = (props: any) => props.children(useContext(ctx));
  return ctx;
}
