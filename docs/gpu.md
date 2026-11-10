# The GPU layer on Mac, Windows and Linux: what raea should own (research, 2026-11-10)

Reader: whoever decides how much of the GPU layer raea owns. The question (Shane, 11-10): Vork and Rune each built
their own Metal layer; should raea write its own Metal shaders or GPU kernels, and render as fast as possible? His
follow-up the same day: plan for Mac and Windows users (and Linux), not the iPhone first. The companion note
`docs/rerender.md` covers why agents re-render less than they should. Its finding comes first: the biggest render
costs today are features (looks held for a whole render, no partial re-render), not the GPU layer.

**Verdict: keep wgpu as the base on every platform (Metal on Mac, DX12 or Vulkan on Windows, Vulkan on Linux), and own
no backend. Own kernels and platform accelerators behind one interface, each with a portable fallback, on the one
shared device.** wgpu 30 already accepts each platform's own shader code (MSL on Metal, HLSL or DXIL on DX12, SPIR-V
on Vulkan, as passthrough) and hands out the native device for interop (`as_hal`, as `raytrace/metalfx.rs` does).
Rune's own research priced owning Vulkan and DX12 at 5-7x the size of owning Metal, and kept wgpu there. An owned Metal
backend stays an option for the Mac only, on a measured trigger.

## The platforms, seam by seam

| Seam | Mac | Windows | Linux | Portable fallback |
|---|---|---|---|---|
| Graphics API (wgpu) | Metal | DX12 (Vulkan possible) | Vulkan | n/a |
| Hardware video encode | VideoToolbox (have) | Media Foundation: NVIDIA, AMD and Intel encoders through one API (to build) | VAAPI / NVENC (to build) | openh264, rav1e (have; CPU) |
| Upscaling for previews | MetalFX temporal scaler | DLSS on NVIDIA (Bevy's `dlss` feature; NVIDIA's SDK terms to check, `docs/render-thirdparty.md`) | DLSS on NVIDIA | a lower render scale, plain resample |
| Denoising ray-traced light | MetalFX denoised scaler (have, RT4) | DLSS Ray Reconstruction on NVIDIA | same | Open Image Denoise (Apache-2.0; CPU, CUDA, HIP, SYCL, Metal), for offline film frames |
| Owned kernels | MSL when Apple-only hardware pays (f16 simdgroup matrices, Metal 4 tensors) | HLSL/DXIL or WGSL | SPIR-V or WGSL | WGSL through K: runs on all three |
| Pipeline cache across launches | none in wgpu: precompile `.metallib`s | driver cache | wgpu `PIPELINE_CACHE` (Vulkan only) | n/a |

## What raea already does natively

- `gpu/device`: one shared wgpu device for Bevy, Vello and K kernels. On Apple it uses `objc2-metal` for the canary
  command buffer's status (`check.rs`) and its own `CAMetalLayer` (`present.rs`).
- `engine/render/raytrace/src/metalfx.rs`: MetalFX's denoiser on Solari's light, in a command buffer of its own
  recorded through wgpu's Metal backend. This is the pattern for every seam above.
- Zero-copy video on Apple: decoder IOSurfaces wrapped by `CVMetalTextureCache` as wgpu textures
  (`engine/render/docs/g8.feature`).
- `systems/k`: K kernels compiled to WGSL, bit-exact against their CPU interpreter, so they already run on every
  platform.
- The native target (`engine/rune`) draws through Rune's own Metal backend on macOS (Rune's default features).

## The two precedents

