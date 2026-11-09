---
title: "raea/color"
summary: "Bevy's colour types: Srgba, LinearRgba, Hsla, Oklab, Oklch and the rest, with their conversions."
order: 47
sources: ["sdk/raea/src/color.ts"]
---

These are Bevy's colour types with Bevy's maths, so a colour mixed, lightened or converted here comes out as it
would in Bevy. Anywhere a prop takes a colour, it also takes a CSS colour string.

```ts
import { LinearRgba, Srgba, kelvin } from "raea/color";

const brand = Srgba.hex("#e8b04a");
const hover = brand.lighter(0.1);
const glow = new LinearRgba(4, 2.5, 1, 1); // brighter than white, for bloom
const warm = kelvin(3200);
```

<!-- generated:begin by tools/manual-ref/sdk.ts from the SDK's exports and doc comments; edit outside this block -->

```ts
import { Hsla, kelvin, LinearRgba, Okhsla } from "raea/color"
```

`raea/color` exports 2 functions, 7 classes. Bevy's counterpart: `bevy_color`. Source: [sdk/raea/src/color.ts](https://github.com/shinyobjectz/raea/blob/main/sdk/raea/src/color.ts).

## Functions

### `kelvin`

A colour temperature as a light's colour (Tanner Helland's fit to the Planckian locus), sRGB hex.

```ts
function kelvin(k: number): string
```

### `toHex`

