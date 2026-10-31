// Path2D, the canvas element's class, images and ImageData, and the globals.
import { C, CanvasHost, CanvasRenderingContext2D_, Gradient, ID, L, M, Mat, Path, Q, apply } from "./all";

/** Path2D: a path built apart from a context, in its own (untransformed) space. */
export class Path2D_ {
  private p: Path;
  constructor(from?: Path2D_) {
    this.p = new Path(() => ID);
    if (from instanceof Path2D_) this.p.d = from.p.d.slice();
  }
  toDevice(ts: Mat): number[] {
    const out: number[] = [];
    const d = this.p.d;
    for (let i = 0; i < d.length; ) {
      const op = d[i++];
      const n = op === M || op === L ? 1 : op === Q ? 2 : op === C ? 3 : 0;
      out.push(op);
      for (let j = 0; j < n; j++) out.push(...apply(ts, d[i++], d[i++]));
    }
    return out;
  }
  moveTo(x: number, y: number) { this.p.move(x, y); }
  lineTo(x: number, y: number) { this.p.line(x, y); }
  closePath() { this.p.close(); }
  quadraticCurveTo(a: number, b: number, x: number, y: number) { this.p.quad(a, b, x, y); }
  bezierCurveTo(a: number, b: number, c: number, d: number, x: number, y: number) { this.p.cubic(a, b, c, d, x, y); }
  arc(x: number, y: number, r: number, a0: number, a1: number, ccw = false) { this.p.ellipse(x, y, r, r, 0, a0, a1, ccw); }
  ellipse(x: number, y: number, rx: number, ry: number, rot: number, a0: number, a1: number, ccw = false) { this.p.ellipse(x, y, rx, ry, rot, a0, a1, ccw); }
  rect(x: number, y: number, w: number, h: number) {
    this.p.move(x, y); this.p.line(x + w, y); this.p.line(x + w, y + h); this.p.line(x, y + h); this.p.close();
  }
}

/** The canvas element on top of the DOM layer's element class (sdk/ui/src/dom.ts). */
// eslint-disable-next-line @typescript-eslint/no-explicit-any
export function canvasClass<T extends new (...a: any[]) => CanvasHost & { setAttribute(k: string, v: string): void }>(Base: T) {
  return class HTMLCanvasElement_ extends Base {
    _ctx?: CanvasRenderingContext2D_;
    get width(): number {
      const v = parseInt(this.getAttribute("width") ?? "", 10);
      return Number.isFinite(v) && v >= 0 ? v : 300;
    }
    set width(v: number) {
      this.setAttribute("width", String(Math.max(0, Math.floor(v))));
      this._ctx?._reset();
    }
    get height(): number {
      const v = parseInt(this.getAttribute("height") ?? "", 10);
      return Number.isFinite(v) && v >= 0 ? v : 150;
    }
    set height(v: number) {
      this.setAttribute("height", String(Math.max(0, Math.floor(v))));
      this._ctx?._reset();
    }
    getContext(kind: string) {
      if (kind !== "2d") return null;
      return (this._ctx ??= new CanvasRenderingContext2D_(this as unknown as CanvasHost & { width: number; height: number }));
    }
    toDataURL() {
      return "data:,";
    }
  };
}

/** The globals a page has for canvas: the context's and Path2D's classes. */
const B64 = "ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";

/** A data: URL's first bytes (the header is all the size needs); no atob or TextEncoder on the host's engines. */
function dataBytes(src: string, max = 4096): Uint8Array | null {
  const m = /^data:([^,]*?)(;base64)?,([\s\S]*)$/.exec(src);
  if (!m) return null;
  if (!m[2]) {
    const t = decodeURIComponent(m[3].slice(0, max * 3));
    const out = new Uint8Array(Math.min(t.length, max));
    for (let i = 0; i < out.length; i++) out[i] = t.charCodeAt(i) & 0xff;
    return out;
  }
  const out: number[] = [];
  let acc = 0;
  let bits = 0;
  for (let i = 0; i < m[3].length && out.length < max; i++) {
    const v = B64.indexOf(m[3][i]);
    if (v < 0) continue;
    acc = ((acc << 6) | v) & 0xffffff;
    bits += 6;
    if (bits >= 8) {
      bits -= 8;
      out.push((acc >> bits) & 0xff);
    }
  }
  return Uint8Array.from(out);
}