| | Rune (anomalyco, MIT; in raea's Cargo.lock) | Vork (voyager-so, source-available research preview) |
|---|---|---|
| What it is | Native UI framework | Headless 3D and VFX engine for agents |
| What it owns | A Metal backend under `wgpu::custom` on macOS; wgpu on Windows, Linux, web and (by default) iOS | Its renderer and its shaders, not the GPU layer. It runs on a vendored wgpu 29 with about 20 patches (`vendor/PATCHES.md`). Its kernels are Slang compiled to SPIR-V (Vulkan, Windows included) and MSL (Metal), loaded through wgpu's passthrough shaders; agents' Slang programs go to WGSL. `ash` and `objc2-metal` handle native interop (OIDN shares Vulkan memory) |
| Size | 4,872 lines in `crates/rune/src/gpu/metal` and `backend.rs`, tests included | 238 `.slang` files; about 20 wgpu patches |
| Why | CPU per frame. 120 Hz HUD on an M2 Max: 894 vs 1,395 kinstr/frame with no buffer writes, 897 vs 1,668 with one; 1 command buffer vs 6-7 | Offline film quality: raster visibility plus hardware ray-traced light, 1.1-2.3x faster than EEVEE (its own timings, not equal quality), with OIDN |
| Left out | Depth and stencil, MSAA, indirect draws, render bundles, binding arrays: all panic and name the wgpu fallback | Volumes; iOS; GPUs without ray queries |

Rune's win is ~0.14-0.5 ms of CPU a frame on light UI frames. Vork's win is offline ray-traced quality on desktop and
datacenter GPUs. A raea film frame costs about 1.6 s per render process (`docs/rerender.md`), so neither win is where
raea's time goes. Rune also tested the claim that its Metal backend used 330 MB against wgpu's 1.5 GB, and it didn't
hold. In paired runs the two were within a few MB; the gap came from an unpaced headless loop.

## Vork's wgpu patches are raea's to-do list

Vork hit the same wgpu gaps raea has, and patched them in its vendored copy rather than leaving wgpu. Its licence is a
source-available preview licence, so raea writes its own fixes (upstream first, `vendor/` if needed) instead of
copying Vork's:

- **Metal command-buffer errors go unchecked.** raea hit this as GPU faults that never reported. Vork checks every
  buffer and reports the device loss from `poll` instead of panicking.
- **Fast math in MSL.** Vork compiles naga's and passthrough MSL with safe math, so frames don't depend on how a
  variant was compiled. This matters for raea's Q5 (Mac = iPhone, byte for byte).
- **No Metal pipeline cache.** Vork persists Metal pipelines as an `MTLBinaryArchive` through wgpu's `PipelineCache`.
  That's G3.
- **Buffer binding arrays on Metal** (Tier-2 argument buffers). This is the bindless feature that once blocked Solari
  on Metal.
- **Passthrough MSL invariance**, so a depth prepass and the forward pass agree exactly.

## Alternatives to wgpu, and why none fits raea

Bevy's renderer is built on wgpu, so any of these means leaving Bevy's renderer, against "completely Bevy":

- **Each platform's own API**: `ash` (Vulkan), `objc2-metal` (Metal), `windows` (D3D12). This is full control and a
  backend per platform to own. Zed and Rune do it on the Mac only.
- **Blade** (Vulkan, Metal, GLES; thin). Zed used it on Linux and returned to wgpu in January 2026.
- **Vulkan everywhere, translated to Metal by MoltenVK.** This loses Metal-only features (MetalFX, memoryless tile
  memory, Metal 4).
- **C/C++ hardware layers** (SDL3 GPU, slang-rhi, The Forge, bgfx, Diligent, Dawn). Each needs FFI and gives up
  Rust-side safety. Dawn is another WebGPU implementation, so it has the same model as wgpu.

The route both Vork and Rune took is to keep wgpu and go native underneath it where it pays. That's this note's plan.

## What wgpu 30 already reaches (checked in wgpu-hal / wgpu-types 30.0.1)

| Feature | Where | Used by Bevy 0.20.1 / raea |
|---|---|---|
| Passthrough shaders (`PASSTHROUGH_SHADERS`) | MSL or `.metallib` on Metal; HLSL or DXIL on DX12; SPIR-V on Vulkan | No |
| `TRANSIENT_ATTACHMENT` | Memoryless tile memory on Apple GPUs: depth and MSAA with no backing store | No Bevy crate sets it |
| `PIPELINE_CACHE` | Vulkan only | n/a |
| `EXPERIMENTAL_COOPERATIVE_MATRIX` | Metal: 8x8 f32 only; Vulkan where the driver offers 8x8 f32 | No |
| `SHADER_F16`, `SUBGROUP`, `EXPERIMENTAL_MESH_SHADER`, ray queries | Varies by backend | Solari uses ray queries |
| Metal 4 (new command model, ML in shaders) | wgpu-hal detects the family but drives the classic API | Only through our own Metal calls |
| MSL compile options | Left at Metal's default (fast math) | Owned kernels can choose |

## Plan, with gates

**G0. Profile a heavy film frame on the Mac and on the Windows PC (NVIDIA).** This is `docs/rerender.md` S0: a few
hundred frames with `RAEA_FRAME_LOG` and `RAEA_TIMES`, split into start-up, React/JS, Bevy CPU, wgpu, GPU per pass,
readback and encode. Everything below is ordered by that table.

**G1. A preview tier.** Render at a lower scale and upscale with the platform's upscaler (MetalFX on Mac, DLSS on
NVIDIA, a plain resample elsewhere), for agents' look-checks and drafts. Full quality stays the default for delivery.
- Done when: a draft of a heavy shot renders at least 3x faster on both machines, and the agent's checks still flag
  the same problems on it.

