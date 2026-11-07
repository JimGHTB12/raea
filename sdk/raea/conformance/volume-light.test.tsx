// Simulated volumes in the scene's light (spec/motion.feature, "3D: simulated volumes"): a <Volume> casts a shadow by
// default (engine/render/src/engine/volume.rs draws it into the lights' shadow maps through volume_shadow.wesl, so it
// shadows meshes and stops the sun's shafts in the volumetric fog; the examples built on it, examples/volumes, set no
// shadow of their own); `shadow` scales or turns that off and goes to the row as a number; a Mesh's `shadow="only"`
// casts without being drawn.
import { describe, expect, test } from "bun:test";
import { Canvas3D, Volume } from "../src/3d";
import { Mesh, MeshStandardMaterial } from "@raea/render/three";
// the retired three.js shapes, compared against (M5: no longer exported from raea)
import { Box } from "@raea/render/3d/shapes";
import { renderFrame } from "../src/test";
import { useCurrentFrame } from "@raea/app/clock";

const base = { fps: 30, width: 1920, height: 1080, durationInFrames: 60 };
const props = (html: string, kind: string) => {
  const m = html.match(new RegExp(`data-3d="${kind}" data-props="([^"]*)"`));
  return m ? JSON.parse(m[1].replace(/&quot;/g, '"').replace(/&amp;/g, "&")) : undefined;
};
const at = (el: any, kind = "Fire", frame = 0) => props(renderFrame(<Canvas3D>{el}</Canvas3D>, { ...base, frame }), kind);

describe("volume shadows", () => {
  test("a volume casts a shadow as dense as it's drawn by default", () => {
    expect(at(<Volume size={[8, 4, 8]} />).shadow).toBe(1);
  });

  test("shadow scales it, and false or 0 casts none", () => {
    expect(at(<Volume size={[8, 4, 8]} shadow={0.5} />).shadow).toBe(0.5);
    expect(at(<Volume size={[8, 4, 8]} shadow={3} />).shadow).toBe(3);
    expect(at(<Volume size={[8, 4, 8]} shadow />).shadow).toBe(1);
    expect(at(<Volume size={[8, 4, 8]} shadow={false} />).shadow).toBe(0);
    expect(at(<Volume size={[8, 4, 8]} shadow={0} />).shadow).toBe(0);
  });

  test("the sim spec stays beside it", () => {
    const p = at(<Volume size={[8, 4, 8]} name="c" shadow={0.8} seed={3} puffs={[[0, 2, 0, 1.5]]} />);
    expect(p).toMatchObject({ name: "c", shadow: 0.8 });
    expect(p.sim.puffs.length).toBeGreaterThan(0);
    expect(p.sim.shadow).toBeUndefined();
  });

  test("the same frame gives the same row", () => {
    function C() {
      const f = useCurrentFrame();
      return <Volume size={[8, 4, 8]} name="drift" position={[f * 0.2, 30, 0]} shadow={0.7} />;
    }
    expect(at(<C />, "Fire", 12)).toEqual(at(<C />, "Fire", 12));
  });

  test("bad shadows throw, naming the prop", () => {
    const r = (el: any) => () => at(el);
    expect(r(<Volume size={[8, 4, 8]} shadow={-1} />)).toThrow(/shadow/);
    expect(r(<Volume size={[8, 4, 8]} shadow={5} />)).toThrow(/shadow/);
    expect(r(<Volume size={[8, 4, 8]} shadow={"yes" as any} />)).toThrow(/shadow/);
    expect(r(<Volume size={[8, 4, 8]} shadow={NaN} />)).toThrow(/shadow/);
  });
});

describe("a mesh's shadow", () => {
  const mesh = (shadow: any) => (
    <Mesh name="roof" position={[0, 6, 0]} shadow={shadow}>
      <Box args={[8, 0.2, 8]} />
      <MeshStandardMaterial color="#888888" />
    </Mesh>
  );

  test('"only" goes through as it is; true and false as before', () => {
    expect(at(mesh("only"), "Mesh").shadow).toBe("only");
    expect(at(mesh(false), "Mesh").shadow).toBe(false);
    expect(at(mesh(undefined), "Mesh").shadow).toBeUndefined();
  });

  test("anything else throws", () => {
    expect(() => at(mesh("never"), "Mesh")).toThrow(/shadow/);
    expect(() => at(mesh(1), "Mesh")).toThrow(/shadow/);
  });
});
