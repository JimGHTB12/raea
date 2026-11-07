// A three.js texture as the picture raea loads: its src (an <img>'s, a loader's url) when it came from a file, a PNG
// data URL when its pixels were made in code (a DataTexture, a canvas). raea's engine loads pictures itself, so in a
// native render, where there's no DOM to load an <img> into, three's ImageLoader is stood in for by one that hands
// back the url (with the picture's size unknown) and the engine loads that file.
import * as THREE from "three";

const urls = new WeakMap<object, string>();

/** three's ImageLoader without a DOM: the image is a stand-in carrying its url (raea loads the file) */
export function standInImages() {
  const doc = (globalThis as { document?: { createElementNS?: unknown } }).document;
  if (doc && typeof doc.createElementNS === "function") return;
  const load = function (this: THREE.ImageLoader, url: string, onLoad?: (img: HTMLImageElement) => void) {
    const src = this.manager.resolveURL((this.path ?? "") + url);
    const img = { src, width: 1, height: 1, naturalWidth: 1, naturalHeight: 1, complete: true } as unknown as HTMLImageElement;
    urls.set(img, src);
    this.manager.itemStart(src);
    queueMicrotask(() => {
      onLoad?.(img);
      this.manager.itemEnd(src);
    });
    return img;
  };
  THREE.ImageLoader.prototype.load = load as unknown as THREE.ImageLoader["load"];
}

// ---- PNG, uncompressed (stored deflate blocks): small code, and the pictures made in code are small ----
const CRC = (() => {
  const t = new Uint32Array(256);
  for (let n = 0; n < 256; n++) {
    let c = n;
    for (let k = 0; k < 8; k++) c = c & 1 ? 0xedb88320 ^ (c >>> 1) : c >>> 1;
    t[n] = c >>> 0;
  }
  return t;
})();

function crc(bytes: Uint8Array): number {
  let c = 0xffffffff;
  for (const b of bytes) c = CRC[(c ^ b) & 0xff] ^ (c >>> 8);
  return (c ^ 0xffffffff) >>> 0;
}

function chunk(type: string, data: Uint8Array): Uint8Array {
  const out = new Uint8Array(12 + data.length);
  const dv = new DataView(out.buffer);
  dv.setUint32(0, data.length);
  for (let i = 0; i < 4; i++) out[4 + i] = type.charCodeAt(i);
  out.set(data, 8);
  dv.setUint32(8 + data.length, crc(out.subarray(4, 8 + data.length)));
  return out;
}

function zlibStored(raw: Uint8Array): Uint8Array {
  const blocks = Math.max(1, Math.ceil(raw.length / 65535));
  const out = new Uint8Array(2 + raw.length + blocks * 5 + 4);
  out[0] = 0x78;
  out[1] = 0x01;
  let o = 2;
  for (let b = 0; b < blocks; b++) {
    const part = raw.subarray(b * 65535, Math.min(raw.length, (b + 1) * 65535));
    out[o++] = b === blocks - 1 ? 1 : 0;
    out[o++] = part.length & 0xff;
    out[o++] = part.length >>> 8;
    out[o++] = ~part.length & 0xff;
    out[o++] = (~part.length >>> 8) & 0xff;
    out.set(part, o);
    o += part.length;
  }
  let a = 1, s = 0;
  for (const x of raw) {
    a = (a + x) % 65521;
    s = (s + a) % 65521;
  }
  new DataView(out.buffer).setUint32(o, ((s << 16) | a) >>> 0);
  return out;
}

const B64 = "ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
function base64(bytes: Uint8Array): string {
  let s = "";
  for (let i = 0; i < bytes.length; i += 3) {
    const [a, b = 0, c = 0] = [bytes[i], bytes[i + 1], bytes[i + 2]];
    s += B64[a >> 2] + B64[((a & 3) << 4) | (b >> 4)] + (i + 1 < bytes.length ? B64[((b & 15) << 2) | (c >> 6)] : "=") + (i + 2 < bytes.length ? B64[c & 63] : "=");
  }
  return s;
}

