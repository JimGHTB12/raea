// raea → R3F source (docs/r3f.feature R3F9): what the source export writes. tools/parity.sh draws what it writes in
// Chrome against raea's render (src-* cases); this checks what's in the file.
import { describe, expect, test } from "bun:test";
import { toR3FSource } from "../src/export";
import { OUTPUTS, SIZE } from "./parity/fixtures";

describe("the source export", () => {
  test("a scene's parts are JSX, what moves a frame table, settings on the canvas", async () => {
    const { source: src, effects } = await toR3FSource(OUTPUTS.basic.component as never, { ...SIZE, from: 0, frames: 3, name: "Basic" });
    expect(effects).toBeUndefined();
    expect(src).toContain('import { Canvas, useFrame, useLoader, useThree } from "@react-three/fiber";');
    expect(src).not.toMatch(/from "(raea|@raea)/);
    // four meshes (ground, box, ball, ring), three lights, raea's background
    expect(src.match(/<mesh /g)?.length).toBe(4);
    for (const l of ["<ambientLight", "<directionalLight", "<pointLight"]) expect(src).toContain(l);
    expect(src).toContain('<color attach="background" args={["#1d2330"]} />');
    expect(src).toContain('<meshPhysicalMaterial color="#3fb950" roughness={0.2} clearcoat={1} />');
    expect(src).toContain("<sphereGeometry args=");
    // the box turns: its quaternion is a track of three frames; nothing else moves
    const tracks = JSON.parse(/^const TRACKS[^=]*= (.*);$/m.exec(src)![1]) as Record<string, Record<string, unknown[]>>;
    expect(Object.values(tracks).map((t) => Object.keys(t))).toEqual([["q"]]);
    expect(Object.values(tracks)[0].q.length).toBe(3);
    expect(src).toContain("gl.toneMapping = THREE.ACESFilmicToneMapping;");
    expect(src).toContain("export default function Basic()");
  });

  test("a solid (Text3D) is its triangles", async () => {
    const { source: src } = await toR3FSource(OUTPUTS.lights.component as never, { ...SIZE });
    expect(src).toMatch(/<primitive object=\{G\[\d+\]\} attach="geometry" \/>/);
    expect(src).toContain("RectAreaLightUniformsLib.init();");
    expect(src).toContain('<fog attach="fog" args={["#0b0e14", 6, 14]} />');
  });

  test("effects and Tony are drawn by raea-effects.ts, the composer <Raea> uses, written beside the module", async () => {
    const bloom = await toR3FSource(OUTPUTS.bloom.component as never, { ...SIZE });
    expect(bloom.source).toContain('import { Effects, type Effect } from "./raea-effects";');
    expect(bloom.source).toMatch(/const EFFECTS: Effect\[\] \| Effect\[\]\[\] = \[\{"kind":"Bloom"/);
    expect(bloom.source).toContain("const TONY = false;");
    expect(bloom.effects).toContain("export class Effects");
    expect(bloom.effects).not.toMatch(/from "\.\//);
    const tony = await toR3FSource(OUTPUTS.tony.component as never, { ...SIZE });
    expect(tony.source).toContain("const TONY = true;");
    expect(tony.source).toContain("const EFFECTS: Effect[] | Effect[][] = [];");
  });
});
