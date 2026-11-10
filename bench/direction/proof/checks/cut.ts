// A cut lands on a word's end (DR4's second proof: a footage edit). At each cut it reads the outgoing clip's last frame
// and the incoming clip's first against the words their take's transcript has: a cut through a word is found. No
// render: the edit and the transcript say it all.
import { check, found, type Clip } from "raea/check";

export default check({ id: "cut", text: "a cut lands on a word's end", frames: "cuts" }, (world, frame) => {
  const fps = world.resource<{ fps: number }>("Frame")!.fps;
  const tol = 1 / fps;
  for (const c of world.find({ with: ["Clip"] })) {
    const clip = world.component<Clip>(c, "Clip")!;
    // where this clip is cut, if it is cut on this frame: its out on its last frame, its in on its first
    const last = clip.out != null && frame === clip.at + Math.round((clip.out - clip.in) * fps) - 1;
    const points: [string, number][] = [];
    if (last) points.push(["out of", clip.out!]);
    if (frame === clip.at && clip.in > 0) points.push(["into", clip.in]);
    for (const [how, at] of points) {
      const through = clip.words.find((w) => w.start + tol < at && at < w.end - tol);
      if (through) {
        found({ about: [c], says: `the cut ${how} the take at ${at.toFixed(2)} s falls inside "${through.text}" (${through.start.toFixed(2)} to ${through.end.toFixed(2)} s)`, measure: { name: "into the word", value: Math.round((at - through.start) * 1000), unit: "ms" }, fix: `cut at ${through.end.toFixed(2)} s, its end` });
      }
    }
  }
});
