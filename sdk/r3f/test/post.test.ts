// @react-three/postprocessing's effects as raea's (src/input/post.ts, docs/r3f.feature R3F12): pmndrs postprocessing's
// own classes, read the way they're found under an <EffectComposer>.
import { describe, expect, test } from "bun:test";
import { BloomEffect, ChromaticAberrationEffect, HueSaturationEffect, NoiseEffect, SMAAEffect, ToneMappingEffect, ToneMappingMode, VignetteEffect } from "postprocessing";
import * as THREE from "three";
import { postParts } from "../src/input/post";
import { EFFECTS } from "../src/input/kinds";

describe("<EffectComposer> → raea's frame effects", () => {
  test("bloom, colour, tone and anti-aliasing become raea's", () => {
    const { parts, tone } = postParts([
      new BloomEffect({ intensity: 1.5, luminanceThreshold: 0.6 }),
      new VignetteEffect({ offset: 0.3, darkness: 0.7 }),
      new HueSaturationEffect({ saturation: -0.3 }),
      new ChromaticAberrationEffect({ offset: new THREE.Vector2(0.003, 0.004), radialModulation: false, modulationOffset: 0 }),
      new ToneMappingEffect({ mode: ToneMappingMode.AGX }),
      new SMAAEffect(),
    ]);
    expect(parts.find((p) => p.type === "Bloom")!.props).toEqual({ intensity: 1.5, threshold: 0.6 });
    const grade = parts.find((p) => p.type === "Grade")!.props as Record<string, any>;
    expect(grade.vignette.intensity).toBeCloseTo(0.7);
    expect(grade.saturation).toBeCloseTo(0.7);
    expect(grade.aberration).toBeCloseTo(0.005);
    expect(parts.find((p) => p.type === "AntiAliasing")!.props).toEqual({ mode: "smaa" });
    expect(tone!.props).toEqual({ mode: "agx" });
  });

  test("ACES carries three's 1/0.6 as exposure", () => {
    const { tone } = postParts([new ToneMappingEffect({ mode: ToneMappingMode.ACES_FILMIC })]);
    expect(tone!.props.mode).toBe("aces");
    expect(tone!.props.exposure as number).toBeCloseTo(Math.log2(1 / 0.6));
  });

  test("one raea has nothing for is named and left out", () => {
    const warn = console.warn;
    const said: string[] = [];
    console.warn = (s: string) => said.push(s);
    try {
      expect(postParts([new NoiseEffect()]).parts).toEqual([]);
      expect(said.join("\n")).toContain("NoiseEffect");
    } finally {
      console.warn = warn;
    }
  });

  test("every effect postprocessing exports is decided in kinds.ts", async () => {
    const pp = (await import("postprocessing")) as Record<string, unknown>;
    const effects = Object.keys(pp).filter((k) => /Effect$/.test(k) && typeof pp[k] === "function" && k !== "Effect");
    expect(effects.filter((k) => !EFFECTS[k])).toEqual([]);
  });
});
