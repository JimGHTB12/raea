// bevy_feathers' colour plane (color_plane.rs, color_plane.wesl; docs/control.feature FC0a): two channels of a colour
// across and up a box, the third fixed. Bevy draws the plane with a UiMaterial running color_plane.wesl per pixel;
// raea has no UiMaterial yet (FC2), so here the same formulas fill a bitmap (a PNG data URL, stored deflate) that the
// inner node stretches over itself. The thumb is placed as update_plane_thumb_position places it after layout.
import { useContext, useMemo, type ReactNode } from "react";
import { InUi, Node, UiScaleCx, boxOf, nodeCss, type NodeProps } from "../ui";
import { Srgba } from "../colors";
import { Hsla } from "../colorspaces";
import { Okhsla } from "../okhsla";
import { BorderRadius, UiRect, percent, px } from "../ui-val";
import { UiTransform, Val2 } from "../ui-transform";
import { useThemeColor } from "./theme";
import { tokens } from "./tokens";

/** which two channels a plane spans, x then y: FeathersColorPlane's variants */
type ColorPlane = "RedGreen" | "RedBlue" | "GreenBlue" | "HueSaturation" | "HueLightness" | "OkhslHueSaturation" | "OkhslHueLightness";
const PLANES: readonly ColorPlane[] = ["RedGreen", "RedBlue", "GreenBlue", "HueSaturation", "HueLightness", "OkhslHueSaturation", "OkhslHueLightness"];

/** color_plane.wesl: the sRGB colour at (u, v), v down from the top, `z` the fixed channel (a hue in turns) */
function planeColor(plane: ColorPlane, u: number, v: number, z: number): Srgba {
  const up = 1 - v;
  switch (plane) {
    case "RedGreen": return new Srgba(u, up, z);
    case "RedBlue": return new Srgba(u, z, up);
    case "GreenBlue": return new Srgba(z, u, up);
    case "HueSaturation": return new Hsla(u * 360, up, z).to_srgba();
    case "HueLightness": return new Hsla(u * 360, z, up).to_srgba();
    case "OkhslHueSaturation": return okhsl(u, up, z);
    case "OkhslHueLightness": return okhsl(u, z, up);
  }
}

/** okhsl_to_linear_rgb, as an sRGB colour: white at full lightness, black at none, as the shader's early returns */
function okhsl(h: number, s: number, l: number): Srgba {
  if (l >= 1) return new Srgba(1, 1, 1);
  if (l <= 0) return new Srgba(0, 0, 0);
  return new Okhsla(h * 360, s, l).to_srgba();
}

const W = 384;
const H = 96;

const CRC = (() => {
  const t = new Uint32Array(256);
  for (let n = 0; n < 256; n++) {
    let c = n;
    for (let k = 0; k < 8; k++) c = c & 1 ? 0xedb88320 ^ (c >>> 1) : c >>> 1;
    t[n] = c >>> 0;
  }
  return t;
})();
const B64 = "ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";

/** w×h RGB rows as a PNG data URL: stored deflate blocks, exact and quick to write */
function pngUrl(w: number, h: number, rgb: Uint8Array): string {
  const row = w * 3 + 1;
  const raw = new Uint8Array(row * h);
  for (let y = 0; y < h; y++) raw.set(rgb.subarray(y * w * 3, (y + 1) * w * 3), y * row + 1);
  const blocks = Math.max(1, Math.ceil(raw.length / 65535));
  const z = new Uint8Array(2 + raw.length + blocks * 5 + 4);
  z[0] = 0x78; z[1] = 0x01;
  let q = 2;
  for (let i = 0; i < blocks; i++) {
    const n = Math.min(65535, raw.length - i * 65535);
    z.set([i === blocks - 1 ? 1 : 0, n & 0xff, n >> 8, ~n & 0xff, (~n >> 8) & 0xff], q);
    z.set(raw.subarray(i * 65535, i * 65535 + n), q + 5);
    q += 5 + n;
  }
  let a = 1, b = 0;
  for (const x of raw) { a = (a + x) % 65521; b = (b + a) % 65521; }
  new DataView(z.buffer).setUint32(q, ((b << 16) | a) >>> 0);
  const parts: Uint8Array[] = [new Uint8Array([137, 80, 78, 71, 13, 10, 26, 10])];
  const chunk = (type: string, body: Uint8Array) => {
    const c = new Uint8Array(12 + body.length);
    const dv = new DataView(c.buffer);
    dv.setUint32(0, body.length);
    for (let i = 0; i < 4; i++) c[4 + i] = type.charCodeAt(i);
    c.set(body, 8);
    let crc = 0xffffffff;
    for (let i = 4; i < 8 + body.length; i++) crc = CRC[(crc ^ c[i]) & 0xff] ^ (crc >>> 8);
    dv.setUint32(8 + body.length, (crc ^ 0xffffffff) >>> 0);
    parts.push(c);
  };
  const head = new Uint8Array(13);
  new DataView(head.buffer).setUint32(0, w);
  new DataView(head.buffer).setUint32(4, h);
  head.set([8, 2, 0, 0, 0], 8);
  chunk("IHDR", head);
  chunk("IDAT", z);
  chunk("IEND", new Uint8Array());
  const all = new Uint8Array(parts.reduce((n, p) => n + p.length, 0));
  let o = 0;
  for (const p of parts) { all.set(p, o); o += p.length; }
  let s = "";
  for (let i = 0; i < all.length; i += 3) {
    const n = (all[i] << 16) | ((all[i + 1] ?? 0) << 8) | (all[i + 2] ?? 0);
    s += B64[(n >> 18) & 63] + B64[(n >> 12) & 63] + (i + 1 < all.length ? B64[(n >> 6) & 63] : "=") + (i + 2 < all.length ? B64[n & 63] : "=");
  }
  return `data:image/png;base64,${s}`;
}

