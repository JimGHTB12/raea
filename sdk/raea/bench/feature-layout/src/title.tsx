// A feature: its components together in one file. A feature with work in the App each frame (systems, resources)
// also exports its plugin, definePlugin({ name, build(app) }) from "raea/app"; this one draws only, so it has none.
import { interpolate, useCurrentFrame } from "raea";

export function Title({ text, from }: { text: string; from: number }) {
  const f = useCurrentFrame();
  const shown = interpolate(f, [from, from + 12], [0, 1], { extrapolateLeft: "clamp", extrapolateRight: "clamp" });
  return (
    <div style={{ position: "absolute", left: 0, right: 0, bottom: "12%", textAlign: "center", fontFamily: "Inter", fontWeight: 800, fontSize: 90, color: "#ffffff", opacity: shown }}>
      {text}
    </div>
  );
}
