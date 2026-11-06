# optimize

Notes like these: "it's slow", "the file's too big", "it stutters", "the game lags", "it took forever to render".
The same piece, lighter: smaller files, faster renders, smooth play.

## Assess
- Measure first: the export's size and bitrate (`raea media probe`), the render's time, a game's frame rate live.
- Find the cost: footage at a far higher resolution than the frame, 3D lights and shadows nobody sees, a model with
  more triangles than pixels, per-frame work that could be computed once.

## Change
- Delivery: `--quality` per use (`raea guide rendering`); a share copy for sending (the skill's ffmpeg line).
- Footage: `raea mezz` for long-GOP sources; trim to the parts used.
- 3D: fewer shadow-casting lights, smaller shadow maps, decimated models (`raea guide bevy/blender`), the limits in
  `raea guide bevy/limits`.
- Code: compute what doesn't change outside the frame function, keyed by index, still frame-pure.

## Verify
- The same measurements, before and after. The look is unchanged in stills at the same frames.