/** RGBA bytes, rows from the top, as a PNG data URL */
export function pngOf(rgba: Uint8Array, width: number, height: number): string {
  const raw = new Uint8Array((width * 4 + 1) * height);
  for (let y = 0; y < height; y++) raw.set(rgba.subarray(y * width * 4, (y + 1) * width * 4), y * (width * 4 + 1) + 1);
  const head = new Uint8Array(13);
  const dv = new DataView(head.buffer);
  dv.setUint32(0, width);
  dv.setUint32(4, height);
  head.set([8, 6, 0, 0, 0], 8);
  const sig = new Uint8Array([137, 80, 78, 71, 13, 10, 26, 10]);
  const parts = [sig, chunk("IHDR", head), chunk("IDAT", zlibStored(raw)), chunk("IEND", new Uint8Array())];
  const all = new Uint8Array(parts.reduce((n, p) => n + p.length, 0));
  let o = 0;
  for (const p of parts) {
    all.set(p, o);
    o += p.length;
  }
  return "data:image/png;base64," + base64(all);
}

/** a DataTexture's pixels as RGBA bytes, top row first (three's flipY puts its first row at the bottom) */
function bytesOf(t: THREE.DataTexture): Uint8Array | undefined {
  const { data, width, height } = t.image as { data: ArrayLike<number> | null; width: number; height: number };
  if (!data || !width || !height) return undefined;
  const per = t.format === THREE.RGBAFormat ? 4 : t.format === THREE.RGFormat ? 2 : t.format === THREE.RedFormat ? 1 : 0;
  if (!per) return undefined;
  const float = t.type === THREE.FloatType || t.type === THREE.HalfFloatType;
  const half = t.type === THREE.HalfFloatType;
  const out = new Uint8Array(width * height * 4);
  for (let y = 0; y < height; y++) {
    const from = t.flipY ? y : height - 1 - y;
    for (let x = 0; x < width; x++) {
      const i = (from * width + x) * per, o = (y * width + x) * 4;
      for (let c = 0; c < 4; c++) {
        let v = c < per ? Number(data[i + c]) : c === 3 ? (float ? 1 : 255) : per === 1 ? Number(data[i]) : 0;
        if (half && c < per) v = THREE.DataUtils.fromHalfFloat(v);
        out[o + c] = float ? Math.round(Math.min(1, Math.max(0, v)) * 255) : v;
      }
    }
  }
  return out;
}

/** a texture's picture for raea, or undefined when it has none raea can load (a render target, a video) */
export function srcOf(t: THREE.Texture | null | undefined): string | undefined {
  if (!t) return undefined;
  const given = t.userData?.src;
  if (typeof given === "string") return given;
  const img = t.image as { src?: string; toDataURL?: () => string; data?: unknown } | undefined;
  if (!img) return undefined;
  if (urls.has(img)) return urls.get(img);
  if (typeof img.src === "string" && img.src !== "") return img.src;
  if (typeof img.toDataURL === "function") {
    const s = img.toDataURL();
    urls.set(img, s);
    return s;
  }
  if ((t as THREE.DataTexture).isDataTexture) {
    const bytes = bytesOf(t as THREE.DataTexture);
    if (!bytes) return undefined;
    const { width, height } = t.image as { width: number; height: number };
    const s = pngOf(bytes, width, height);
    urls.set(img, s);
    return s;
  }
  return undefined;
}

/** a texture's repeat and offset, rotation, wrap and filter as raea's material props */
export function placementOf(t: THREE.Texture): { repeat?: [number, number]; uvTransform?: { offset?: [number, number]; rotation?: number; scale?: [number, number] }; sampler?: Record<string, unknown> } {
  const out: ReturnType<typeof placementOf> = {};
  const moved = t.offset.x !== 0 || t.offset.y !== 0 || t.rotation !== 0;
  if (moved) out.uvTransform = { offset: [t.offset.x, t.offset.y], rotation: t.rotation, scale: [t.repeat.x, t.repeat.y] };
  else if (t.repeat.x !== 1 || t.repeat.y !== 1) out.repeat = [t.repeat.x, t.repeat.y];
  const wrap = (w: number) => (w === THREE.RepeatWrapping ? "repeat" : w === THREE.MirroredRepeatWrapping ? "mirror" : "clamp");
  const nearest = t.magFilter === THREE.NearestFilter;
  const wrapS = wrap(t.wrapS), wrapT = wrap(t.wrapT);
  if (nearest || wrapS !== "clamp" || wrapT !== "clamp" || t.anisotropy > 1) {
    out.sampler = { filter: nearest ? "nearest" : "linear", wrap: wrapS === wrapT ? wrapS : [wrapS, wrapT], ...(t.anisotropy > 1 ? { anisotropy: t.anisotropy } : {}) };
  }
  return out;
}
