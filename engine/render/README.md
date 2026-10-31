# raea-render

`raea-render`: raea's render engine. `raea` compositions are lowered to its program (rows spawned once, keyed per
frame) by [`sdk/react/`](../../sdk/react) and drawn here, so a render and a live play draw from the same rows with the
same code.

- `src/engine.rs` and `src/engine/`: the Bevy renderer: 3D scenes (meshes, lights, cameras, glTF models, post
  effects), video frames, the painted pages as image rows (`src/rows2d.rs`, drawn by bevy_ui) and sound.
- `src/gpuview.rs`: the direct GPU path a render encodes from.
- `src/html.rs`: HTML compositions, drawn by bevy_ui through [`runtime/js/`](../raea-runtime-js).
- `src/cli.rs` and `src/media_verbs.rs`: native media verbs (`raea media ...`, plus `raea ffmpeg` and
  `raea ffprobe`, which read ffmpeg-style command lines without running ffmpeg). raea and the shell run
  them in process.

Depends on [`runtime/js/`](../raea-runtime-js), [`systems/k/`](../../systems/k), [`io/media/`](../../io/media) and
[`gpu/device/`](../../gpu/device).

Status: in use as the renderer behind `raea`.
