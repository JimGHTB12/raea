// raea's MaterialExtensions over a StandardMaterial: grid, paint, wear, shader and toon, each an ExtendedMaterial's
// `extension` (materials.tsx turns them into the old material's props).
import type { ReactNode } from "react";
import type { TrackSpec } from "@raea/math/animate";
import type { SurfaceShaderSpec, ToonBand, ToonHighlight, ToonOutline, ToonRim } from "../3d";
import type { MeshStandardMaterialProps } from "../three";

// A fixes an `animate` to the fields T names: any other key is never, so a typo fails in tsc, not at render
export type Only<A, T> = { [K in keyof A]: K extends keyof T ? T[K] : never };

/** raea's extension of StandardMaterial (a Bevy MaterialExtension of raea's): graph paper over the surface for
 *  greybox levels, lines every `cell` world units (fixed in the world, so they run on across shapes), a heavier one
 *  every `major` cells (4), in `color`, and each face of a Box outlined in `edge` ("" for no outlines). Its colours are
 *  the first frame's. It goes in an ExtendedMaterial's `extension`. */
export interface GridExtensionProps {
  /** world units between grid lines */
  cell: number;
  /** cells between heavier lines (4) */
  major?: number;
  /** the lines' colour */
  color?: string;
  /** the colour each Box face is outlined in ("" for none) */
  edge?: string;
}

/** raea's grid over a StandardMaterial, as an ExtendedMaterial's extension (engine/render grid.rs):
 * @bevy bevy_pbr::mesh_material::MeshMaterial3d<bevy_pbr::extended_material::ExtendedMaterial<bevy_pbr::pbr_material::StandardMaterial, raea_render::engine::grid::Grid>> */
export function GridExtension(_: GridExtensionProps): ReactNode {
  throw new TypeError("GridExtension goes in an ExtendedMaterial: <ExtendedMaterial base={<StandardMaterial />} extension={<GridExtension cell={0.5} />} />");
}

/** raea's extension of StandardMaterial for a paint look (<Look mode="paint">): the surface broken up by world-space
 *  noise of `scale` metres (1), its tone boundaries wandering by `breakup` (0.5) and patches of it taking `color`
 *  (`amount`, 0.6, of the way): a mottled field, rust, a wall's uneven paint. It goes in an ExtendedMaterial's
 *  `extension`. */
export interface PaintExtensionProps {
  /** how far tone boundaries wander (0.5) */
  breakup?: number;
  /** the noise's size in metres (1) */
  scale?: number;
  /** the colour patches take */
  color?: string;
  /** how far toward `color` the patches go (0.6) */
  amount?: number;
}

export function PaintExtension(_: PaintExtensionProps): ReactNode {
  throw new TypeError("PaintExtension goes in an ExtendedMaterial: <ExtendedMaterial base={<StandardMaterial />} extension={<PaintExtension scale={0.5} />} />");
}

/** raea's extension of StandardMaterial that wears it (the engine's Wear MaterialExtension, systems/wear): its paint
 *  mottled, chipped back to `chip_color` on edges and a few spots, `grime` gathered low and underneath and `rust` run
 *  down in streaks, fixed in the part's own space as it moves. `amount` 0-1; `chips`, `grime` and `rust` each 0-1 of it
 *  (0.6, 0.5, 0.25); `scale` a blotch's size in metres (0.08); `seed` moves the pattern. It goes in an
 *  ExtendedMaterial's `extension`. */
export interface WearExtensionProps {
  /** how worn it is, 0-1 */
  amount: number;
  /** a blotch's size in metres (0.08) */
  scale?: number;
  /** chips on edges and a few spots, 0-1 of the wear (0.6) */
  chips?: number;
  /** grime low and underneath, 0-1 of the wear (0.5) */
  grime?: number;
  /** rust run down in streaks, 0-1 of the wear (0.25) */
  rust?: number;
  /** moves the pattern */
  seed?: number;
  /** what chips show beneath the paint */
  chip_color?: string;
  /** the grime's colour */
  grime_color?: string;
  /** the rust's colour */
  rust_color?: string;
}

