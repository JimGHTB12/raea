---
title: "raea/asset"
summary: "Files the project holds and assets made for it: staticFile, generated sound and music, and glTF bytes."
order: 46
sources: ["sdk/raea/src/asset.ts"]
---

`staticFile` names a file in the project, so the renderer finds it wherever the composition runs. The generated
assets (`sao` for a sound effect, `tokensynthNote` for an instrument's note, `lyria` for music, `fish` for speech)
are made the first time a render needs them and kept in the project's asset store, so later renders reuse them.

```tsx
import { Audio } from "raea";
import { sao } from "raea/asset";

function Door() {
  return <Audio src={sao("a heavy wooden door creaks open", { seconds: 3 }).src} />;
}
```

<!-- generated:begin by tools/manual-ref/sdk.ts from the SDK's exports and doc comments; edit outside this block -->

```ts
import { fish, glbBytes, isGeneratedAsset, linearRgba } from "raea/asset"
```

`raea/asset` exports 9 functions, 1 value, 8 types. Bevy's counterpart: `bevy_asset`. Source: [sdk/raea/src/asset.ts](https://github.com/shinyobjectz/raea/blob/main/sdk/raea/src/asset.ts).

## Functions

### `fish`

Retired on 2026-11-05: Fish narration is no longer part of raea. Narrate with `<Speech text start>` from
raea/speech (Pocket TTS, on this computer), or make a line with a `raea gen` step on `fish:<model>` and your own
FISH_API_KEY. Kept as a name only, so an older edit says what to use instead of failing to bundle.

```ts
function fish(text: string, _options?: Record<string, unknown>): never
```

### `glbBytes`

The scene as glTF binary bytes.

```ts
function glbBytes(scene: GlbScene): Uint8Array<ArrayBufferLike>
```

### `isGeneratedAsset`

Whether a value is a generated asset (what sao, tokensynthNote and lyria return), not a file path.

```ts
function isGeneratedAsset(v: unknown): v is GeneratedAsset
```

### `linearRgba`

sRGB hex to linear rgba

```ts
function linearRgba(c: string | Rgba, alpha?: number): Rgba
```

### `lyria`

Music an older edit generated (30 seconds by default). Retired: a take already made plays, a new one is refused.

```ts
function lyria(prompt: string, options?: { seconds?: number; }): GeneratedAsset
```

### `sao`

A sound effect to generate (2 seconds by default).

```ts
function sao(prompt: string, options?: { seconds?: number; }): GeneratedAsset
```

### `staticFile`

The URL of a file in the space's media folder, each path segment URL-encoded.

```ts
function staticFile(path: string): string
```

### `tokensynthNote`

One note of an instrument described in words, rendered by TokenSynth (local, CPU): a Sampler's root note.

```ts
function tokensynthNote(timbre: string, midi: number, options?: { seconds?: number; velocity?: number; seed?: number; precision?: "fp32" | "fp16" | "bf16" | "int8" | "coreml"; }): GeneratedAsset
```

### `writeGlb`

The scene written as a.glb named by its content (once per process), its path: a `<Model src>`. Under plain bun
(no engine) nothing is written and the would-be path is still returned, so a composition can be checked.

```ts
function writeGlb(scene: GlbScene): string
```

## Values

### `MEDIA_ROOT`

Where the space's media folder is served from.

## Types

### `AssetKind`

```ts
type AssetKind = "lyria" | "sao" | "tokensynth"
```

### `GeneratedAsset`

| Field | Type | What it is |
| --- | --- | --- |
| `kind` (required) | `AssetKind` | which generator makes it: lyria (music), sao (sound effects) or tokensynth (an instrument's note) |
| `prompt` (required) | `string` | what to make, in words |
| `seconds` (required) | `number` | how long it is |
| `id` (required) | `string` | a stable hash of kind, prompt and seconds |
| `src` (required) | `string` | "asset:<kind>/<id>" |
| `options` | `Record<string, unknown>` | the generator's own settings (a TokenSynth note's midi, velocity, seed and device) |

### `GlbClip`

| Field | Type | What it is |
| --- | --- | --- |
| `name` (required) | `string` | the clip's name, as a Model's \`animation\` picks it |
| `times` (required) | `ArrayLike<number>` | sample times (any unit the Model's `time` uses) |
| `rotations` (required) | `ArrayLike<number>[]` | per joint, a quaternion x y z w per sample: the joint's rotation in its parent's frame |
| `translations` | `Record<number, ArrayLike<number>>` | per root joint (by joint index), a translation offset x y z per sample, added to its rest translation |

### `GlbJoint`

| Field | Type | What it is |
| --- | --- | --- |
| `name` (required) | `string` | the joint's name (a bone's, as a retarget maps it) |
| `parent` (required) | `number` | index of the parent joint, -1 for a root |
| `head` (required) | `[number, number, number]` | the joint's head in world space at rest (its node's rest translation is head - parent's head) |

### `GlbMaterial`

| Field | Type | What it is |
| --- | --- | --- |
| `color` | `string \| Rgba` | hex (#rrggbb) or linear rgba 0..1 |
| `unlit` | `boolean` | drawn without light (KHR_materials_unlit) |
| `blend` | `boolean` | alpha-blended (the colour's alpha, times the vertex colours' alpha) |
| `doubleSided` | `boolean` | draw the back faces too |
| `roughness` | `number` | 0..1, smooth to matte |
| `metalness` | `number` | 0..1, a dielectric to a metal |
| `emissive` | `string` | the colour it glows with |

### `GlbPrimitive`

| Field | Type | What it is |
| --- | --- | --- |
| `positions` (required) | `ArrayLike<number>` | x y z per vertex |
| `normals` | `ArrayLike<number>` | x y z per vertex, unit length |
| `colors` | `ArrayLike<number>` | r g b a per vertex, linear 0..1 (multiplies the material's colour) |
| `indices` | `ArrayLike<number>` | by threes for triangles, by twos for lines; none: every vertex in order |
| `mode` | `"triangles" \| "lines"` | what the indices draw: triangles (the default) or lines |
| `joints` | `ArrayLike<number>` | four joint indices per vertex (a skinned mesh) |
| `weights` | `ArrayLike<number>` | four weights per vertex, summing to 1 |
| `material` | `GlbMaterial` | its surface: colour, roughness, metalness, glow |

### `GlbScene`

| Field | Type | What it is |
| --- | --- | --- |
| `primitives` (required) | `GlbPrimitive[]` | the meshes it holds |
| `joints` | `GlbJoint[]` | a skin: all primitives are skinned by these joints when it's given |
| `clips` | `GlbClip[]` | animation clips over the joints |

### `Rgba`

```ts
type Rgba = [number, number, number, number]
```

<!-- generated:end -->
