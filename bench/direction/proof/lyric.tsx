// DR4's proof, the lyric as asked: footage, the sung line over it as flat 2D type inside the frame, and the subject
// (a cutout standing in for a person) in front of part of it. checks/lyric.ts holds it.
import { AbsoluteFill, Composition, Video, registerRoot } from "raea";

function Lyric() {
  return (
    <AbsoluteFill style={{ background: "#000" }}>
      <Video src="media/take.mp4" muted style={{ position: "absolute", left: 0, top: 0, width: 540, height: 960 }} />
      <div style={{ position: "absolute", left: 60, top: 300, width: 420, fontSize: 64, fontWeight: 800, color: "#ffffff", lineHeight: 1.1 }}>OUT OF THE GREY</div>
      <div style={{ position: "absolute", left: 220, top: 250, width: 180, height: 710, borderRadius: "90px 90px 0 0", background: "#2b2b2b" }} />
    </AbsoluteFill>
  );
}

registerRoot(() => <Composition id="lyric" component={Lyric} width={540} height={960} fps={30} durationInFrames={60} />);
