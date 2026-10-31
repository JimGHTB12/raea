// raea/views: the type-free pictures of an edit happening (docs/views.md).
import { describe, expect, test } from "bun:test";
import { CutStrip, EmbeddingGraph, FaceBox, HandSkeleton, HAND_BONES, Timeline, Waveform, WordTrack, cameraEye, projector, snapTo } from "@raea/dev_tools/views";
import { renderFrame } from "../src/test";

const at = (node: any) => renderFrame(node, { frame: 0, fps: 30, width: 1920, height: 1080, durationInFrames: 30 });
const lefts = (html: string) => [...html.matchAll(/left:\s*(-?[\d.]+)px/g)].map((m) => Number(m[1]));
const cam = { target: [0, 0, 0] as [number, number, number], yaw: 0, pitch: 0, distance: 5, fov: 60 };

describe("projector", () => {
  test("the target lands in the middle of the frame", () => {
    const p = projector(cam, 1920, 1080)([0, 0, 0])!;
    expect(p.x).toBeCloseTo(960);
    expect(p.y).toBeCloseTo(540);
    expect(p.depth).toBeCloseTo(5);
  });
  test("up is up, right is right, and nearer is bigger", () => {
    const pr = projector(cam, 1920, 1080);
    expect(pr([0, 1, 0])!.y).toBeLessThan(540);
    expect(pr([1, 0, 0])!.x).toBeGreaterThan(960);
    expect(pr([0, 0, 2])!.scale).toBeGreaterThan(pr([0, 0, -2])!.scale);
  });
  test("a point behind the camera is not drawn", () => {
    expect(projector(cam, 1920, 1080)([0, 0, 6])).toBeNull();
  });
  test("the eye orbits with yaw", () => {
    const e = cameraEye({ ...cam, yaw: Math.PI / 2 });
    expect(e[0]).toBeCloseTo(5);
    expect(e[2]).toBeCloseTo(0);
  });
});

describe("EmbeddingGraph", () => {
  const nodes = [
    { pos: [0, 0, 0] as [number, number, number], color: "#f00" },
    { pos: [1, 0, -1] as [number, number, number], color: "#0f0", src: "a.jpg" },
  ];
  test("a node per point and a line per edge", () => {
    const html = at(<EmbeddingGraph nodes={nodes} edges={[{ a: 0, b: 1 }]} camera={cam} width={1920} height={1080} />);
    expect(html.match(/<line/g)?.length).toBe(1);
    expect(html).toContain("a.jpg");
  });
  test("an edge not drawn in yet is left out, and a hidden node with it", () => {
    const html = at(<EmbeddingGraph nodes={[nodes[0], { ...nodes[1], opacity: 0 }]} edges={[{ a: 0, b: 1 }]} camera={cam} width={1920} height={1080} />);
    expect(html).not.toContain("<line");
    expect(html).not.toContain("a.jpg");
    const none = at(<EmbeddingGraph nodes={nodes} edges={[{ a: 0, b: 1, drawn: 0 }]} camera={cam} width={1920} height={1080} />);
    expect(none).not.toContain("<line");
  });
  test("a lit node takes the lit colour and glows", () => {
    const html = at(<EmbeddingGraph nodes={[{ ...nodes[0], lit: 1 }]} camera={cam} width={1920} height={1080} litColor="#ffd400" />);
    expect(html).toContain("#ffd400");
    expect(html).toContain("box-shadow");
  });
});

describe("Timeline", () => {
  test("clips sit where their times fall in the view", () => {
    const html = at(<Timeline width={1000} height={300} view={[0, 10]} tracks={[{ height: 80, clips: [{ from: 2, to: 4, color: "#123456" }] }]} />);
    expect(lefts(html)).toContain(201); // 2 s of 10 across 1000 px, plus the 1 px gutter
  });
  test("the playhead and blade are drawn only when in view", () => {
    const tl = (playhead: number) => at(<Timeline width={1000} height={300} view={[0, 10]} tracks={[]} playhead={playhead} blade={5} theme={{ playhead: "#abcdef", blade: "#fedcba" }} />);
    expect(tl(5)).toContain("#abcdef");
    expect(tl(5)).toContain("#fedcba");
    expect(tl(30)).not.toContain("#abcdef");
  });
  test("downbeats are taller than beats", () => {
    const html = at(<Timeline width={1000} height={300} view={[0, 4]} tracks={[]} beats={[1, 2]} downbeats={[1]} ruler={40} />);
    const hs = [...html.matchAll(/height:\s*([\d.]+)px/g)].map((m) => Number(m[1]));
    expect(hs).toContain(32);
    expect(hs).toContain(16);
  });
  test("a flipped clip is turned upside down, and a waveform clip draws its bars", () => {
    const html = at(<Timeline width={1000} height={300} view={[0, 10]} tracks={[{ height: 80, clips: [{ from: 0, to: 5, color: "#111", flip: true, wave: [0.5, 1] }] }]} />);
    expect(html).toContain("scaleY(-1)");
    expect(html).toContain("rgba(11,11,13,0.75)");
  });
});

describe("overlays", () => {
  test("FaceBox draws eight bracket arms once locking, nothing before", () => {
    expect(at(<FaceBox box={[100, 100, 200, 200]} lock={0} />)).toBe("");
    const html = at(<FaceBox box={[100, 100, 200, 200]} lock={1} color="#ffd400" />);
    expect(html.match(/#ffd400/g)?.length).toBe(8);
  });
  test("HandSkeleton draws MediaPipe's bones", () => {
    const pts = Array.from({ length: 21 }, (_, i) => [i * 10, i * 5] as [number, number]);
    const html = at(<HandSkeleton points={pts} width={500} height={500} />);
    expect(html.match(/<line/g)?.length).toBe(HAND_BONES.length);
    expect(html.match(/<circle/g)?.length).toBe(21);
  });
  test("CutStrip snaps a cut line in only once its snap begins", () => {
    const frames = Array.from({ length: 20 }, (_, i) => `f${i}.jpg`);
    const off = at(<CutStrip frames={frames} scroll={0} frameWidth={100} height={100} width={1000} cuts={[{ at: 3, snap: 0 }]} color="#ff3b1f" />);
    const on = at(<CutStrip frames={frames} scroll={0} frameWidth={100} height={100} width={1000} cuts={[{ at: 3, snap: 1 }]} color="#ff3b1f" />);
    expect(off).not.toContain("#ff3b1f");
    expect(on).toContain("#ff3b1f");
    expect(lefts(on)).toContain(297);
  });
  test("WordTrack lights the word being said", () => {
    const words = [{ start: 0, end: 0.5 }, { start: 0.6, end: 1 }];
    const html = at(<WordTrack words={words} time={0.7} span={2} width={1000} height={40} lit="#ffd400" />);
    expect(html.match(/#ffd400/g)?.length).toBe(2); // the lit word and the now line
  });
  test("Waveform colours what has played", () => {
    const html = at(<Waveform peaks={[0.2, 0.8]} width={100} height={40} bar={10} played={0.5} playedColor="#00ff00" color="#ff00ff" />);
    expect(html.match(/#00ff00/g)?.length).toBe(5);
    expect(html.match(/#ff00ff/g)?.length).toBe(5);
  });
});

describe("snapTo", () => {
  test("the nearest grid time and how far it moved", () => {
    expect(snapTo([1, 2, 3], 2.3)).toEqual({ at: 2, moved: -0.2999999999999998 });
  });
});
