// Lens fixture: a white wall ruled with black lines (unlit, so pixels are exact) seen square-on, and a fisheye that
// holds at strength 0 to frame 9, ramps to 1 by frame 40 and holds. A 2D label on the page sits over the 3D: the lens must leave it alone. lens/check.py measures straight
// centre lines, monotonic curves away from the centre, strength 0 = no lens, the page untouched and determinism;
// render equals its baked play (check.sh).
import { AbsoluteFill, Composition, registerRoot, track } from "raea";
import { Fisheye } from "raea/3d";
import { StandardMaterial } from "raea/pbr";
import { Cuboid, Mesh3d } from "raea/mesh";
import { Transform } from "raea/transform";
import { Camera3d } from "raea/camera";

const FPS = 30;
const FRAMES = 60;
const LENS = { strength: track({ 0: 0, 9: 0, 40: 1 }, "in-out-sine") };
const XS = [-4, -3, -2, -1, 0, 1, 2, 3, 4];
const YS = [-2, -1, 0, 1, 2];

function Shot() {
  return (
    <AbsoluteFill style={{ background: "#202020" }}>
      <>
        <Camera3d transform={Transform.fromXyz(0, 0, 6).lookingAt([0, 0, 0])} projection={{ Perspective: { fov: 0.872664625997165 } }} />
        {/* lens */}<Fisheye strength={0} animate={LENS} />
        <Mesh3d transform={Transform.fromXyz(0, 0, -0.1)} name="wall">
          <Cuboid half_size={[12, 7, 0.025]} />
          <StandardMaterial base_color="#ffffff" unlit />
        </Mesh3d>
        {XS.map((x) => (
          <Mesh3d key={`v${x}`} transform={Transform.fromXyz(x, 0, 0)} name={`v${x}`}>
            <Cuboid half_size={[0.02, 7, 0.01]} />
            <StandardMaterial base_color="#000000" unlit />
          </Mesh3d>
        ))}
        {YS.map((y) => (
          <Mesh3d key={`h${y}`} transform={Transform.fromXyz(0, y, 0)} name={`h${y}`}>
            <Cuboid half_size={[12, 0.02, 0.01]} />
            <StandardMaterial base_color="#000000" unlit />
          </Mesh3d>
        ))}
      </>
      <div style={{ position: "absolute", left: 24, top: 24, width: 180, height: 44, background: "#ff3366", color: "#ffffff", font: "700 26px Inter", display: "flex", alignItems: "center", justifyContent: "center" }}>STRAIGHT</div>
    </AbsoluteFill>
  );
}

registerRoot(() => <Composition id="lens" component={Shot} width={960} height={540} fps={FPS} durationInFrames={FRAMES} />);
