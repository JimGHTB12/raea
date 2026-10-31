// Drag and drop on 3D parts (docs/sdk.feature S2c): the cube is pressed and dragged onto the ball. The cube turns
// yellow while it is dragged (onDragStart to onDragEnd); the ball turns orange when the drag enters it (onDragEnter)
// and green when the cube is dropped on it (onDrop). drag3d/check.sh renders input.json's run and checks the colours.
import { useState } from "react";
import { AbsoluteFill, Composition, registerRoot } from "raea";
import { Sphere } from "raea/3d";
import { StandardMaterial } from "raea/pbr";
import { Cuboid, Mesh3d } from "raea/mesh";
import { Transform } from "raea/transform";
import { Camera3d } from "raea/camera";

function Shot() {
  const [dragging, setDragging] = useState(false);
  const [ball, setBall] = useState<"idle" | "entered" | "dropped">("idle");
  return (
    <AbsoluteFill style={{ background: "#e8eef6" }}>
      <Camera3d transform={Transform.fromXyz(0, 0, 6).lookingAt([0, 0, 0])} projection={{ Perspective: { fov: 0.698131700797732 } }} />
      <Mesh3d transform={Transform.fromXyz(-1.5, 0, 0)} name="cube" onDragStart={() => setDragging(true)} onDragEnd={() => setDragging(false)}>
        <Cuboid half_size={[0.6, 0.6, 0.6]} />
        <StandardMaterial base_color={dragging ? "#ffd21f" : "#3a6cff"} unlit />
      </Mesh3d>
      <Mesh3d transform={Transform.fromXyz(1.5, 0, 0)} name="ball" onDragEnter={(e) => e.dragged === "cube" && setBall("entered")} onDrop={(e) => e.dragged === "cube" && setBall("dropped")}>
        <Sphere radius={0.7} />
        <StandardMaterial base_color={ball === "dropped" ? "#22b573" : ball === "entered" ? "#ff8a1f" : "#9a9aa8"} unlit />
      </Mesh3d>
    </AbsoluteFill>
  );
}

registerRoot(() => <Composition id="drag3d" component={Shot} width={960} height={540} fps={30} durationInFrames={30} />);
