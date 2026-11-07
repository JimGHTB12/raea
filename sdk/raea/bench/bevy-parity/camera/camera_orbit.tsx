// Bevy v0.20.0 examples/camera/camera_orbit.rs in raea's TSX (docs/control.feature FC0a): a 5 m green plane, a 1 m
// cube at (1.5, 0.51, 1.5) and a default PointLight at (3, 8, 5); the camera starts at (5, 5, 5) looking at the origin
// and its orbit system (no mouse) puts it 20 m back along its view, and three lines of instructions in Bevy's default
// font at top 12, left 12, in Bevy's default 1280 x 720 window.
import { AbsoluteFill, Composition, registerRoot, staticFile } from "raea";
import { Canvas3D } from "raea/3d";
import { Transform } from "raea/transform";
import { Camera3d } from "raea/camera";
import { Cuboid, Mesh3d, Plane3d } from "raea/mesh";
import { StandardMaterial } from "raea/pbr";
import { PointLight } from "raea/light";

const FONT = `@font-face { font-family: "Fira Mono"; src: url("${staticFile("fonts/FiraMono-Medium.ttf")}"); }`;
// orbit: translation = target - forward x orbit_distance (20), the view along (-1, -1, -1)
const D = 20 / Math.sqrt(3);

function Shot() {
  return (
    <AbsoluteFill>
      <style>{FONT}</style>
      <Canvas3D layered clearColor="rgb(43, 44, 47)">
        <Camera3d transform={Transform.fromXyz(D, D, D).lookingAt([0, 0, 0])} />
        <Mesh3d transform={Transform.IDENTITY}>
          <Plane3d half_size={[2.5, 2.5]} />
          <StandardMaterial base_color="rgb(76.5, 127.5, 76.5)" cull_mode={null} />
        </Mesh3d>
        <Mesh3d transform={Transform.fromXyz(1.5, 0.51, 1.5)}>
          <Cuboid half_size={[0.5, 0.5, 0.5]} />
          <StandardMaterial base_color="rgb(204, 178.5, 153)" />
        </Mesh3d>
        <PointLight transform={Transform.fromXyz(3, 8, 5)} />
      </Canvas3D>
      <div style={{ position: "absolute", top: 12, left: 12, fontFamily: "Fira Mono", fontSize: 20, lineHeight: 1.2, color: "#fff", whiteSpace: "pre" }}>{"Mouse up or down: pitch\nMouse left or right: yaw\nMouse buttons: roll"}</div>
    </AbsoluteFill>
  );
}

registerRoot(() => <Composition id="main" component={Shot} durationInFrames={11} fps={60} width={1280} height={720} />);
