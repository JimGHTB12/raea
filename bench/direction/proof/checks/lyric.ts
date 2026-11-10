// A sung line is 2D type, inside the frame, with the subject in front of it (docs/direction.feature DR4's first proof:
// a React layer over footage). It runs where a line is sung and reads what covers it, so it needs a render with
// --layers; the line's text is the Text whose words a Line on the frame sings.
import { check, found, type Covers, type Layer, type Line, type Text } from "raea/check";

const plain = (s: string) => s.replace(/[^a-z0-9 ]/gi, "").replace(/\s+/g, " ").trim().toLowerCase();

export default check({ id: "lyric", text: "a sung line is 2D type, inside the frame, with the subject in front of it", needs: "render", frames: "lines" }, (world, _frame, values) => {
  const sung = world.find({ with: ["Line"] }).map((l) => plain(world.component<Line>(l, "Line")!.text));
  const margin = Number(values.margin ?? 0.03);
  for (const t of world.find({ with: ["Text"] })) {
    const text = world.component<Text>(t, "Text")!;
    if (!sung.includes(plain(text.words))) continue;
    const layer = world.component<Layer>(t, "Layer");
    if (layer?.medium !== "2d-type" || text.extruded > 0) {
      found({ about: [t], says: `the line is ${layer?.medium ?? "3d"} type${text.extruded > 0 ? `, ${text.extruded} m deep` : ""}: it is to be flat 2D type` });
    }
    const b = text.box;
    if (b && (b[0] < margin || b[1] < margin || b[2] > 1 - margin || b[3] > 1 - margin)) found({ about: [t], says: "the line runs to the frame's edge" });
    const covered = world.find({ with: ["Covers"] }).some((c) => (world.component<Covers>(c, "Covers") ?? []).some((x) => x.layer === t && x.share > 0));
    if (!covered) found({ about: [t], says: "nothing is in front of the line: the subject is to mask it" });
  }
});
