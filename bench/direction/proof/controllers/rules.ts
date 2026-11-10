// The proof project's rules (docs/direction.feature DR4): each a rule field naming the check that holds it, a .ts
// check by its path from here, with its values. `raea check FILE.tsx --only ID` runs one.
import { controller } from "raea/brief";

export default controller({
  title: "Proof rules",
  groups: [
    {
      title: "What the pieces are held to",
      fields: [
        { label: "Sung lines are flat 2D type, inside the frame, behind the subject", type: "rule", check: "../checks/lyric.ts" },
        { label: "Cuts land on a word's end", type: "rule", check: "../checks/cut.ts" },
        { label: "Cues are heard where they are placed", type: "rule", check: "../checks/cue.ts" },
        { label: "Captions clear the button band", type: "rule", check: "../checks/caption.ts", values: { band: 0.8 } },
        { label: "The rider turns with its seat and is in no part of it", type: "rule", check: "../checks/rider.ts", values: { rider: "rider", seat: "seat" } },
      ],
    },
  ],
});