/** raea's chips, grime and rust over a StandardMaterial, as an ExtendedMaterial's extension (systems/wear):
 * @bevy bevy_pbr::mesh_material::MeshMaterial3d<bevy_pbr::extended_material::ExtendedMaterial<bevy_pbr::pbr_material::StandardMaterial, raea_sys_wear::Wear>> */
export function WearExtension(_: WearExtensionProps): ReactNode {
  throw new TypeError("WearExtension goes in an ExtendedMaterial: <ExtendedMaterial base={<StandardMaterial />} extension={<WearExtension amount={0.6} />} />");
}

// the old material's `wear` from WearExtension's fields (its colours camelCase there)
export function wearOf(w: WearExtensionProps): MeshStandardMaterialProps["wear"] {
  const { chip_color, grime_color, rust_color, ...rest } = w;
  return { ...rest, ...(chip_color !== undefined ? { chipColor: chip_color } : {}), ...(grime_color !== undefined ? { grimeColor: grime_color } : {}), ...(rust_color !== undefined ? { rustColor: rust_color } : {}) };
}

/** raea's extension of StandardMaterial written in WESL (the engine's ShaderExt, docs/shaders.feature SH1): its
 *  `fn surface` runs before Bevy's lighting and its `fn lit` after it, on the base's inputs, so the surface stays lit
 *  and shadowed. raea/3d's <MeshStandardMaterial shader> is the same thing; the spec is SurfaceShaderSpec. It goes in an
 *  ExtendedMaterial's `extension`, alone.
 *  @bevy bevy_pbr::extended_material::ExtendedMaterial<bevy_pbr::pbr_material::StandardMaterial, raea_sys_shader::ShaderExt> */
export type ShaderExtensionProps = SurfaceShaderSpec;

export function ShaderExtension(_: ShaderExtensionProps): ReactNode {
  throw new TypeError("ShaderExtension goes in an ExtendedMaterial: <ExtendedMaterial base={<StandardMaterial />} extension={<ShaderExtension wesl={src} />} />");
}


/** raea's extension of StandardMaterial that cel-shades it (the engine's ToonExt; three.js's MeshToonMaterial): the
 *  scene's lights in `steps` hard bands (2; `bands` for an explicit ramp, darkest first), the bottom one
 *  `shadow_color` (the base colour darker, nudged towards violet), with an optional rim, highlight and ink outline
 *  (true for their defaults). Its base gives base_color, base_color_texture, emissive, alpha_mode and material; a
 *  toon surface has no roughness or metal. It goes in an ExtendedMaterial's `extension`, alone. */
export interface ToonExtensionProps {
  /** how many hard bands the light falls into (2) */
  steps?: number;
  /** an explicit ramp of bands, darkest first, in place of `steps` */
  bands?: ToonBand[];
  /** the bottom band's colour (the base colour darker, nudged towards violet) */
  shadow_color?: string;
  /** a rim light on the silhouette: true for its default, or its settings */
  rim?: boolean | ToonRim;
  /** a hard specular highlight: true for its default, or its settings */
  highlight?: boolean | ToonHighlight;
  /** an ink outline: true for its default, or its settings */
  outline?: boolean | ToonOutline;
  /** how bright the rim is */
  rim_intensity?: number;
  /** how bright the highlight is */
  highlight_intensity?: number;
  /** the outline's width */
  outline_width?: number;
  /** its fields' tracks, as a StandardMaterial's `animate` (its base animates base_color and emissive) */
  animate?: ToonExtensionAnimate;
}

export interface ToonExtensionAnimate {
  /** a track of `shadow_color`, sampled each frame */
  shadow_color?: TrackSpec<string>;
  /** a track of `rim_intensity`, sampled each frame */
  rim_intensity?: TrackSpec<number>;
  /** a track of `highlight_intensity`, sampled each frame */
  highlight_intensity?: TrackSpec<number>;
  /** a track of `outline_width`, sampled each frame */
  outline_width?: TrackSpec<number>;
}

export function ToonExtension<A>(_: ToonExtensionProps & { animate?: Only<A, ToonExtensionAnimate> }): ReactNode {
  throw new TypeError("ToonExtension goes in an ExtendedMaterial: <ExtendedMaterial base={<StandardMaterial />} extension={<ToonExtension steps={3} />} />");
}
