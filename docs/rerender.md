# Why agents stitch clips with ffmpeg, and what makes re-rendering the easy path (research, 2026-11-10)

Reader: whoever plans raea's render and export work. Shane's note (11-10): agents using raea keep rendering clips and
joining them with ffmpeg. Should raea get fast enough that re-rendering is easier, or is something else going on?

**Finding: speed is only part of it. The main cause is that one raea render can't change its look or camera, and it
can't re-render part of a video.** A film with more than one look has to be several renders, and the only partial
re-render is one the agent builds by hand: a composition per chunk, a folder of finished chunks, and ffmpeg to join
them. An infinitely fast renderer wouldn't remove the first cause. Without a partial cache, a fast renderer still
redraws every frame for a one-shot change.

## The evidence

**What agents use ffmpeg for.** 5,833 distinct ffmpeg command lines in Claude Code sessions on raea (~/raea, ~/Studio,
the feed test projects). The raea app's own agents' transcripts aren't included. One command can count under several
purposes:

| Purpose | Commands | Native raea today |
|---|---:|---|
| Contact sheets and side-by-sides (tile, hstack, vstack, xstack) | 2,627 | `raea media sheet`; no side-by-side compare |
| Pulling frames to look at | 1,778 | `raea media still`, `raea render --at` |
| Seeking and trimming (`-ss`) | 796 | `raea media trim` |
| ffprobe | 546 | `raea media probe` |
| Encoding a PNG sequence into a video | 412 | none: `--alpha`, `--aov` and neural passes write PNG folders |
| Re-encoding (x264, x265, ProRes) | 357 | `--quality` on the render |
| Loudness | 319 | `raea media loudness`, `<Mix>` mastering |
| **Joining clips** (concat, xfade) | 122 | `raea media concat`, which re-encodes; ffmpeg's `-c copy` doesn't |
| Colour changes over time (eq, hue, curves) | 114 | none inside one render (below) |
| Laying sound under picture | 109 | `raea render --audio` and `-o OUT.wav` (landed 11-10) |
| Text and overlays | 79 | the composition itself |

Most of the ffmpeg use is agents **looking at** their output, not assembling it. The stitching is a smaller count, but
every multi-shot film project has it.

**The clearest case: the launch film** (`~/Studio/raea/projects/v0-1-launch/promo/stage`, 120.6 s, 3,618 frames at
1920x1080). Its source explains itself:

- `camera.ts`: "an engine render holds its first frame's look (grade, tones, haze) for its whole length, so the chunks
  split at every cut, every change of look and every light cue." That gave 80 chunk compositions (`fm-0` to `fm-79`).
