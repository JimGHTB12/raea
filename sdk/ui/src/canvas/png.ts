// A canvas's pixels as a PNG data URL (toDataURL, and drawImage with a canvas as its source): RGBA rows in stored
// (uncompressed) deflate blocks. Big, but exact and quick to write; the host decodes it like any PNG.

const CRC = (() => {
  const t = new Uint32Array(256);
  for (let n = 0; n < 256; n++) {
    let c = n;
    for (let k = 0; k < 8; k++) c = c & 1 ? 0xedb88320 ^ (c >>> 1) : c >>> 1;
    t[n] = c >>> 0;
  }
  return t;
})();

function crc32(b: Uint8Array, from: number, to: number) {
  let c = 0xffffffff;
  for (let i = from; i < to; i++) c = CRC[(c ^ b[i]) & 0xff] ^ (c >>> 8);
  return (c ^ 0xffffffff) >>> 0;
}

const B64 = "ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";

/** base64 without btoa (the host's engines may not have it). */
export function base64(b: Uint8Array): string {
  const out: string[] = [];
  let line = "";
  for (let i = 0; i < b.length; i += 3) {
    const n = (b[i] << 16) | ((b[i + 1] ?? 0) << 8) | (b[i + 2] ?? 0);
    line += B64[(n >> 18) & 63] + B64[(n >> 12) & 63] + (i + 1 < b.length ? B64[(n >> 6) & 63] : "=") + (i + 2 < b.length ? B64[n & 63] : "=");
    if (line.length >= 8192) {
      out.push(line);
      line = "";
    }
  }
  out.push(line);
  return out.join("");
}

/** w×h RGBA (straight alpha, as ImageData holds it) as PNG bytes. */
export function encodePng(w: number, h: number, rgba: Uint8ClampedArray | Uint8Array): Uint8Array {
  const row = w * 4 + 1;
  const raw = row * h;
  // zlib: header, stored blocks of at most 65535 bytes, adler32
  const blocks = Math.max(1, Math.ceil(raw / 65535));
  const zlen = 2 + raw + blocks * 5 + 4;
  const out = new Uint8Array(8 + 25 + 12 + zlen + 12);
  const dv = new DataView(out.buffer);
  out.set([137, 80, 78, 71, 13, 10, 26, 10], 0);
  let p = 8;
  const chunk = (type: string, len: number, body: (at: number) => void) => {
    dv.setUint32(p, len);
    for (let i = 0; i < 4; i++) out[p + 4 + i] = type.charCodeAt(i);
    body(p + 8);
    dv.setUint32(p + 8 + len, crc32(out, p + 4, p + 8 + len));
    p += 12 + len;
  };
  chunk("IHDR", 13, (at) => {
    dv.setUint32(at, w);
    dv.setUint32(at + 4, h);
    out.set([8, 6, 0, 0, 0], at + 8);
  });
  chunk("IDAT", zlen, (at) => {
    out[at] = 0x78;
    out[at + 1] = 0x01;
    let q = at + 2;
    let a = 1;
    let b = 0;
    let left = raw;
    let i = 0; // index into the filtered stream (a 0 filter byte, then the row)
    while (left > 0 || q === at + 2) {
      const n = Math.min(65535, left);
      out[q] = left - n === 0 ? 1 : 0;
      out[q + 1] = n & 0xff;
      out[q + 2] = n >> 8;
      out[q + 3] = ~n & 0xff;
      out[q + 4] = (~n >> 8) & 0xff;
      q += 5;
      for (let k = 0; k < n; k++, i++) {
        const y = Math.floor(i / row);
        const x = i - y * row;
        const v = x === 0 ? 0 : rgba[y * w * 4 + x - 1];
        out[q++] = v;
        a = (a + v) % 65521;
        b = (b + a) % 65521;
      }
      left -= n;
      if (n === 0) break;
    }
    dv.setUint32(q, ((b << 16) | a) >>> 0);
  });
  chunk("IEND", 0, () => {});
  return out;
}