**G2. Owned kernels, portable first.** Image and look kernels go in K, which compiles to WGSL and runs everywhere
with its CPU interpreter as the reference. A kernel that needs hardware-specific operations (the neural-shading net:
f16 matrix tiles on Apple, cooperative matrices or tensor cores elsewhere) is written once in Slang. Slang compiles to
MSL, HLSL and SPIR-V, each loaded as wgpu passthrough on its backend, with a slow WGSL or CPU reference beside it. Every
kernel runs on `gpu/device`'s device and queue.
- Done when, per kernel: it beats its reference by at least 1.5x on both the Mac and the Windows PC, or it's deleted.

**G3. No cold pipeline compiles.** On Metal, capture the MSL naga writes for each pipeline a project uses (Rune's
capture approach), compile `.metallib`s at bundle time, and look them up at pipeline creation. That needs a small
wgpu-hal patch carried in `vendor/` with its reason written down. On Vulkan, turn on wgpu's `PIPELINE_CACHE`. On DX12,
measure first: the driver cache may be enough.
- Done when: a cold first render or first play of an exported game is at least 5x faster to its first frame on each
  platform.

**G4. Denoising for offline film frames.** Open Image Denoise on every platform as the portable denoiser for
ray-traced film light. MetalFX (have) and DLSS Ray Reconstruction stay as the fast paths where they exist.

**G5. An owned Metal backend: Mac only, only on a trigger.** Start it only if G0 shows wgpu-core + wgpu-hal + driver
dispatch at 25% or more of a render's CPU on the Mac after G1-G3. Start from Rune's MIT backend, and keep wgpu as the
fallback and oracle in the same binary. Bevy needs depth, MSAA, indirect draws, binding arrays, timestamps and
acceleration structures, which Rune's backend lacks. No owned DX12 or Vulkan backend: Rune measured those at 5-7x
Metal's size, and Zed returned to wgpu on Linux.

## Later: phones and tablets

The same seams carry over to iOS: the iPhone 13 Pro Max (A15) has run raea (golden scenes at 43-88 ms a frame on
10-16, titles at 525 ms, peak 373 MB against a 200 MB target). The phone-specific levers are memoryless depth and MSAA
(`TRANSIENT_ATTACHMENT`, which Bevy doesn't set yet), MetalFX upscaling, a lighter post stack, and precompiled
pipelines (G3). Hardware ray tracing starts at A17 Pro / M3. A15 profiling waits until Mac and Windows are done.

## Gotchas

- One device: owned kernels and accelerators run on `gpu/device`'s device and queue, never a second device, or
  textures can't be shared without copies.
- wgpu's Metal backend reports no command-buffer error, and a GPU fault on this Mac has needed a reboot (10-26). Every
  owned command buffer checks its status, as `check.rs`'s canary does. Test failures by injecting them.
- Determinism: wgpu compiles MSL with Metal's default fast math, so cross-device byte equality (Q5) can only be chased
  in kernels whose compile options we set.
- The Mac's GPU is shared by every session: keep kernel benches small and run them one at a time.
- `as_hal` is unstable API: each wgpu bump re-checks the seams.

Sources: github.com/voyager-so/vork (README, `Cargo.toml`, `crates/vork-render`, `crates/vork-slang`,
`vendor/PATCHES.md`, `vendor/wgpu-hal/PROVENANCE.md`, read 2026-11-10); github.com/anomalyco/rune `research/34-own-gpu-plan.md` and
`crates/rune/src/gpu/metal/mod.rs` at 7ead7d6; wgpu-hal 30.0.1 `src/metal/{adapter,device}.rs`; wgpu-types 30.0.1
`src/features.rs`; github.com/RenderKit/oidn (Apache-2.0; Metal since 2.2).