A colour as the 3D renderer takes it (#rrggbb): hex, rgb()/rgba() or a common name; CSS variables throw.

```ts
function toHex(owner: string, k: string, v: unknown): string
```

## Classes

### `Hsla`

Bevy's Hsla: hue (degrees), saturation and lightness, each 0..1; converted through Hsva and Hwba, as Bevy's is.

Bevy: `bevy_color::hsla::Hsla`.

| Member | What it does |
| --- | --- |
| `new Hsla(hue: number, saturation: number, lightness: number, alpha?: number)` | Hue in degrees (0 to 360), saturation, lightness and alpha 0 to 1. |
| `Hsla.from(c: Rgb): Hsla` | The same colour from an Srgba or LinearRgba. |
| `.hue: number` | Bevy: The hue channel. [0.0, 360.0] |
| `.saturation: number` | Bevy: The saturation channel. [0.0, 1.0] |
| `.lightness: number` | Bevy: The lightness channel. [0.0, 1.0] |
| `.alpha: number` | Bevy: The alpha channel. [0.0, 1.0] |
| `.to_srgba(): Srgba` | The same colour as an Srgba. |
| `.to_linear(): LinearRgba` | The same colour as a LinearRgba. |
| `.mix(o: Hsla, t: number): Hsla` | The colour `t` (0 to 1) of the way to `o`, mixed in this space. |

Used in: [ui/scroll_and_overflow/drag_to_scroll](https://github.com/shinyobjectz/raea/blob/main/sdk/raea/bench/bevy-parity/ui/scroll_and_overflow/drag_to_scroll.tsx), [ui/ui_material](https://github.com/shinyobjectz/raea/blob/main/sdk/raea/bench/bevy-parity/ui/ui_material.tsx), [ui/widgets/feathers_gallery](https://github.com/shinyobjectz/raea/blob/main/sdk/raea/bench/bevy-parity/ui/widgets/feathers_gallery.tsx), [animation/color_animation](https://github.com/shinyobjectz/raea/blob/main/sdk/raea/bench/bevy-parity/animation/color_animation.tsx), [2d/2d_shapes](https://github.com/shinyobjectz/raea/blob/main/sdk/raea/bench/bevy-parity/2d/2d_shapes.tsx), [shader_advanced/custom_shader_instancing](https://github.com/shinyobjectz/raea/blob/main/sdk/raea/bench/bevy-parity/shader_advanced/custom_shader_instancing.tsx).

### `LinearRgba`

Bevy's LinearRgba: linear light, which a StandardMaterial's emissive is; a channel past 1 is brighter than white.

Bevy: `bevy_color::linear_rgba::LinearRgba`.

| Member | What it does |
| --- | --- |
| `new LinearRgba(red: number, green: number, blue: number, alpha?: number)` | Linear red, green, blue (above 1 for brighter than white) and alpha. |
| `LinearRgba.rgb(red: number, green: number, blue: number): LinearRgba` | An opaque linear colour. |
| `LinearRgba.from(c: Srgba): LinearRgba` | Bevy's LinearRgba::from(Srgba) |
| `.red: number` | Bevy: The red channel. [0.0, 1.0] for SDR colors. |
| `.green: number` | Bevy: The green channel. [0.0, 1.0] for SDR colors. |
| `.blue: number` | Bevy: The blue channel. [0.0, 1.0] for SDR colors. |
| `.alpha: number` | Bevy: The alpha channel. [0.0, 1.0] |
| `.to_srgba(): Srgba` | Bevy's Srgba::from(LinearRgba): gamma-encoded (a channel at or below 0 kept as it is) |
| `.luminance(): number` | Bevy's Luminance::luminance: Rec. 709's weights over the linear channels |
| `.darker(amount: number): LinearRgba` | Bevy's Luminance::darker: mixed toward black until its luminance is `amount` lower (at least 0) |
| `.lighter(amount: number): LinearRgba` | Bevy's Luminance::lighter: mixed toward white until its luminance is `amount` higher (at most 1) |
| `.mix(o: LinearRgba, t: number): LinearRgba` | Bevy's Mix: each channel lerped, in f32 |
| `.mul(k: number): LinearRgba` | Bevy's `* k`: each colour channel scaled (alpha kept) |

Used in: [3d/spotlight](https://github.com/shinyobjectz/raea/blob/main/sdk/raea/bench/bevy-parity/3d/spotlight.tsx), [3d/bloom_3d](https://github.com/shinyobjectz/raea/blob/main/sdk/raea/bench/bevy-parity/3d/bloom_3d.tsx), [3d/mesh_ray_cast](https://github.com/shinyobjectz/raea/blob/main/sdk/raea/bench/bevy-parity/3d/mesh_ray_cast.tsx), [animation/color_animation](https://github.com/shinyobjectz/raea/blob/main/sdk/raea/bench/bevy-parity/animation/color_animation.tsx), [2d/mesh2d_vertex_color_texture](https://github.com/shinyobjectz/raea/blob/main/sdk/raea/bench/bevy-parity/2d/mesh2d_vertex_color_texture.tsx), [shader_advanced/custom_shader_instancing](https://github.com/shinyobjectz/raea/blob/main/sdk/raea/bench/bevy-parity/shader_advanced/custom_shader_instancing.tsx).

### `Okhsla`

Bevy's Okhsla: Oklab's hue (degrees), a saturation relative to the sRGB gamut, and a lightness with a toe, each
0..1.

Bevy: `bevy_color::Okhsla`.

| Member | What it does |
| --- | --- |
| `new Okhsla(hue: number, saturation: number, lightness: number, alpha?: number)` | Hue in degrees, saturation, lightness and alpha 0 to 1, in Okhsl. |
| `Okhsla.from_oklaba(lab: Oklaba): Okhsla` | From Oklab (bevy_color oklab_to_okhsl). |
| `Okhsla.from(c: Srgba \| LinearRgba \| Hsla): Okhsla` | From any sRGB colour, through Oklab. |
| `.hue: number` | Bevy: The hue channel. [0.0, 360.0] |
| `.saturation: number` | Bevy: The saturation channel. [0.0, 1.0] |
| `.lightness: number` | Bevy: The lightness channel. [0.0, 1.0] |
| `.alpha: number` | Bevy: The alpha channel. [0.0, 1.0] |
| `.to_oklaba(): Oklaba` | To Oklab (bevy_color okhsl_to_oklab). |
| `.to_linear(): LinearRgba` | The same colour as a LinearRgba. |
| `.to_srgba(): Srgba` | The same colour as an Srgba. |
| `.with_alpha(alpha: number): Okhsla` | The same colour with this alpha. |

Used in: [ui/widgets/feathers_gallery](https://github.com/shinyobjectz/raea/blob/main/sdk/raea/bench/bevy-parity/ui/widgets/feathers_gallery.tsx).

### `Oklaba`

Bevy's Oklaba: perceptual lightness and two opponent axes.

Bevy: `bevy_color::oklaba::Oklaba`.

| Member | What it does |
| --- | --- |
| `new Oklaba(lightness: number, a: number, b: number, alpha?: number)` | Perceptual lightness, a (green to red), b (blue to yellow) and alpha. |
| `Oklaba.from(c: Oklcha \| Rgb): Oklaba` | The same colour from an Srgba, LinearRgba or Oklcha. |
| `.lightness: number` | Bevy: The 'lightness' channel. [0.0, 1.0] |
| `.a: number` | Bevy: The 'a' channel. [-1.0, 1.0] |
| `.b: number` | Bevy: The 'b' channel. [-1.0, 1.0] |
| `.alpha: number` | Bevy: The alpha channel. [0.0, 1.0] |
| `.to_linear(): LinearRgba` | The same colour as a LinearRgba. |
| `.to_srgba(): Srgba` | The same colour as an Srgba. |
| `.mix(o: Oklaba, t: number): Oklaba` | The colour `t` (0 to 1) of the way to `o`, mixed in this space. |

Used in: [animation/color_animation](https://github.com/shinyobjectz/raea/blob/main/sdk/raea/bench/bevy-parity/animation/color_animation.tsx).

### `Oklcha`

Bevy's Oklcha: Oklab's lightness, chroma and hue (degrees).

Bevy: `bevy_color::oklcha::Oklcha`.

| Member | What it does |
| --- | --- |
| `new Oklcha(lightness: number, chroma: number, hue: number, alpha?: number)` | Perceptual lightness, chroma, hue in degrees and alpha. |
| `Oklcha.from(c: Rgb \| Oklaba): Oklcha` | The same colour from an Srgba, LinearRgba or Oklaba. |
| `.lightness: number` | Bevy: The 'lightness' channel. [0.0, 1.0] |
| `.chroma: number` | Bevy: The 'chroma' channel. [0.0, 1.0] |
| `.hue: number` | Bevy: The 'hue' channel. [0.0, 360.0] |
| `.alpha: number` | Bevy: The alpha channel. [0.0, 1.0] |
| `.to_linear(): LinearRgba` | The same colour as a LinearRgba. |
| `.to_srgba(): Srgba` | The same colour as an Srgba. |
| `.mix(o: Oklcha, t: number): Oklcha` | The colour `t` (0 to 1) of the way to `o`, mixed in this space. |
| `.lighter(amount: number): Oklcha` | Bevy's Luminance::lighter: the lightness raised by `amount`, at most 1 |
| `.darker(amount: number): Oklcha` | Bevy's Luminance::darker: the lightness lowered by `amount`, at least 0 |
| `.with_alpha(alpha: number): Oklcha` | Bevy's Alpha::with_alpha |

Used in: [animation/color_animation](https://github.com/shinyobjectz/raea/blob/main/sdk/raea/bench/bevy-parity/animation/color_animation.tsx).

### `Srgba`

Bevy's Srgba: gamma-encoded colour, each channel 0..1.

Bevy: `bevy_color::srgba::Srgba`.

| Member | What it does |
| --- | --- |
| `new Srgba(red: number, green: number, blue: number, alpha?: number)` | sRGB red, green, blue and alpha, each 0 to 1. |
| `Srgba.hex(hex: string): Srgba` | Bevy's Srgba::hex: rgb, rgba, rrggbb or rrggbbaa (the # optional), and also rgb() or a common name |
| `.red: number` | Bevy: The red channel. [0.0, 1.0] for SDR colors. |
| `.green: number` | Bevy: The green channel. [0.0, 1.0] for SDR colors. |
| `.blue: number` | Bevy: The blue channel. [0.0, 1.0] for SDR colors. |
| `.alpha: number` | Bevy: The alpha channel. [0.0, 1.0] |
| `.darker(amount: number): Srgba` | Bevy's Luminance::darker on an Srgba: its linear colour's, encoded again |
| `.lighter(amount: number): Srgba` | Bevy's Luminance::lighter on an Srgba: its linear colour's, encoded again |
| `.with_alpha(alpha: number): Srgba` | The same colour with this alpha. |
| `.to_hex(): string` | Bevy's Srgba::to_hex: #RRGGBB (and AA when not opaque), each channel clamped and rounded |
| `.mix(o: Srgba, t: number): Srgba` | Bevy's Mix: each channel lerped, in f32 |

Used in: [ui/images/image_node](https://github.com/shinyobjectz/raea/blob/main/sdk/raea/bench/bevy-parity/ui/images/image_node.tsx), [ui/images/image_node_resizing](https://github.com/shinyobjectz/raea/blob/main/sdk/raea/bench/bevy-parity/ui/images/image_node_resizing.tsx), [ui/images/ui_texture_atlas_slice](https://github.com/shinyobjectz/raea/blob/main/sdk/raea/bench/bevy-parity/ui/images/ui_texture_atlas_slice.tsx), [ui/images/ui_texture_slice](https://github.com/shinyobjectz/raea/blob/main/sdk/raea/bench/bevy-parity/ui/images/ui_texture_slice.tsx), [ui/images/ui_texture_atlas](https://github.com/shinyobjectz/raea/blob/main/sdk/raea/bench/bevy-parity/ui/images/ui_texture_atlas.tsx), [ui/layout/grid](https://github.com/shinyobjectz/raea/blob/main/sdk/raea/bench/bevy-parity/ui/layout/grid.tsx), [ui/layout/ghost_nodes](https://github.com/shinyobjectz/raea/blob/main/sdk/raea/bench/bevy-parity/ui/layout/ghost_nodes.tsx), [ui/layout/flex_layout](https://github.com/shinyobjectz/raea/blob/main/sdk/raea/bench/bevy-parity/ui/layout/flex_layout.tsx), and 62 more.

### `Xyza`

Bevy's Xyza: CIE 1931 XYZ (D65, as linear sRGB's).

Bevy: `bevy_color::xyza::Xyza`.

| Member | What it does |
| --- | --- |
| `new Xyza(x: number, y: number, z: number, alpha?: number)` | CIE XYZ and alpha. |
| `Xyza.from(c: Rgb): Xyza` | The same colour from an Srgba or LinearRgba. |
| `.x: number` | Bevy: The x-axis. [0.0, 0.9505] for SDR colors. |
| `.y: number` | Bevy: The y-axis, intended to represent luminance. [0.0, 1.0] for SDR colors. |
| `.z: number` | Bevy: The z-axis. [0.0, 1.0889] for SDR colors. |
| `.alpha: number` | Bevy: The alpha channel. [0.0, 1.0] |
| `.to_linear(): LinearRgba` | The same colour as a LinearRgba. |
| `.to_srgba(): Srgba` | The same colour as an Srgba. |
| `.mix(o: Xyza, t: number): Xyza` | The colour `t` (0 to 1) of the way to `o`, mixed in this space. |

Used in: [animation/color_animation](https://github.com/shinyobjectz/raea/blob/main/sdk/raea/bench/bevy-parity/animation/color_animation.tsx).

<!-- generated:end -->
