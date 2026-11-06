# adapt

Notes like these: "we need it in 16:9", "a square one for the feed", "4:5 for Instagram", "a 6 second cut", "for
YouTube". The same piece made right for another size, length or platform; never a crop of the old one.

## Assess
- The controllers' delivery specs name the sizes, lengths and platforms; work to those.
- For each size, find what no longer fits: type that wraps, a face cut off, a part in a platform's button zone, a
  hold that's too long for a shorter cut.

## Change
- Register one `<Composition>` per size in the same root, sharing the parts; each lays out for its own frame
  (`raea guide motion/compositions`). Never scale or crop one size into another.
- Re-place type and faces for each frame and its safe zones (`raea guide motion/placement`). Wider frames get room,
  not bigger type.
- A shorter cut keeps the hook and the ending and drops whole beats from the middle, cut on words.
- Name each file for its size: `spring-launch-16x9.mp4`.

## Verify
- `raea render --composition ID --at 0,45,89` stills for every size; `raea check` on each export.
- Every size says the same thing; nothing is cut off or crowded in any of them.
