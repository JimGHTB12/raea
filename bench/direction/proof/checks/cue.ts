// A cue is heard within a frame of where it is placed (DR4's third proof: a mix). It reads each cue's Onset, measured
// on the render's mix, so it needs a render. That the cue is placed on its cause (a hit, a door) is DR21's CausedBy;
// this holds the half the mix alone can: a cue whose sound starts late, or never, is found.
import { check, found, type Onset } from "raea/check";

export default check({ id: "cue", text: "a cue is heard within a frame of where it is placed", needs: "render" }, (world) => {
  const fps = world.resource<{ fps: number }>("Frame")!.fps;
  const limit = Math.round(1000 / fps);
  for (const c of world.find({ with: ["Cue", "Onset"] })) {
    const o = world.component<Onset>(c, "Onset")!;
    if (o.late_ms == null) found({ about: [c], says: "nothing is heard where the cue is placed" });
    else if (Math.abs(o.late_ms) > limit) {
      found({ about: [c], says: `it is heard ${Math.abs(o.late_ms)} ms ${o.late_ms > 0 ? "after" : "before"} where it is placed`, measure: { name: "late", value: o.late_ms, unit: "ms", limit } });
    }
  }
});
