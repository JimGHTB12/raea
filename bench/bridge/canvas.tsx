// B3 H7 from TSX (bench/bridge/examples/canvas_box): a layered <Canvas3D> inside a positioned 200 x 150 box on a red
// page draws its 3D at that box, at the box's size and aspect: the box's green background where the 3D draws nothing,
// a blue unit cube in the middle (square on screen: a frame-sized picture squeezed into the box would draw it 0.75 as
// wide as tall), and the page red outside it.
import { AbsoluteFill, Composition, registerRoot } from "raea";

import { Transform } from "raea/transform";
import { Camera3d, Canvas3D } from "raea/camera";
import { Mesh3d, Cuboid } from "raea/mesh";
import { StandardMaterial } from "raea/pbr";

function Shot() {
  return (
    <AbsoluteFill style={{ background: "#ff0000" }}>
      <div style={{ position: "absolute", left: 100, top: 50, width: 200, height: 150, background: "#00ff00" }}>
        <Canvas3D layered>
          <Camera3d transform={Transform.fromXyz(0, 0, 5).lookingAt([0, 0, 0])} projection={{ Perspective: { fov: 0.698131700797732 } }} />
          <Mesh3d>
            <Cuboid half_size={[0.5, 0.5, 0.5]} />
            <StandardMaterial base_color="#0000ff" unlit />
          </Mesh3d>
        </Canvas3D>
      </div>
    </AbsoluteFill>
  );
}

registerRoot(() => <Composition id="bridgecanvas" component={Shot} width={640} height={360} fps={30} durationInFrames={60} />);
