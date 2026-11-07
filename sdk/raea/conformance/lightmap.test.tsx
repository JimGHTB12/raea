// docs/control.feature FC0a: Bevy's Lightmap on a Model's glTF meshes by name, and StandardMaterial's
// lightmap_exposure on the Model's materials (3d/lightmaps).
import { describe, expect, test } from "bun:test";
import { Model } from "raea/3d";
import { Lightmap, StandardMaterial } from "raea/pbr";
import { renderFrame } from "../src/test";

const html = (el: React.ReactElement) => renderFrame(el, { frame: 0, width: 320, height: 240, fps: 30, durationInFrames: 30 });
const q = (s: string) => s.replaceAll('"', "&quot;");

describe("Lightmap: a bake on the glTF meshes it names", () => {
  test("inside a Model it carries its file, mesh, sampling and rect (the whole image by default)", () => {
    const out = html(<Model src="box.glb"><Lightmap mesh="large_box" image="maps/large.ktx2" /></Model>);
    expect(out).toContain('data-3d="Lightmap"');
    expect(out).toContain(q('{"lightmap":"maps/large.ktx2","mesh":"large_box","bicubic":0,"uv_x0":0,"uv_y0":0,"uv_x1":1,"uv_y1":1}'));
    const atlas = html(<Model src="box.glb"><Lightmap mesh="b" image="m.ktx2" bicubic_sampling uv_rect={{ min: [0.5, 0], max: [1, 0.25] }} /></Model>);
    expect(atlas).toContain(q('"bicubic":1,"uv_x0":0.5,"uv_y0":0,"uv_x1":1,"uv_y1":0.25'));
  });
  test("outside a Model, without a mesh or image, or with a rect off the image, it is refused", () => {
    expect(() => html(<Lightmap mesh="b" image="m.ktx2" />)).toThrow(/goes in a <Model>/);
    expect(() => html(<Model src="b.glb"><Lightmap mesh="" image="m.ktx2" /></Model>)).toThrow(/mesh is the glTF mesh/);
    expect(() => html(<Model src="b.glb"><Lightmap mesh="b" image="" /></Model>)).toThrow(/image is a picture/);
    expect(() => html(<Model src="b.glb"><Lightmap mesh="b" image="m.ktx2" uv_rect={{ min: [0, 0], max: [1.5, 1] }} /></Model>)).toThrow(/uv_rect/);
  });
  test("lightmap_exposure reaches a Model's material as its field, and only when given", () => {
    expect(html(<Model src="b.glb"><StandardMaterial material="box" lightmap_exposure={250} /></Model>)).toContain(q('"lightmapExposure":250'));
    expect(html(<Model src="b.glb"><StandardMaterial material="box" base_color="#ff0000" /></Model>)).not.toContain("lightmapExposure");
  });
});
