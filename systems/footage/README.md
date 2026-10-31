# footage

`raea-sys-footage`: reading footage the way an editor does, before it is used: shots and how each begins, framing,
faces, the subject, on-screen text, speech, music and silence, camera motion, and the person's performance over time.
It writes a fact sheet (`facts.json`) beside the footage, and on request mattes and object tracks.

It stands on four crates beside it: [`systems/layout/`](../layout) (a frame's layout, the framing coach, legibility),
[`systems/vision/`](../vision) (Apple Vision and Core ML models), [`systems/song/`](../song) (beats, stems, speech
and cuts through ONNX models) and [`systems/perceive/`](../perceive) (the same Vision trait on open models). The
`raea-perceive` command ([`tools/perceive/`](../../tools/perceive)) runs each part from a shell.
