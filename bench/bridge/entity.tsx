// B3 H1's React side (docs/blitz-bevy.feature): useEntity(ref) gives an element's twin, the Bevy entity the page keeps
// for it (bench/bridge/examples/entity_ref.rs checks that the entity shown is the twin of #card).
import { AbsoluteFill, Composition, registerRoot } from "raea";
import { useEntity } from "raea/ecs";
import { useRef } from "react";

function Shot() {
  const card = useRef(null);
  const twin = useEntity(card);
  return (
    <AbsoluteFill style={{ background: "#101820" }}>
      <div id="card" ref={card} style={{ position: "absolute", left: 40, top: 40, width: 200, height: 100, background: "#2040ff" }} />
      <div id="twin" style={{ position: "absolute", left: 40, top: 160, color: "white", font: "24px Inter" }}>{twin === undefined ? "twin none" : `twin ${twin}`}</div>
    </AbsoluteFill>
  );
}

registerRoot(() => <Composition id="entity" component={Shot} width={320} height={240} fps={30} durationInFrames={600} />);
