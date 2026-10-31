# media

`raea-io-media`: decoding and encoding behind one seam (`Backend`, in [`io/codec/`](../codec)), so an edit's clips, voice and music never depend on
one tool. A decoder answers "the frame showing at time t", so frames come back the same in any order of requests.
Muxing is pure Rust everywhere (muxide, vendored in `vendor/muxide`).

| Backend (crate) | What it uses |
|---|---|
| `apple` ([`io/apple/`](../apple), [`io/apple-coding/`](../apple-coding)) | AVFoundation and VideoToolbox: HEVC and H.264 decode, H.264, HEVC and AAC encode |
| `soft` ([`io/soft/`](../soft)) | portable software: openh264, re_mp4, Symphonia, Opus |
| `av1` ([`io/av1/`](../av1)) | dav1d decode, rav1e encode |
| `ffmpeg` ([`io/ffmpeg/`](../ffmpeg)) | the ffmpeg CLI, as a baseline for tests |

Audio is handled as columns: decoding, mixing, resampling and loudness are raea's own code.
[`docs/licences.feature`](docs/licences.feature) records what each backend links and under which licence.

Used by [`engine/render/`](../../engine/render), [`sdk/react/`](../../sdk/react) and [`systems/footage/`](../../systems/footage).