/** the plane's pixels, each sampled at its centre */
function planeUrl(plane: ColorPlane, z: number): string {
  const rgb = new Uint8Array(W * H * 3);
  const byte = (c: number) => Math.round(Math.max(0, Math.min(1, c)) * 255);
  for (let y = 0; y < H; y++) {
    for (let x = 0; x < W; x++) {
      const c = planeColor(plane, (x + 0.5) / W, (y + 0.5) / H, z);
      rgb.set([byte(c.red), byte(c.green), byte(c.blue)], (y * W + x) * 3);
    }
  }
  return pngUrl(W, H, rgb);
}

/** the inner node: the plane's bitmap stretched over it, as the MaterialNode fills it */
function PlaneFill({ url, children, ...p }: NodeProps & { url: string; children?: ReactNode }) {
  const ui = useContext(InUi);
  const n = nodeCss("FeathersColorPlane", p, ui?.back, useContext(UiScaleCx));
  return boxOf({ ...n.css, backgroundImage: `url("${url}")`, backgroundSize: "100% 100%", backgroundRepeat: "no-repeat" }, undefined, children, ui === null, undefined, n.attrs);
}

export interface FeathersColorPlaneProps extends NodeProps {
  /** the two channels it spans, x then y (the FeathersColorPlane variant): RedGreen, RedBlue, GreenBlue, HueSaturation,
   *  HueLightness, OkhslHueSaturation or OkhslHueLightness */
  plane?: ColorPlane;
  /** ColorPlaneValue: x and y the channels across and up (0 to 1, a hue in turns), z the fixed one */
  value?: [number, number, number];
}

/** A colour plane: at least 100 px high, 4 px padding, 5 px corners on COLOR_PLANE_BG, the plane filling its inside and
 *  a 10 px ringed thumb at the value. @bevy bevy_feathers::controls::FeathersColorPlane */
export function FeathersColorPlane({ plane = "RedGreen", value = [0, 0, 0], ...node }: FeathersColorPlaneProps) {
  if (!PLANES.includes(plane)) throw new TypeError(`FeathersColorPlane: plane is one of ${PLANES.join(", ")}, got ${JSON.stringify(plane)}`);
  if (!Array.isArray(value) || value.length !== 3 || !value.every(Number.isFinite)) throw new TypeError(`FeathersColorPlane: value is [x, y, z], got ${JSON.stringify(value)}`);
  const [x, y, z] = value;
  const url = useMemo(() => planeUrl(plane, z), [plane, z]);
  return (
    <Node display="Flex" min_height={px(100)} align_self="Stretch" padding={UiRect.all(px(4))} border_radius={BorderRadius.all(px(5))} background_color={useThemeColor(tokens.COLOR_PLANE_BG)} {...node}>
      <PlaneFill url={url} align_self="Stretch" flex_grow={1}>
        <Node position_type="Absolute" left={percent(x * 100)} top={percent((1 - y) * 100)} width={px(10)} height={px(10)} border={UiRect.all(px(1))} border_radius={BorderRadius.MAX} border_color={new Srgba(1, 1, 1, 1)} outline={{ width: px(1), offset: px(0), color: new Srgba(0, 0, 0, 1) }} ui_transform={UiTransform.from_translation(Val2.px(-5, -5))} />
      </PlaneFill>
    </Node>
  );
}
