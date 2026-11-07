// @react-three/postprocessing's <EffectComposer> as raea's frame effects (docs/r3f.feature R3F12). Its effects are R3F
// instances under the composer's <group>, objects of pmndrs' postprocessing library, not three's: each is read here and
// becomes the raea effect that does the same thing (Bloom, Grade, ToneMapping, DepthOfField, AntiAliasing,
// AmbientOcclusion, Look, LensDistortion), which raea's engine draws after the scene as Bevy does. One raea has nothing
// for (noise, glitch, scanlines, god rays ...) is named once and left out.
//
// A composer renders the scene into a render target, where three doesn't tone-map: the frame is mapped only by a
// <ToneMapping> effect in it (postprocessing's ToneMappingEffect), so a scene with a composer is drawn untonemapped
// unless it has one.
import type * as THREE from "three";
import type { Part, Props } from "./material";

const warned = new Set<string>();
function warnOnce(k: string, why: string) {
  if (warned.has(k)) return;
  warned.add(k);
  console.warn(`raea/r3f: ${k} ${why}`);
}

/** an effect of pmndrs' postprocessing (duck-typed: its class's name, its blend mode) */
interface Effect {
  name: string;
  blendMode?: { blendFunction?: number; opacity?: { value: number } };
  [k: string]: unknown;
}

const isEffect = (o: unknown): o is Effect => typeof (o as Effect | null)?.name === "string" && (o as Effect).blendMode !== undefined && /Effect$/.test((o as Effect).name);
/** a postprocessing Pass that isn't an EffectPass (N8AO's N8AOPostPass) */
const isPass = (o: unknown): o is { name: string; constructor: { name: string } } => typeof (o as { render?: unknown } | null)?.render === "function" && (o as { isPass?: boolean }).isPass === true;

/** postprocessing's ToneMappingMode, as raea's ToneMapping mode */
const TONE: Record<number, string> = { 0: "none", 1: "reinhard", 2: "reinhard", 3: "reinhard", 4: "aces", 5: "aces", 6: "aces", 7: "agx", 8: "neutral" };

const num = (v: unknown, d: number) => (typeof v === "number" && Number.isFinite(v) ? v : d);

/** the composers' effects in a scene, in the order they're drawn, and whether a composer was found */
export function composerEffects(scene: THREE.Scene): { found: boolean; effects: unknown[] } {
  const effects: unknown[] = [];
  let found = false;
  scene.traverse((o) => {
    const inst = (o as unknown as { __r3f?: { children?: { object?: unknown }[] } }).__r3f;
    for (const c of inst?.children ?? []) {
      const e = c.object;
      if (isEffect(e) || isPass(e)) {
        found = true;
        effects.push(e);
      }
    }
  });
  return { found, effects };
}

/** the effects as raea's parts: one Grade gathers the colour effects; `tone` is the ToneMapping part, if any */
export function postParts(effects: unknown[]): { parts: Part[]; tone?: Part } {
  const parts: Part[] = [];
  const grade: Props = {};
  let tone: Part | undefined;
  for (const raw of effects) {
    if (isPass(raw) && !isEffect(raw)) {
      const n = raw.constructor.name;
      if (/N8AO/i.test(n)) parts.push({ type: "AmbientOcclusion", props: { quality: "medium" } });
      else warnOnce(`<EffectComposer> pass ${n}`, "has no raea counterpart: left out");
      continue;
    }
    const e = raw as Effect;
    // a blend of SKIP (0) draws nothing; an effect's opacity scales it
    if (e.blendMode?.blendFunction === 0) continue;
    const share = num(e.blendMode?.opacity?.value, 1);
    switch (e.name) {
      case "BloomEffect":
      case "SelectiveBloomEffect": {
        if (e.name === "SelectiveBloomEffect") warnOnce("<SelectiveBloom>", "glows everything bright, not only its selection (raea's Bloom is the frame's)");
        const lum = e.luminanceMaterial as { threshold?: number } | undefined;
        parts.push({ type: "Bloom", props: { intensity: num(e.intensity, 1) * share, threshold: num(lum?.threshold, 0.9) } });
        break;
      }
      case "VignetteEffect": {
        // postprocessing's: darkness toward the corners from `offset` out (0.5, 0.5 by default)
        const darkness = num(e.darkness, 0.5) * share;
        const offset = num(e.offset, 0.5);
        grade.vignette = { intensity: Math.min(1, darkness), radius: Math.max(0.1, 1 - offset * 0.5) };
        break;
      }
      case "ChromaticAberrationEffect": {
        const o = e.offset as { x: number; y: number } | undefined;
        if (o) grade.aberration = Math.hypot(o.x, o.y) * share;
        break;
      }
      case "HueSaturationEffect": {
        grade.saturation = 1 + num(e.saturation, 0) * share;
        if (num(e.hue, 0) !== 0) warnOnce("<HueSaturation> hue", "isn't turned in raea (its saturation is)");
        break;
      }
      case "BrightnessContrastEffect": {
        grade.midtones = { contrast: 1 + num(e.contrast, 0) * share, lift: num(e.brightness, 0) * share };
        break;
      }
      case "ToneMappingEffect": {
        const mode = TONE[num(e.mode, 7)] ?? "agx";
        // three's ACES scales its input by 1/0.6 where Bevy's doesn't (lights.ts's toneOf does the same)
        tone = { type: "ToneMapping", props: mode === "aces" ? { mode, exposure: Math.log2(1 / 0.6) } : { mode } };
        break;
      }
      case "DepthOfFieldEffect": {
        const coc = e.cocMaterial as { worldFocusDistance?: number; worldFocusRange?: number } | undefined;
        const props: Props = { mode: "gaussian", maxBlur: Math.round(num(e.bokehScale, 1) * 16) };
        if (coc?.worldFocusDistance !== undefined) props.focus = coc.worldFocusDistance;
        parts.push({ type: "DepthOfField", props });
        break;
      }
      case "BokehEffect":
      case "RealisticBokehEffect": {
        const focus = num(e.focus ?? (e.uniforms as Map<string, { value: number }> | undefined)?.get?.("focus")?.value, 10);
        parts.push({ type: "DepthOfField", props: { mode: "bokeh", focus } });
        break;
      }
      case "GammaCorrectionEffect":
        break;
      case "SMAAEffect":
        parts.push({ type: "AntiAliasing", props: { mode: "smaa" } });
        break;
      case "FXAAEffect":
        parts.push({ type: "AntiAliasing", props: { mode: "fxaa" } });
        break;
      case "SSAOEffect":
        parts.push({ type: "AmbientOcclusion", props: { quality: "medium" } });
        break;
      case "PixelationEffect":
        parts.push({ type: "Look", props: { mode: "pixel", pixelSize: Math.max(1, Math.round(num(e.granularity, 30) / 5)) } });
        break;
      case "LensDistortionEffect": {
        const d = e.distortion as { x: number } | undefined;
        parts.push({ type: "LensDistortion", props: { k1: d?.x ?? 0 } });
        break;
      }
      default:
        warnOnce(`<EffectComposer> effect ${e.name}`, "has no raea counterpart: left out (write it as a raea <FramePass> outside <R3F>)");
    }
  }
  if (Object.keys(grade).length) parts.push({ type: "Grade", props: grade });
  return { parts, tone };
}
