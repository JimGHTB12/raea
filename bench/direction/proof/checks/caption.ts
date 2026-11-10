// A caption clears the platform's button band (DR4's fourth proof: a vertical ad). Every frame's 2D text, by the box
// its words were laid in: no render needed. `band`: where the band starts, down the frame (0.8 when unset).
import { check, found, type Layer, type Text } from "raea/check";

export default check({ id: "caption", text: "a caption clears the platform's button band" }, (world, _frame, values) => {
  const band = Number(values.band ?? 0.8);
  for (const t of world.find({ with: ["Text", "Layer"] })) {
    if (world.component<Layer>(t, "Layer")!.medium !== "2d-type") continue;
    const box = world.component<Text>(t, "Text")!.box;
    if (box && box[3] > band) {
      found({ about: [t], says: `the caption reaches ${Math.round(box[3] * 100)}% down the frame, into the button band from ${Math.round(band * 100)}%`, measure: { name: "bottom", value: Math.round(box[3] * 100), unit: "%", limit: Math.round(band * 100) } });
    }
  }
});