/** A picture's size from its header: PNG, GIF, JPEG, WebP, BMP, svg (null: unknown). */
export function sniffSize(src: string): [number, number] | null {
  const b = dataBytes(src);
  if (!b || b.length < 26) return null;
  const be16 = (i: number) => (b[i] << 8) | b[i + 1];
  const le16 = (i: number) => b[i] | (b[i + 1] << 8);
  const be32 = (i: number) => ((b[i] << 24) | (b[i + 1] << 16) | (b[i + 2] << 8) | b[i + 3]) >>> 0;
  const le32 = (i: number) => (b[i] | (b[i + 1] << 8) | (b[i + 2] << 16) | (b[i + 3] << 24)) >>> 0;
  if (b[0] === 0x89 && b[1] === 0x50) return [be32(16), be32(20)];
  if (b[0] === 0x47 && b[1] === 0x49) return [le16(6), le16(8)];
  if (b[0] === 0x42 && b[1] === 0x4d) return [le32(18), Math.abs(le32(22) | 0)];
  if (b[0] === 0x52 && b[8] === 0x57) {
    const tag = String.fromCharCode(b[12], b[13], b[14], b[15]);
    if (tag === "VP8 ") return [le16(26) & 0x3fff, le16(28) & 0x3fff];
    if (tag === "VP8L") return [1 + (((b[22] & 0x3f) << 8) | b[21]), 1 + (((b[24] & 0xf) << 10) | (b[23] << 2) | ((b[22] & 0xc0) >> 6))];
    if (tag === "VP8X") return [1 + (b[24] | (b[25] << 8) | (b[26] << 16)), 1 + (b[27] | (b[28] << 8) | (b[29] << 16))];
  }
  if (b[0] === 0xff && b[1] === 0xd8) {
    let i = 2;
    while (i + 9 < b.length) {
      if (b[i] !== 0xff) return null;
      const mk = b[i + 1];
      if (mk >= 0xc0 && mk <= 0xcf && mk !== 0xc4 && mk !== 0xc8 && mk !== 0xcc) return [be16(i + 7), be16(i + 5)];
      i += 2 + be16(i + 2);
    }
    return null;
  }
  let text = "";
  for (let i = 0; i < b.length; i++) text += String.fromCharCode(b[i]);
  const svg = /<svg\b[^>]*>/.exec(text);
  if (svg) {
    const num = (k: string) => {
      const v = new RegExp(`\\s${k}\\s*=\\s*["']([\\d.]+)(px)?["']`).exec(svg[0]);
      return v ? parseFloat(v[1]) : null;
    };
    const vb = /viewBox\s*=\s*["']\s*[-\d.]+[\s,]+[-\d.]+[\s,]+([\d.]+)[\s,]+([\d.]+)/.exec(svg[0]);
    const w = num("width") ?? (vb ? parseFloat(vb[1]) : 300);
    const h = num("height") ?? (vb ? parseFloat(vb[2]) : 150);
    return [w, h];
  }
  return null;
}

/** `new Image()`: an img element that isn't on the page. Its picture is decoded where it's drawn (drawImage, on the
 * host); here, a data: URL's size comes from its header and `load` (or `error`) fires a task later, as in a browser. */
function makeImage(): unknown {
  return function Image(this: unknown, w?: number, h?: number) {
    const el = (globalThis as unknown as { document: { createElement(t: string): Record<string, unknown> & { setAttribute(k: string, v: string): void; getAttribute(k: string): string | null; dispatchEvent?(e: unknown): void } } }).document.createElement("img");
    if (w !== undefined) el.setAttribute("width", String(w));
    if (h !== undefined) el.setAttribute("height", String(h));
    let size: [number, number] | null = null;
    let done = false;
    const fire = (type: "load" | "error") => {
      done = true;
      // (the DOM's dispatch calls its onload or onerror too; without one, the handler is called here)
      const E = (globalThis as unknown as { Event?: new (t: string) => unknown }).Event;
      if (E && el.dispatchEvent) {
        try {
          el.dispatchEvent(new E(type));
        } catch {}
        return;
      }
      const h = el["on" + type] as ((e: unknown) => void) | null | undefined;
      if (typeof h === "function") h.call(el, { type, target: el, currentTarget: el });
    };
    Object.defineProperty(el, "src", {
      configurable: true,
      get: () => el.getAttribute("src") ?? "",
      set: (v: string) => {
        el.setAttribute("src", String(v));
        done = false;
        size = sniffSize(String(v));
        const s = size;
        setTimeout(() => fire(s ? "load" : "error"), 0);
      },
    });
    Object.defineProperty(el, "currentSrc", { configurable: true, get: () => (done && size ? el.getAttribute("src") ?? "" : "") });
    Object.defineProperty(el, "naturalWidth", { configurable: true, get: () => (done && size ? size[0] : 0) });
    Object.defineProperty(el, "naturalHeight", { configurable: true, get: () => (done && size ? size[1] : 0) });
    Object.defineProperty(el, "complete", { configurable: true, get: () => done || !el.getAttribute("src") });
    Object.defineProperty(el, "width", { configurable: true, get: () => Number(el.getAttribute("width") ?? (size ? size[0] : 0)), set: (v: number) => el.setAttribute("width", String(v)) });
    Object.defineProperty(el, "height", { configurable: true, get: () => Number(el.getAttribute("height") ?? (size ? size[1] : 0)), set: (v: number) => el.setAttribute("height", String(v)) });
    el.decode = () => new Promise<void>((res, rej) => (done ? (size ? res() : rej(new Error("decode"))) : setTimeout(() => (size ? res() : rej(new Error("decode"))), 0)));
    return el;
  };
}

/** ImageData: RGBA bytes, not premultiplied, row by row (getImageData's answer, or made empty). */
export class ImageData_ {
  readonly data: Uint8ClampedArray;
  readonly width: number;
  readonly height: number;
  readonly colorSpace = "srgb";
  constructor(a: number | Uint8ClampedArray, b: number, c?: number) {
    if (a instanceof Uint8ClampedArray) {
      if (!a.length || a.length % 4) throw new DOMException("Failed to construct 'ImageData': The input data length is not a multiple of 4.", "InvalidStateError");
      const h = c ?? a.length / 4 / b;
      if (!b || a.length !== b * h * 4 || !Number.isInteger(h)) throw new DOMException("Failed to construct 'ImageData': The input data length is not a multiple of (4 * width).", "IndexSizeError");
      [this.data, this.width, this.height] = [a, b, h];
    } else {
      if (!a || !b) throw new DOMException(`Failed to construct 'ImageData': The source ${a ? "height" : "width"} is zero or not a number.`, "IndexSizeError");
      [this.data, this.width, this.height] = [new Uint8ClampedArray(a * b * 4), a, b];
    }
  }
}

/** base64 into `out`, as many bytes as fit. */
export function unb64(s: string, out: Uint8ClampedArray) {
  let acc = 0;
  let bits = 0;
  let o = 0;
  for (let i = 0; i < s.length && o < out.length; i++) {
    const c = s.charCodeAt(i);
    const v = c >= 65 && c <= 90 ? c - 65 : c >= 97 && c <= 122 ? c - 71 : c >= 48 && c <= 57 ? c + 4 : c === 43 ? 62 : c === 47 ? 63 : -1;
    if (v < 0) continue;
    acc = ((acc << 6) | v) & 0xffffff;
    bits += 6;
    if (bits >= 8) {
      bits -= 8;
      out[o++] = (acc >> bits) & 0xff;
    }
  }
}

export function canvasGlobals(): Record<string, unknown> {
  return { CanvasRenderingContext2D: CanvasRenderingContext2D_, Path2D: Path2D_, CanvasGradient: Gradient, ImageData: ImageData_, Image: makeImage() };
}
