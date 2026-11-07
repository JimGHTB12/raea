// troika's Text (drei's <Text>, and anything else built on troika-three-text) as raea's <RichText> (docs/r3f.feature
// R3F13): raea lays out and draws the words itself, with the face troika was given found by its family name, so the
// SDF atlas, the worker and troika's shader never run. In a native render the Text is troika.ts's stand-in, which
// holds the same settings; in a browser it's troika's own, read the same way.
//
// What's carried: the text, the size (fontSize is the em), the face's family and weight, the colour and opacity, the
// line height, maxWidth, textAlign, and the anchors (anchorX left/center/right as RichText's align, anchorY as an
// offset from the block's middle). Lit when troika's material is (a MeshStandardMaterial given as `material`),
// unlit as troika's own MeshBasicMaterial. Outlines, strokes, curveRadius and colorRanges are left out, named once.
import * as THREE from "three";
import type { Part, Props } from "./material";

/** a troika Text, real or stood in */
export interface TroikaText extends THREE.Mesh {
  isTroikaText: true;
  text: string;
  font: string | null;
  fontSize: number;
  fontWeight: number | string;
  fontStyle: string;
  color: THREE.ColorRepresentation | null;
  anchorX: number | string;
  anchorY: number | string;
  lineHeight: number | string;
  maxWidth: number;
  textAlign: string;
  fillOpacity: number;
  outlineWidth: number | string;
  strokeWidth: number | string;
  curveRadius: number;
  colorRanges: unknown;
}

export const isTroikaText = (o: THREE.Object3D): o is TroikaText => (o as Partial<TroikaText>).isTroikaText === true && typeof (o as TroikaText).text === "string";

const warned = new Set<string>();
function warnOnce(k: string, why: string) {
  if (warned.has(k)) return;
  warned.add(k);
  console.warn(`raea/r3f: ${k} ${why}`);
}

const families = new Map<string, string>();

/** a font file's family (its name table's typographic family, else its family), read once */
function familyOf(src: string): string {
  const had = families.get(src);
  if (had) return had;
  let family: string | undefined;
  const call = (globalThis as { __raea_call?: (n: string, a: string) => string | undefined }).__raea_call;
  if (typeof call === "function" && !src.startsWith("data:")) {
    try {
      const out = call("asset.read", JSON.stringify({ path: src }));
      const b64 = out ? (JSON.parse(out) as { base64?: string }).base64 : undefined;
      if (b64) family = nameOf(Uint8Array.from(atob(b64), (c) => c.charCodeAt(0)));
    } catch {
      // not readable here: the file's own name below
    }
  }
  // Inter-400.ttf, Roboto_Bold.woff: the name before its style
  family ??= (src.split(/[?#]/)[0].split("/").pop() ?? "").replace(/\.\w+$/, "").split(/[-_]/)[0] || "Inter";
  families.set(src, family);
  return family;
}

/** an OpenType font's family from its name table (16, else 1; Windows' UTF-16 or Mac's Roman), undefined for a WOFF2 */
function nameOf(b: Uint8Array): string | undefined {
  const dv = new DataView(b.buffer, b.byteOffset, b.byteLength);
  if (b.length < 12) return undefined;
  const tables = dv.getUint16(4);
  for (let i = 0; i < tables; i++) {
    const at = 12 + i * 16;
    if (String.fromCharCode(b[at], b[at + 1], b[at + 2], b[at + 3]) !== "name") continue;
    const off = dv.getUint32(at + 8);
    const count = dv.getUint16(off + 2), strings = off + dv.getUint16(off + 4);
    const found: Record<number, string> = {};
    for (let r = 0; r < count; r++) {
      const rec = off + 6 + r * 12;
      const platform = dv.getUint16(rec), id = dv.getUint16(rec + 6), len = dv.getUint16(rec + 8), o = strings + dv.getUint16(rec + 10);
      if ((id !== 1 && id !== 16) || found[id]) continue;
      if (platform === 3 || platform === 0) {
        let s = "";
        for (let k = 0; k + 1 < len; k += 2) s += String.fromCharCode(dv.getUint16(o + k));
        found[id] = s;
      } else if (platform === 1) found[id] = String.fromCharCode(...b.subarray(o, o + len));
    }
    return found[16] || found[1];
  }
  return undefined;
}

const weightOf = (w: number | string) => (typeof w === "number" ? w : w === "bold" ? 700 : /^\d+$/.test(w) ? Number(w) : 400);

const hex = (c: THREE.Color) => `#${c.getHexString()}`;

/** the Text as a RichText in its own place (the walk puts its transform on a group around it) */
export function textParts(t: TroikaText, name: string): Part[] {
  const words = t.text;
  if (!words.trim()) return [];
  for (const [k, on] of [["outlineWidth", t.outlineWidth !== 0], ["strokeWidth", t.strokeWidth !== 0], ["curveRadius", t.curveRadius !== 0], ["colorRanges", t.colorRanges != null]] as const) {
    if (on) warnOnce(`<Text> ${k}`, "isn't drawn by raea's text: left out");
  }
  const mat = (Array.isArray(t.material) ? t.material[0] : t.material) as THREE.MeshBasicMaterial | THREE.MeshStandardMaterial;
  const colour = t.color != null ? new THREE.Color(t.color) : mat?.color ?? new THREE.Color(1, 1, 1);
  const size = t.fontSize;
  const lineHeight = typeof t.lineHeight === "number" ? t.lineHeight : 1.2;
  const props: Props = { name, size, color: hex(colour), lineHeight, weight: weightOf(t.fontWeight), unlit: !(mat as THREE.MeshStandardMaterial)?.isMeshStandardMaterial, castShadow: t.castShadow, receiveShadow: t.receiveShadow };
  if (t.font) props.font = familyOf(t.font);
  if (t.fontStyle === "italic") props.italic = true;
  const opacity = t.fillOpacity * (mat?.opacity ?? 1);
  if (opacity < 1) props.opacity = opacity;
  if (Number.isFinite(t.maxWidth)) props.maxWidth = t.maxWidth;
  // anchorX: RichText's origin is the lines' left, middle or right; a number or a percentage shifts from the left
  const ax = t.anchorX;
  const pos: [number, number, number] = [0, 0, 0];
  if (ax === "center" || ax === "right" || ax === "left") props.align = ax;
  else {
    props.align = "left";
    pos[0] = -(typeof ax === "number" ? ax : 0);
    if (typeof ax === "string" && ax.endsWith("%")) warnOnce("<Text> anchorX as a percentage", "is drawn from the left edge");
  }
  // anchorY: RichText's origin is the block's middle; the block is its lines' height
  const lines = words.split("\n").length;
  const half = (lines * lineHeight * size) / 2;
  const ay = t.anchorY;
  if (ay === "top" || ay === "top-baseline") pos[1] = -half;
  else if (ay === "bottom" || ay === "bottom-baseline") pos[1] = half;
  else if (typeof ay === "number") pos[1] = -half - ay;
  if (pos[0] || pos[1]) props.position = pos;
  props.children = words;
  return [{ type: "RichText", props }];
}
