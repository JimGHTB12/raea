// troika-three-text as raea's bundler gives it to a native bundle (engine/bundle's stand-ins): troika lays text out
// in a web worker, fetches its fonts by XMLHttpRequest and draws an SDF atlas on a canvas, none of which a native
// render's JS has, and raea draws text itself. So its Text here is a mesh that holds the same settings (text, font,
// fontSize, anchors, colour ...) and draws nothing; sync() and preloadFont() answer at once. text.ts reads it, as it
// reads troika's own Text in a browser, as raea's <RichText>. drei's <Text> and anything else built on troika's Text
// work unchanged.
import * as THREE from "three";

type Done = () => void;

/** troika's Text: its settings, drawn by raea (text.ts) */
export class Text extends THREE.Mesh {
  readonly isTroikaText = true;
  text = "";
  anchorX: number | string = 0;
  anchorY: number | string = 0;
  curveRadius = 0;
  direction = "auto";
  font: string | null = null;
  unicodeFontsURL: string | null = null;
  fontSize = 0.1;
  fontWeight: number | string = "normal";
  fontStyle = "normal";
  lang: string | null = null;
  letterSpacing = 0;
  lineHeight: number | string = "normal";
  maxWidth = Infinity;
  overflowWrap = "normal";
  textAlign = "left";
  textIndent = 0;
  whiteSpace = "normal";
  color: THREE.ColorRepresentation | null = null;
  colorRanges: unknown = null;
  outlineWidth: number | string = 0;
  outlineColor: THREE.ColorRepresentation = 0x000000;
  outlineOpacity = 1;
  outlineBlur: number | string = 0;
  outlineOffsetX: number | string = 0;
  outlineOffsetY: number | string = 0;
  strokeWidth: number | string = 0;
  strokeColor: THREE.ColorRepresentation = 0x808080;
  strokeOpacity = 1;
  fillOpacity = 1;
  depthOffset = 0;
  clipRect: unknown = null;
  orientation = "+x+y";
  glyphGeometryDetail = 1;
  sdfGlyphSize: number | null = null;
  gpuAccelerateSDF = true;
  debugSDF = false;
  textRenderInfo: unknown = null;

  constructor() {
    super(new THREE.BufferGeometry(), new THREE.MeshBasicMaterial({ color: 0xffffff, side: THREE.DoubleSide, transparent: true }));
  }

  /** laid out at once: raea lays the text out as it draws it */
  sync(callback?: Done) {
    callback?.();
  }

  dispose() {
    this.geometry.dispose();
  }

  localPositionToTextCoords(position: THREE.Vector3, target = new THREE.Vector2()) {
    return target.set(position.x, position.y);
  }

  worldPositionToTextCoords(position: THREE.Vector3, target = new THREE.Vector2()) {
    const p = this.worldToLocal(position.clone());
    return target.set(p.x, p.y);
  }
}

/** troika's BatchedText: a group of Texts, each drawn as its own */
export class BatchedText extends THREE.Group {
  readonly isBatchedText = true;
  addText(t: Text) {
    this.add(t);
  }
  removeText(t: Text) {
    this.remove(t);
  }
  sync(callback?: Done) {
    callback?.();
  }
}

/** the font is raea's to load: nothing to wait for */
export function preloadFont(_o: { font?: string; characters?: string | string[]; sdfGlyphSize?: number }, callback?: Done) {
  callback?.();
}

export function configureTextBuilder(_config: Record<string, unknown>) {}

export function getTextRenderInfo(_args: unknown, callback?: (info: unknown) => void) {
  callback?.({ parameters: _args, sdfTexture: null, glyphBounds: new Float32Array(), blockBounds: [0, 0, 0, 0], visibleBounds: [0, 0, 0, 0] });
}

export function getCaretAtPoint() {
  return null;
}

export function getSelectionRects() {
  return [];
}

export function createTextDerivedMaterial(base: THREE.Material) {
  return base;
}

export function dumpSDFTextures() {}

export const fontResolverWorkerModule = null;
export const typesetterWorkerModule = null;
