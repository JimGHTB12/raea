// docs/control.feature FC2a's first step: your own Material, its fragment shader a file compiled as written.
import { describe, expect, test } from "bun:test";
import { Cuboid } from "../src/3d";
import { ExtendedMaterial, Mesh3d, ShaderExtension, StandardMaterial } from "@raea/render/bevy3d";
import { Material } from "@raea/render/bevy3d"; // raea/pbr once scene-loop lands (its pbr.ts commit)
import { renderFrame } from "../src/test";

const html = (el: React.ReactElement) => renderFrame(el, { frame: 0, width: 320, height: 240, fps: 30, durationInFrames: 30 });

describe("FC2a: a Material of your own", () => {
  test("a Material is its fragment file over the standard base, uniforms and all, as the shader row reads it", () => {
    const own = html(<Mesh3d><Cuboid /><Material fragment="shaders/a.wesl" uniforms={{ steps: 3 }} /></Mesh3d>);
    expect(own).toBe(html(<Mesh3d><Cuboid /><ExtendedMaterial base={<StandardMaterial />} extension={<ShaderExtension fragment="shaders/a.wesl" uniforms={{ steps: 3 }} />} /></Mesh3d>));
    expect(own).toContain("shaders/a.wesl");
    expect(own).not.toBe(html(<Mesh3d><Cuboid /><Material fragment="shaders/b.wesl" uniforms={{ steps: 3 }} /></Mesh3d>));
  });
  test("a Material without its file, or a shader with two sources, is refused", () => {
    expect(() => html(<Mesh3d><Cuboid /><Material {...({} as { fragment: string })} /></Mesh3d>)).toThrow(/fragment is the shader file/);
    expect(() => html(<Mesh3d><Cuboid /><ExtendedMaterial base={<StandardMaterial />} extension={<ShaderExtension fragment="a.wesl" wesl="@fragment fn f() {}" />} /></Mesh3d>)).toThrow(/one of them/);
  });
});
