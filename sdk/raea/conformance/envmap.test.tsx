// docs/control.feature FC0a: Bevy's EnvironmentMapLight with its own prefiltered maps (3d/pbr, 3d/animated_material).
import { describe, expect, test } from "bun:test";
import { EnvironmentMapLight } from "raea/light";
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