- `film.sh`: renders the chunks three at a time (`xargs -P 3`), keeps any chunk already on disk ("a chunk already
  rendered is kept unless FRESH=1"), joins them with `ffmpeg -f concat -c copy`, then uses ffmpeg again for the sound,
  a limiter, and the colour drain on "black and white", "since the engine holds Grade for a render, so the drain is
  ffmpeg's hue". The final encode is x264 at CRF 16.
- Cost (antsy history): a full pass took 31-33 min with three renders at once, about 0.55 s of wall time a frame.
  Redoing three chunks took 2-5 min. 27 runs, 14% failed, peak 14.2 GB.

The anything film and the opening cuts did the same: each shot rendered to its own MP4, joined with ffmpeg, or placed
in a cut composition that plays the rendered MP4s.

## The causes, checked in the code

1. **A render holds one look and one camera.** `runtime/look/src/camera.rs` reads atmosphere, bloom, fog, depth of
   field, grade, SSAO, anti-aliasing, shadows, volumetric fog and tonemapping "as the first frame that has each asks",
   once for the whole render. Only `<Grade aberration>` was moved to a per-frame column, and only after it bit someone
   ("it was the first frame's, for every frame"). The skill teaches agents the same rule: "One camera (the first), one
   set of effects (the first frame's)" (skills `3d.md` rule 6, `3d/limits.md`, `3d/post.md`). Any film whose look
   changes therefore has to be split into renders.
2. **No partial re-render.** The export cache is keyed on the whole video (`runtime/bundle/src/cache.rs`), so changing
   one shot redraws every frame. `raea render` has no frame range for a video (`--at` writes stills only). Agents got a
   range by making each chunk a `<Composition>` and a cache by keeping the chunk files.
3. **Heavy 3D frames are slow, so partial redo matters.** Motion graphics are fast (a 20 s edit in 2.5 s, capped by
   the encoder), but a film's 3D frame costs about 1.6 s per render process. That's an estimate from the launch film's
   33 min for 3,618 frames on three processes, and it includes 80 process start-ups. Where the time goes inside a
   frame hasn't been measured (step S0 below).
4. **No parallelism across a render.** One render is one process. Agents parallelize with `xargs -P`.
5. **raea's join re-encodes.** `raea media concat` decodes and re-encodes (`apps/media/src/encode.rs`). ffmpeg's
   `-c copy` joins in a moment with no quality loss, so agents pick ffmpeg.
6. **Off the Mac, raea encodes in software.** `io/media` has a hardware encoder only for Apple (VideoToolbox); on
   Windows and Linux H.264 is openh264 on the CPU ("other platforms slot in behind the same trait"). Windows renders
   are encoder-bound and lower quality, which gives agents another reason to reach for ffmpeg there.
7. **Habit, and the skill points there.** The skill's share-copy line is an ffmpeg command (`SKILL.md:249`), and
   models know ffmpeg's flags better than raea's verbs.

## What to build, in order

Each step has a test that says it's done. The order follows the causes: features before speed.

**S0. Measure a heavy frame on Mac and Windows.** Render a few hundred frames of a heavy 3D scene with
`RAEA_FRAME_LOG` and `RAEA_TIMES`, on the Mac and on the Windows PC (NVIDIA). Split the time into start-up, React/JS,
Bevy CPU, GPU, readback and encode. This decides S6's order. Needs no new code.

**S1. Looks and cameras change per frame.** Move every effect in `camera.rs` from read-once to a per-frame column, as
aberration already is. Values like grade, bloom strength, fog colour and depth of field are per-view uniforms in Bevy,
so they're cheap to change. Mode switches like the tonemapper, AA mode or shadow filter are pipeline variants: build
them ahead of time so a mid-render switch doesn't stall. Let the active camera change per frame. Update the skill's
rules to match.
- Done when: one composition with a look change at every cut renders frame-equal (SSIM 1.0) to the same film rendered
  as chunks, and a colour ramp is keyframes, not ffmpeg.

**S2. Shot-level incremental renders.** Key each shot (a `<Sequence>`, or a range between cuts) on what its frames
depend on: the baked rows for those frames, the assets, the render settings. Encode each shot as its own segment
starting on a keyframe, and assemble the MP4 by remuxing segments without re-encoding (raea has a remuxer in
`io/codec` and muxide). A changed shot redraws only its frames, and the join takes milliseconds. This depends on frames
being pure functions of their number, which raea already holds itself to (render = play).
- Done when: changing one shot of a 120 s film costs that shot's frames plus under a second, and the result equals a
  full render.

**S3. Ranges and parallel work inside one render.** `raea render --frames A-B` for videos. A long render splits
itself into workers at shot boundaries, sized to the GPU and memory, and joins them with S2's remux. No more
`xargs -P`.

**S4. Native verbs for the rest, and a skill that uses them.** `raea media concat` stream-copies when the inputs
match. Add a side-by-side compare (before/after, A/B takes), since that's the commonest ffmpeg use. Sound goes under
picture through `--audio` and `<Mix>`. The skill's ffmpeg share line becomes a raea command. The skill change follows
raea-skills' own process (its SOP and checks).
- Done when: a replayed film session needs no ffmpeg command.

**S5. Hardware encode on Windows and Linux.** On Windows, Media Foundation's H.264/HEVC encoders, which cover
NVIDIA, AMD and Intel GPUs through one API. On Linux, VAAPI or NVENC. Feed them GPU textures without a CPU copy, as
L2 (lightspeed) wants on the Mac.
- Done when: a Windows 1080p render is no longer encoder-bound, at the Mac's export quality.

**S6. Raw frame speed, after S0.** The GPU work in `docs/gpu.md`, ordered by S0's split. A proposed target to set
after measuring: a 5 s shot of a heavy scene re-renders in under 30 s on an M-series Mac and on a mid-range NVIDIA PC,
and a half-resolution preview with upscaling in a few seconds.

## What this doesn't solve

- Footage and other people's video still come in as files, and raea's media verbs handle those. This is about raea's
  own renders.
- Agents will still use ffmpeg out of habit for a while. S4's replay check is what shows the habit is gone.
