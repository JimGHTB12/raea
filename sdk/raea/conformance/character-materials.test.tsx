// The character's skin and hair materials in Bevy's names (docs/control.feature FC1e, sdk/character drawing.tsx): the
// ExtendedMaterial of a StandardMaterial and a ShaderExtension that re-covers a Model's material by name gives the
// engine the nodes the old MeshStandardMaterial and MeshPhysicalMaterial with `shader` did.
import { expect, test } from "bun:test";
import { renderFrame } from "../src/test";
import { MeshPhysicalMaterial, MeshStandardMaterial, Model } from "@raea/render/3d";
import { ExtendedMaterial, ShaderExtension, StandardMaterial } from "@raea/render/bevy3d";
import { HAIR_WESL } from "@raea/character/character/hair";
import { SKIN_WESL } from "@raea/character/character/skin";
const html = (el: any) => renderFrame(el, { frame: 0, width: 320, height: 240, fps: 30, durationInFrames: 30 });
const U = { a: 1, b: [1, 2] };
// the nodes as the engine reads them: kind (runtime/scene reads MeshStandardMaterial and MeshPhysicalMaterial alike) and props
const nodes = (h: string) => [...h.matchAll(/data-3d="([^"]*)" data-props="([^"]*)"/g)].map((m) => [m[1] === "MeshPhysicalMaterial" ? "MeshStandardMaterial" : m[1], JSON.parse(m[2].replace(/&quot;/g, '"'))]);
test("hair", () => {
  for (const extra of [{}, { color: "#aa7744" }, { alphaMode: "coverage" as const }]) {
    const old = <MeshStandardMaterial material="hair" roughness={1} reflectance={0.04} {...extra} shader={{ wesl: HAIR_WESL, uniforms: U }} />;
    const nb = <StandardMaterial material="hair" perceptual_roughness={1} reflectance={0.04} {...("color" in extra ? { base_color: extra.color } : {})} {...("alphaMode" in extra ? { alpha_mode: "AlphaToCoverage" as const } : {})} />;
    expect(nodes(html(<Model src="a.glb">{<ExtendedMaterial base={nb} extension={<ShaderExtension wesl={HAIR_WESL} uniforms={U} />} />}</Model>))).toEqual(nodes(html(<Model src="a.glb">{old}</Model>)));
  }
});
test("skin", () => {
  const tex = { textures: { mask: "m.png" }, defs: { SKIN_MASK: true } };
  const old = <MeshPhysicalMaterial material="skin" clearcoat={0.15} clearcoatRoughness={0.32} reflectance={0.4} diffuseTransmission={0} shader={{ wesl: SKIN_WESL, uniforms: U, ...tex }} />;
  const nb = <StandardMaterial material="skin" clearcoat={0.15} clearcoat_perceptual_roughness={0.32} reflectance={0.4} diffuse_transmission={0} />;
  expect(nodes(html(<Model src="a.glb">{<ExtendedMaterial base={nb} extension={<ShaderExtension wesl={SKIN_WESL} uniforms={U} {...tex} />} />}</Model>))).toEqual(nodes(html(<Model src="a.glb">{old}</Model>)));
});
