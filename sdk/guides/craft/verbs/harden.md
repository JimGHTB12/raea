# harden

Notes like these: "it broke when I…", "what if two people press", "it gets stuck", "edge case". A game's or a tool's
edges.

## Assess
- List the inputs nobody designed for: two keys at once, holding a key, pressing during a transition, losing focus,
  resizing, the very first frame and the last, a very long name, nothing at all.
- Reproduce the note's case first, live, with real input.

## Change
- Fix the cause in the pure step (`useGameState`), not a guard in the view.
- Every state has a picture: empty, loading, error, won, lost.
- Keep it deterministic, so a logged play renders back to the same pixels.

## Verify
- Play the case again live, then the cases next to it. Run the replay check.
