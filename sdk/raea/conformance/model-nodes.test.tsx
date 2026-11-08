// <Model nodes> (docs/control.feature FC0a: gltf/gltf_skinned_mesh): glTF nodes' local transforms by name, carried on
// the model's row as given, and refused when a part isn't its numbers.
import { expect, test } from "bun:test";
import { Model } from "../src/3d";
import { renderFrame } from "../src/test";

const html = (el: React.ReactElement) => renderFrame(el, { frame: 0, width: 320, height: 240, fps: 30, durationInFrames: 30 });

test("nodes reach the model's row as given", () => {
  const out = html(<Model src="skin.gltf" nodes={{ GltfNode2: { rotation: [0, 0, 0.5, 0.866] }, hip: { translation: [0, 1, 0], scale: [2, 2, 2] } }} />);
  expect(out).toContain("GltfNode2");
  expect(out).toContain("0.866");
  expect(out).toContain("hip");
});

test("refuses an unknown part and a part of the wrong length", () => {
  expect(() => html(<Model src="skin.gltf" nodes={{ a: { spin: [0, 0, 0] } as never }} />)).toThrow(/translation, rotation and scale/);
  expect(() => html(<Model src="skin.gltf" nodes={{ a: { rotation: [0, 0, 1] as never } }} />)).toThrow(/4 numbers/);
  expect(() => html(<Model src="skin.gltf" nodes={{ a: { translation: [0, NaN, 0] } }} />)).toThrow(/3 numbers/);
});
