// Drag and drop on 2D parts (docs/sdk.feature S2c): the square is pressed and dragged onto the circle. The square turns
// yellow while it is dragged (onDragStart to onDragEnd); the circle turns orange when the drag enters it (onDragEnter)
// and green when the square is dropped on it (onDrop). drag2d/check.sh renders input.json's run and checks the colours.
import { useState } from "react";
import { AbsoluteFill, Composition, registerRoot } from "raea";
import { Shape2D } from "raea/sprite";

function Shot() {
  const [dragging, setDragging] = useState(false);
  const [circle, setCircle] = useState<"idle" | "entered" | "dropped">("idle");
  return (
    <AbsoluteFill style={{ background: "#e8eef6" }}>
      <Shape2D name="square" shape="rect" size={[120, 120]} x={295} y={270} color={dragging ? "#ffd21f" : "#3a6cff"} onDragStart={() => setDragging(true)} onDragEnd={() => setDragging(false)} />
      <Shape2D name="circle" shape="circle" size={[140, 140]} x={690} y={270} color={circle === "dropped" ? "#22b573" : circle === "entered" ? "#ff8a1f" : "#9a9aa8"} onDragEnter={(e) => e.dragged === "square" && setCircle("entered")} onDrop={(e) => e.dragged === "square" && setCircle("dropped")} />
    </AbsoluteFill>
  );
}

registerRoot(() => <Composition id="drag2d" component={Shot} width={960} height={540} fps={30} durationInFrames={30} />);
