// Bevy v0.20.0 examples/camera/projection_zoom.rs in raea's TSX (docs/control.feature FC0a): an orthographic camera
// 5 m tall at (5, 5, 5) looking at the origin (no scroll, so its scale stays 1), a 5 m green plane drawn both sides,
// the fox (models/animated/Fox.glb's scene, unanimated) at 0.025, a default PointLight at (3, 8, 5), and two lines of
// instructions in Bevy's default font at top 12, left 12, in Bevy's default 1280 x 720 window.
import { AbsoluteFill, Composition, registerRoot, staticFile } from "raea";
import { Canvas3D, Entity, Model } from "raea/3d";
import { Transform } from "raea/transform";
import { Camera3d } from "raea/camera";
import { Mesh3d, Plane3d } from "raea/mesh";
import { StandardMaterial } from "raea/pbr";
import { PointLight } from "raea/light";

const FONT = `@font-face { font-family: "Fira Mono"; src: url("${staticFile("fonts/FiraMono-Medium.ttf")}"); }`;

function Shot() {
  return (
    <AbsoluteFill>
      <style>{FONT}</style>
      <Canvas3D layered clearColor="rgb(43, 44, 47)">
        <Camera3d projection={{ Orthographic: { scaling_mode: { FixedVertical: { viewport_height: 5 } } } }} transform={Transform.fromXyz(5, 5, 5).lookingAt([0, 0, 0])} />
        <Mesh3d transform={Transform.IDENTITY}>
          <Plane3d half_size={[2.5, 2.5]} />
          <StandardMaterial base_color="rgb(76.5, 127.5, 76.5)" cull_mode={null} />
        </Mesh3d>
        <Entity transform={{ translation: [0, 0, 0], scale: [0.025, 0.025, 0.025] }}>
          <Model src={staticFile("models/animated/Fox.glb")} />
        </Entity>
        <PointLight transform={Transform.fromXyz(3, 8, 5)} />
      </Canvas3D>
      <div style={{ position: "absolute", top: 12, left: 12, fontFamily: "Fira Mono", fontSize: 20, lineHeight: 1.2, color: "#fff", whiteSpace: "pre" }}>{"Scroll mouse wheel to zoom in/out\nSpace: switch between orthographic and perspective projections"}</div>
    </AbsoluteFill>
  );
}

registerRoot(() => <Composition id="main" component={Shot} durationInFrames={31} fps={60} width={1280} height={720} />);
