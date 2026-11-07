// docs/control.feature FC0a: Bevy's EnvironmentMapLight with its own prefiltered maps (3d/pbr, 3d/animated_material),
// its rotation, and Bevy's Skybox (3d/rotate_environment_map).
import { describe, expect, test } from "bun:test";
import { EnvironmentMapLight, Skybox } from "raea/light";
import { renderFrame } from "../src/test";

const html = (el: React.ReactElement) => renderFrame(el, { frame: 0, width: 320, height: 240, fps: 30, durationInFrames: 30 });

describe("EnvironmentMapLight: prefiltered cubemaps on the camera", () => {
  test("its maps and intensity reach the camera's Environment row, intensity 1 by Bevy's default", () => {
    const out = html(<EnvironmentMapLight diffuse_map="maps/d.ktx2" specular_map="maps/s.ktx2" intensity={900} />);
    expect(out).toContain('data-3d="Environment"');
    for (const s of ["maps/d.ktx2", "maps/s.ktx2", "900"]) expect(out).toContain(s);
    expect(html(<EnvironmentMapLight diffuse_map="d.ktx2" specular_map="s.ktx2" />)).toContain("&quot;intensity&quot;:1");
  });
  test("a map that isn't a .ktx2, or a negative intensity, is refused", () => {
    expect(() => html(<EnvironmentMapLight diffuse_map="sky.hdr" specular_map="s.ktx2" />)).toThrow(/diffuse_map is a prefiltered/);
    expect(() => html(<EnvironmentMapLight diffuse_map="d.ktx2" specular_map="s.png" />)).toThrow(/specular_map is a prefiltered/);
    expect(() => html(<EnvironmentMapLight diffuse_map="d.ktx2" specular_map="s.ktx2" intensity={-1} />)).toThrow(/intensity/);
  });
});

const Y = (a: number): [number, number, number, number] => [0, Math.sin(a / 2), 0, Math.cos(a / 2)];

describe("rotation and Skybox: Bevy's Quat on both, the sky's cubemap and brightness", () => {
  test("EnvironmentMapLight's rotation reaches its row as glam's [x, y, z, w]; none sends none", () => {
    const q = Y(0.4);
    expect(html(<EnvironmentMapLight diffuse_map="d.ktx2" specular_map="s.ktx2" rotation={q} />)).toContain(`&quot;rotation&quot;:[0,${q[1]},0,${q[3]}]`);
    expect(html(<EnvironmentMapLight diffuse_map="d.ktx2" specular_map="s.ktx2" />)).not.toContain("rotation");
  });
  test("a Skybox row carries its image, brightness (Bevy's default 0) and rotation", () => {
    const out = html(<Skybox image="maps/s.ktx2" brightness={5000} rotation={Y(1)} />);
    expect(out).toContain('data-3d="Skybox"');
    for (const s of ["maps/s.ktx2", "&quot;brightness&quot;:5000", `${Y(1)[1]}`]) expect(out).toContain(s);
    expect(html(<Skybox image="s.ktx2" />)).toContain("&quot;brightness&quot;:0");
  });
  test("a sky that isn't a .ktx2, a negative brightness or a rotation that isn't a unit Quat is refused", () => {
    expect(() => html(<Skybox image="sky.hdr" />)).toThrow(/Skybox: image/);
    expect(() => html(<Skybox image="s.ktx2" brightness={-1} />)).toThrow(/brightness/);
    expect(() => html(<Skybox image="s.ktx2" rotation={[0, 2, 0, 1]} />)).toThrow(/unit Quat/);
    expect(() => html(<EnvironmentMapLight diffuse_map="d.ktx2" specular_map="s.ktx2" rotation={[0, 0, 0] as never} />)).toThrow(/unit Quat/);
  });
});
