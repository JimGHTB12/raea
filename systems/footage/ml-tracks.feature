Feature: On-device models: which licences ship, and the tracks they write
  Restored 2026-10-24 from engine/html/docs/ml-tracks.feature (deleted with engine/html in f1b4a17e) because
  perception.feature, meaning.feature and the roadmap rely on its licence rule and its track contract. Its parts about
  the old HTML engine (fx.js markup, the renderer's mask-image and feImage work, the 10-09 owners) are left out; git
  history has them. The licence facts are as surveyed on 2026-10-09.

  Rule: Models and licences
    Not usable: Depth Anything 3 Large/Giant/Nested and Video Depth Anything Base/Large (CC BY-NC 4.0), Apple Depth
    Pro (research-only weights). Every model added later needs its weights licence checked first.

    Scenario: The bundled constructs
      Given the constructs the apps ship
        | Construct                        | Model                                     | Weights licence | Size                                                                                         | Runtime |
        | Face landmarks + blendshapes     | MediaPipe face landmarker                 | Apache-2.0      | 4.8 MB (detector 0.29 + landmarks 2.61 + blendshapes 1.86)                                   | converted 2026-10-10: ~/the host app/models/mediapipe-face-landmarker (validated vs the .tflite: landmarks r 0.9999996). In the apps 2026-10-10: Perception.swift `face` track (weighted NMS, rotated crop, followed from the last frame's points); vs MediaPipe on reference/ 0.34–0.42% of face width (bar 0.6%); ~8 ms/frame on M4 |
        | Hair / part segmentation         | MediaPipe hair (+ multiclass selfie later) | Apache-2.0      | 0.62 MB                                                                                      | converted 2026-10-10: ~/the host app/models/mediapipe-hair-segmenter (mask IoU 0.998–0.999 vs MediaPipe). In the apps 2026-10-10: Perception.swift `hair` track; reference/ masks reproduced to 0 levels, IoU 0.9977–0.9989 vs MediaPipe; ~26 ms/frame on M4 |
        | Depth                            | Depth Anything 3 Small (Base later)       | Apache-2.0      | 50.8 MB fp16                                                                                 | converted 2026-10-09: ~/the host app/models/depth-anything-3-small (r ≥ 0.999998) |
        | Tracked object masks             | EdgeTAM                                   | Apache-2.0      | 36.7 MB (image encoder 10.0, mask decoder 11.0, memory encoder 8.8 fp32, memory attention 6.9) | converted 2026-10-10 incl. the memory parts: ~/the host app/models/edgetam (Core ML-only tracking vs Meta's predictor: mean IoU 0.996 click / 0.995 box; ~86 ms/frame on M4). In the apps 2026-10-10: Perception.swift `object-<name>` track from a box or point at `at` s, forward only; vs the official predictor on reference/ mean IoU 0.992–0.993, min 0.98 (bar 0.99/0.97), judged at the video's size (half-size mattes cost ~1% on the small bird); ~150–280 ms/frame on a busy M4 |
        | Person matte, pose, box tracking | Apple Vision                              | OS              | 0                                                                                            | Vision  |
      Then the apps bundle the first four models and use Apple Vision for the rest

  Rule: More small models worth shipping (survey 2026-10-09)
    The user asked which other small models would help, within a total under 2 GB. Weights licences were checked;
    sizes are on-disk estimates.

    Scenario: Candidate models and their plan
      Given the surveyed models
        | Model              | Use                                                                 | Licence                                               | Size                                  | Port                                | Plan                                                       |
        | DeepFilterNet3     | denoise any clip; clean audio before ASR                            | MIT/Apache-2.0                                        | 2 MB                                  | Core ML; Rust libDF                 | ship now                                                   |
        | Beat This! (small) | beats/downbeats: cut to the beat, GSAP cue points (a `beats` track) | MIT                                                   | ~10 MB                                | ONNX; pure-Rust crate               | ship now                                                   |
        | HTDemucs v4        | strip/duck music under voice, isolate vocals                        | MIT (MUSDB training data is research-licensed: note) | ~80 MB                                | MLX + Swift; Core ML (GPU)          | ship now on Mac, iPhone after a device benchmark           |
        | LaMa               | remove objects from stills; clean plates with EdgeTAM masks         | Apache-2.0                                            | 187 MB                                | Core ML (~2 s/image on iPhone 13 Pro) | ship now (stills, clean plates; per-frame video flickers) |
        | SigLIP2-B/16       | text-to-footage search for the Agent                                | Apache-2.0                                            | 356 MB (image encoder alone 92 MB)    | Core ML                             | ship now                                                   |
        | pyannote community-1 | speaker labels, active-speaker reframe                            | CC-BY-4.0 weights (attribution)                       | ≤129 MB                               | Core ML + Swift (FluidAudio)         | later                                                      |
        | TransNetV2         | cut/dissolve detection                                              | MIT                                                   | 31 MB                                 | ONNX (convert)                      | later (Perception detects shots). Converted 2026-10-14: ~/.local/share/raea-models/transnetv2, official TF weights -> PyTorch -> ONNX 31.2 MB (per-frame probability max diff 5.2e-7 vs TF over 1,074 real frames); ~12-22 ms/frame ONNX Runtime CPU on M4. Loses to ffmpeg on Buddy's renders: it misses jump cuts within a take (meaning.feature, Cuts) |
        | Audio2Face-3D      | lip-sync the 3D avatar from TTS                                     | NVIDIA Open Model License                             | 158 MB                                | MLX                                 | later                                                      |
        | Magenta RT2 Small  | generated music beds                                                | CC-BY-4.0                                             | ~450 MB                               | MLX                                 | later, Mac first                                           |
      Then the ship-now set is about 635 MB, or about 370 MB with only SigLIP2's image encoder on device

    Scenario: Use Apple's APIs instead of a model
      Given a job one of Apple's APIs already does
        | job                                              | Apple API                                                     |
        | translation                                      | Translation framework                                         |
        | captions/hooks/titles                            | Foundation Models, iOS 26 + Apple Intelligence devices        |
        | slow motion/upscaling/denoise/optical flow       | VideoToolbox `VTFrameProcessor`, iOS 26 / macOS 15.4          |
        | best-frame scoring                               | `CalculateImageAestheticsScoresRequest`                       |
        | person/foreground mattes                         | Vision                                                        |
        | loudness                                         | plain EBU R128                                                |
      Then the apps use that API and ship no model for it

    Scenario: Models not usable because of their licence
      Not usable (licence): Robust Video Matting (GPL-3.0), CodeFormer and ProPainter (S-Lab non-commercial), GFPGAN
      (non-commercial NVIDIA code), MusicGen (CC-BY-NC), madmom models (CC BY-NC-SA), Apple FastVLM and MobileCLIP2
      (research-only), NLLB-200 (CC-BY-NC); Stable Audio Open Small/3 ends above $1M annual revenue; LivePortrait only
      after replacing its non-commercial InsightFace detector; Wav2Lip unverified, reported non-commercial.
      Given a model on this list
      Then it is not shipped

  Rule: Track formats are the contract between the writers and the renderer
    Every track is a folder in the project: `tracks/<key>/<kind>[-<label>]/`, where `<key>` is the clip's project
    path with `/` written as `-` and without a leading `assets/` (the key previews and facts use; every take's video
    is `takes/<id>/video.mov`, so bare file names would collide): `tracks/takes-take-20261009-1100-video.mov/person/`,
    `tracks/IMG_0412.mov/object-mug/`. Each folder holds `track.json` plus the data; `source` in it is the clip's
    project path. Writers: Perception.track in the apps and `ugc-perceive track <video> <kind> <dir> <source>`
    (the host app-a2); the Agent asks for a track with the `footage` action "track".

    Frame sequences (mattes and depth) are PNG files rather than alpha video. No alpha codec works in Chrome, WebKit
    and ffmpeg alike (VP9-alpha needs libvpx and doesn't play in WebKit; HEVC-alpha doesn't play in Chrome), but a
    PNG used as `mask-image` or an SVG `feImage` renders the same everywhere.

    Scenario: Data and pixels per track kind
      Given a track folder
        | Kind                                       | Data                              | Pixels                                                                                                                                                                                          |
        | `person`, `hair`, `object-<label>` (mattes) | `000000.png`, `000001.png`, …     | 8-bit gray+alpha (PNG colour type 4): gray 255, alpha = matte (255 = subject). Half the clip's resolution, rounded to even, same aspect.                                                         |
        | `depth`                                    | `000000.png`, …                   | 8-bit grayscale (colour type 0), 255 = nearest, 0 = farthest: relative inverse depth, normalized with one min/max for the whole clip (not per frame), so it doesn't flicker. Half resolution. |
        | `face`, `hands`, `pose` (landmarks)        | `track.json` only                 | see the samples scenario                                                                                                                                                                        |
        | `box-<label>` (tracked boxes)              | `track.json` only                 | see the samples scenario                                                                                                                                                                        |
      Then a composition reads the frames as images (a matte as a mask, depth as a map)

    Scenario: The common fields of track.json
      Frame `i` covers media time `start + i / fps` in the source clip: the clip's own time, before any trimming or
      `data-media-start` in a composition. `fps` is the clip's frame rate, capped at 30 (a 60 fps clip uses every
      second frame). `width`/`height` are the PNGs' size; landmark and box tracks give the clip's size.
      Given a track folder
      Then its track.json holds these common fields:
        """json
        {
          "kind": "person",
          "label": null,
          "source": "footage/take-03.mov",
          "model": "apple-vision-person@iOS18 | edgetam@1 | depth-anything-3-small@1 | mediapipe-face@0.10",
          "fps": 30,
          "start": 0.0,
          "frames": 412,
          "width": 540,
          "height": 960
        }
        """

    Scenario: Landmark and box samples
      Landmarks and boxes add `"samples"`, one per frame, with coordinates normalized 0–1 in the clip's frame (x
      right, y down) and `null` where nothing was found. Face points follow MediaPipe's 478-point mesh order, hands
      its 21 points per hand (`"hands": [{"side": "left", "points": [...]}]`), and pose its 33 points (`"points"`
      with a 4th value, visibility). A `box-<label>` sample is `{box, score}`. The renderer and fx.js read only these
      fields, so writers may add more.
      Given a landmark or box track
      Then its track.json adds samples like these:
        """json
        "samples": [
          { "faces": [ { "box": [0.31, 0.18, 0.36, 0.27], "points": [[0.42, 0.29, -0.01], ...],
                         "blendshapes": { "jawOpen": 0.12, "eyeBlinkLeft": 0.03 } } ] },
          { "box": [0.12, 0.40, 0.20, 0.22], "score": 0.94 },
          null
        ]
        """
