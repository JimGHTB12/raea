// Image planes, decals, bloom, fog, grading, environment, ambient occlusion, shadows and volumetric fog.
import { useCurrentFrame, useVideoConfig } from "@raea/app/clock";
import { ImagePlaneProps, Vec3, assetPath, checkShares, checkVec, colours, node3d } from "./all";

/** A picture on a flat card in the scene, unlit (a poster, a screen, a sprite).
 * @bevy bevy_image::image::Image */
export function ImagePlane(props: ImagePlaneProps) {
  if (typeof props.src !== "string" || props.src === "") throw new TypeError("ImagePlane: src must be a picture (staticFile(\"x.png\"))");
  const gr = props.grid;
  if (gr !== undefined && !(Array.isArray(gr) && gr.length === 2 && gr.every((n) => Number.isInteger(n) && n >= 1))) throw new TypeError(`ImagePlane: grid is [columns, rows], two whole numbers of 1 or more, got ${JSON.stringify(gr)}`);
  if (props.frame !== undefined && !(Number.isFinite(props.frame) && props.frame >= 0)) throw new RangeError(`ImagePlane: frame is a cell of the grid, 0 or more, got ${props.frame}`);
  for (const k of ["position", "rotation"] as const) if (props[k] !== undefined) checkVec("ImagePlane", k, props[k]);
  return node3d("ImagePlane", props as unknown as Record<string, unknown>);
}

export interface DecalProps {
  /** the picture pressed onto the surface: staticFile("label.png"); its clear parts show the surface through */
  src: string;
  name?: string;
  normalMap?: string;
  /** width by height in metres (a number: a square) */
  size?: number | [number, number];
  position?: Vec3;
  /** which way the surface faces at the decal (out of it; [0, 1, 0], a floor or counter, by default): turns the decal
   * so the picture lies on it, upright on a wall, its top toward -z on a floor. Instead of rotation */
  normal?: Vec3;
  /** a turn about the normal (radians) */
  spin?: number;
  /** radians, the decal's +y out of the surface (rather than normal) */
  rotation?: Vec3;
  /** a tint times the picture (keyed: a stain darkening) */
  color?: string;
  /** 0 to 1 (keyed: a label fading in) */
  opacity?: number;
  roughness?: number;
  metalness?: number;
  unlit?: boolean;
  /** how far off its plane (metres) it still lies on a surface, fading to nothing there (0.05 by default). Anything
   * nearer the plane than this takes it, a cabinet's side or a fridge's as well as the wall: keep it small */
  fade?: number;
}

/** The turn that takes +y to `normal`, with the picture's top (a decal's -z) toward world up on a wall, toward -z on a
 * floor, turned `spin` about the normal: a quaternion [x, y, z, w]. */
export function surfaceTurn(normal: Vec3, spin = 0): [number, number, number, number] {
  const len = (v: Vec3) => Math.hypot(v[0], v[1], v[2]);
  const unit = (v: Vec3): Vec3 => { const l = len(v) || 1; return [v[0] / l, v[1] / l, v[2] / l]; };
  const cross = (a: Vec3, b: Vec3): Vec3 => [a[1] * b[2] - a[2] * b[1], a[2] * b[0] - a[0] * b[2], a[0] * b[1] - a[1] * b[0]];
  const Y = unit(normal);
  const up: Vec3 = Math.abs(Y[1]) < 0.99 ? [0, 1, 0] : [0, 0, -1];
  const d = up[0] * Y[0] + up[1] * Y[1] + up[2] * Y[2];
  let Z = unit([-(up[0] - d * Y[0]), -(up[1] - d * Y[1]), -(up[2] - d * Y[2])]);
  let X = cross(Y, Z);
  if (spin) {
    const [c, s] = [Math.cos(spin), Math.sin(spin)];
    [X, Z] = [[X[0] * c - Z[0] * s, X[1] * c - Z[1] * s, X[2] * c - Z[2] * s], [X[0] * s + Z[0] * c, X[1] * s + Z[1] * c, X[2] * s + Z[2] * c]];
  }
  // the rotation matrix's columns X, Y, Z as a quaternion
  const [m00, m11, m22] = [X[0], Y[1], Z[2]];
  const tr = m00 + m11 + m22;
  if (tr > 0) {
    const k = Math.sqrt(tr + 1) * 2;
    return [(Y[2] - Z[1]) / k, (Z[0] - X[2]) / k, (X[1] - Y[0]) / k, k / 4];
  }
  if (m00 > m11 && m00 > m22) {
    const k = Math.sqrt(1 + m00 - m11 - m22) * 2;
    return [k / 4, (Y[0] + X[1]) / k, (Z[0] + X[2]) / k, (Y[2] - Z[1]) / k];
  }
  if (m11 > m22) {
    const k = Math.sqrt(1 + m11 - m00 - m22) * 2;
    return [(Y[0] + X[1]) / k, k / 4, (Z[1] + Y[2]) / k, (Z[0] - X[2]) / k];
  }
  const k = Math.sqrt(1 + m22 - m00 - m11) * 2;
  return [(Z[0] + X[2]) / k, (Z[1] + Y[2]) / k, k / 4, (X[1] - Y[0]) / k];
}

/** A picture pressed onto whatever surface it lies across: a label on a jar, a stain on a counter, tiles on a wall, a
 * sticker on a laptop (Bevy's forward decal; the camera gets a depth prepass for it). Placed and keyed like any part,
 * so it moves and fades with a render's clock and a game's play alike.
 * @bevy bevy_pbr::mesh_material::MeshMaterial3d<bevy_pbr::extended_material::ExtendedMaterial<bevy_pbr::pbr_material::StandardMaterial, bevy_pbr::decal::forward::ForwardDecalMaterialExt>> */
export function Decal(props: DecalProps) {
  if (typeof props.src !== "string" || props.src === "") throw new TypeError("Decal: src must be a picture (staticFile(\"x.png\"))");
  for (const k of ["position", "rotation", "normal"] as const) if (props[k] !== undefined) checkVec("Decal", k, props[k]);
  const { size = 1, normal, spin, ...rest } = props;
  const [w, h] = typeof size === "number" ? [size, size] : size;
  const turn = props.rotation === undefined ? { quaternion: surfaceTurn(normal ?? [0, 1, 0], spin) } : {};
  return node3d("Decal", { ...rest, ...turn, scale: [w, 1, h] } as unknown as Record<string, unknown>);
}

export interface PlateProps {
  /** a video in the media folder: staticFile("clip.mp4") */
  src: string;
  name?: string;
  /** the clip's time on this frame, in seconds (by default the Sequence's: frame / fps) */
  time?: number;
  /** the lens's radial distortion (SIMPLE_RADIAL's k1, from a solve): the plate is drawn undistorted, so the
   * footage's straight lines are straight where the 3D's are */
  k1?: number;
}

/** Footage behind the 3D: the clip's frame filling the camera's whole view (its `view` when it has one), on the far
 * side of everything, unlit. Give the scene `<ToneMapping mode="none" />` to keep its colours exact. <Matchmove> is
 * the usual way in. */
export function Plate(props: PlateProps) {
  const frame = useCurrentFrame();
  const { fps } = useVideoConfig();
  if (typeof props.src !== "string" || props.src === "") throw new TypeError("Plate: src must be a video (staticFile(\"x.mp4\"))");
  const time = props.time ?? frame / fps;
  if (!(Number.isFinite(time) && time >= 0)) throw new RangeError(`Plate: time must be 0 or more seconds, got ${time}`);
  if (props.k1 !== undefined && !(Number.isFinite(props.k1) && Math.abs(props.k1) < 1)) throw new RangeError(`Plate: k1 is a small distortion coefficient, got ${props.k1}`);
  return node3d("Plate", { ...props, time });
}

export interface BloomProps {
  /** 1 is a natural glow, 2 strong; 0 none */
  intensity?: number;
  /** only what's brighter than this glows (0 = everything a little; 1 = only emissive parts and highlights) */
  threshold?: number;
  /** composition passes (a part's `pass`) left out of it: they neither glow nor take the glow */
  skip?: string[];
  /** each named pass's share of it, 0 (as skip) to 1 (all of it, as "scene" has) */
  passes?: Record<string, number>;
}

/** Glow around bright parts (emissive materials, strong highlights). Put it inside the Canvas3D.
 * @bevy bevy_post_process::bloom::settings::Bloom */
export function Bloom(props: BloomProps) {
  if (props.intensity !== undefined && !(props.intensity >= 0)) throw new RangeError(`Bloom: intensity must be 0 or more, got ${props.intensity}`);
  checkShares("Bloom", props.skip, props.passes);
  return node3d("Bloom", props as Record<string, unknown>);
}

export interface FogProps {
  /** hex; match the background so far things fade into it */
  color?: string;
  /** linear fog from `near` to `far` (5 and 20 by default) */
  near?: number;
  far?: number;
  /** or exponential fog of this density (0.05-0.3), instead of near and far */
  density?: number;
}

/** Distance fog: far things fade into `color`. Put it inside the Canvas3D.
 * @bevy bevy_pbr::fog::DistanceFog */
export function Fog(props: FogProps) {
  return node3d("Fog", colours("Fog", props as Record<string, unknown>, ["color"]));
}

export interface ToneMappingProps {
  /** how bright light maps to the screen: "tony" (Bevy's default), "aces" (filmic, contrasty), "agx", "neutral",
   * "reinhard", "none" */
  mode?: "tony" | "aces" | "agx" | "neutral" | "reinhard" | "none";
  /** stops brighter (+) or darker (-) than the default exposure */
  exposure?: number;
}

/** The camera's tone mapping and exposure. Put it inside the Canvas3D.
 * @bevy bevy_render::view::Tonemapping
 * @bevy bevy_camera::camera::Exposure */
export function ToneMapping(props: ToneMappingProps) {
  return node3d("ToneMapping", props as Record<string, unknown>);
}

export interface GradeSection {
  /** 1 keeps the colour; 0 grey; above 1 richer */
  saturation?: number;
  /** 1 keeps it; above 1 harder */
  contrast?: number;
  /** 1 keeps it; above 1 darker mids */
  gamma?: number;
  /** a multiplier: 1 keeps it */
  gain?: number;
  /** added: 0 keeps it, positive lifts the blacks */
  lift?: number;
}

export interface GradeProps {
  /** warmer (+) or cooler (-): a shift of the CIE x chromaticity, about -0.3 to 0.3 */
  temperature?: number;
  /** magenta (+) or green (-) */
  tint?: number;
  /** the whole frame's saturation after tone mapping (1 keeps it) */
  saturation?: number;
  shadows?: GradeSection;
  midtones?: GradeSection;
  highlights?: GradeSection;
  /** the corners darkened: 0 none, 1 black; or {intensity, radius (0.75), smoothness (5), color} */
  vignette?: number | { intensity?: number; radius?: number; smoothness?: number; color?: string };
  /** a lens's colour fringes at edges, as a fraction of the frame (0.005 subtle, 0.02 strong) */
  aberration?: number;
  /** composition passes (a part's `pass`) the grade leaves as drawn: no fringes or vignette on them, and on an
   * untonemapped camera (a paint Look's) no colour grading either */
  skip?: string[];
  /** each named pass's share of the grade, 0 (as skip) to 1 */
  passes?: Record<string, number>;
}

/** The camera's colour grade (lift, gamma, gain and saturation per range of light, before tone mapping), a lens
 * vignette and colour fringes: one grade over every scene, so a film looks of a piece. Put it inside the Canvas3D.
 * @bevy bevy_render::view::ColorGrading
 * @bevy bevy_post_process::effect_stack::chromatic_aberration::ChromaticAberration
 * @bevy bevy_post_process::effect_stack::vignette::Vignette */
export function Grade(props: GradeProps) {
  checkShares("Grade", props.skip, props.passes);
  const v = props.vignette;
  return node3d("Grade", { ...props, vignette: typeof v === "object" && v.color ? { ...v, color: colours("Grade", { color: v.color }, ["color"]).color } : v } as Record<string, unknown>);
}

export interface EnvironmentProps {
  /** an equirectangular panorama in the media folder: a Radiance .hdr (Poly Haven's), an OpenEXR .exr, or an 8-bit
   * .png/.jpg. Change it between frames to cut from one place to another */
  src: string;
  /** how strongly it lights the scene (1 by default: a sky about as bright as an AmbientLight of 2.5 overall); animate
   * it, or set 0 to switch it off for a shot */
  intensity?: number;
  /** a turn about the vertical axis, in radians, for both the light and the sky: puts the sun where the shot wants it */
  rotation?: number;
  /** draw the panorama as the sky behind everything (off by default: light only) */
  background?: boolean;
  /** the sky's brightness, when it should differ from the light's (`intensity` by default) */
  backgroundIntensity?: number;
}

/** Light from all around, out of a panorama: soft light from the sky and reflections of it in every glossy or metal
 * surface (image-based lighting), and with `background` the panorama as the sky. Put it inside the Canvas3D, once a
 * frame; its src, intensity, rotation and background may change from frame to frame (a cut from a hall to a street).
 * Bevy's EnvironmentMapLight with maps already filtered is <EnvironmentMapLight>.
 * Its panorama is filtered by Bevy's generator and, with `background`, drawn as the sky (engine/render
 * environment.rs): @bevy bevy_light::probe::GeneratedEnvironmentMapLight @bevy bevy_light::probe::Skybox */
export function Environment(props: EnvironmentProps) {
  if (typeof props.src !== "string" || props.src === "") throw new TypeError(`Environment: src must be a panorama's path, staticFile("sky.hdr"), got ${JSON.stringify(props.src)}`);
  if (!/\.(hdr|exr|png|jpe?g)$/i.test(props.src.split("?")[0])) throw new TypeError(`Environment: src must be an equirectangular .hdr, .exr, .png or .jpg, got ${JSON.stringify(props.src)}`);
  for (const k of ["intensity", "backgroundIntensity"] as const) {
    const v = props[k];
    if (v !== undefined && !(typeof v === "number" && v >= 0 && Number.isFinite(v))) throw new RangeError(`Environment: ${k} must be 0 or more, got ${v}`);
  }
  if (props.rotation !== undefined && !(typeof props.rotation === "number" && Number.isFinite(props.rotation))) throw new RangeError(`Environment: rotation must be a number of radians, got ${props.rotation}`);
  return node3d("Environment", props as unknown as Record<string, unknown>);
}

export interface EnvironmentMapLightProps {
  /** the prefiltered diffuse (irradiance) cubemap, a .ktx2 file: staticFile("environment_maps/pisa_diffuse_rgb9e5_zstd.ktx2") */
  diffuse_map: string;
  /** the prefiltered specular (radiance) cubemap with its mips, a .ktx2 file */
  specular_map: string;
  /** cd/m² (Bevy's default 1) */
  intensity?: number;
}

/** Bevy's EnvironmentMapLight with its own prefiltered maps, as Bevy's examples put one on the camera (docs/control.feature
 *  FC0a): light from all around and its reflections, from cubemaps already filtered (an <Environment> filters a panorama
 *  itself). Put it inside the Canvas3D, once; the first frame's holds.
 *  @bevy bevy_light::probe::EnvironmentMapLight */
export function EnvironmentMapLight(props: EnvironmentMapLightProps) {
  for (const k of ["diffuse_map", "specular_map"] as const) {
    if (typeof props[k] !== "string" || !/\.ktx2$/i.test(props[k])) throw new TypeError(`EnvironmentMapLight: ${k} is a prefiltered cubemap's .ktx2 file, got ${JSON.stringify(props[k])}`);
  }
  if (props.intensity !== undefined && !(typeof props.intensity === "number" && props.intensity >= 0 && Number.isFinite(props.intensity))) throw new RangeError(`EnvironmentMapLight: intensity is cd/m², 0 or more, got ${props.intensity}`);
  return node3d("Environment", { diffuse_map: assetPath(props.diffuse_map), specular_map: assetPath(props.specular_map), intensity: props.intensity ?? 1 });
}

export interface AmbientOcclusionProps {
  /** how many samples a pixel takes: "low" (4), "medium" (8, the default), "high" (18), "ultra" (54) */
  quality?: "low" | "medium" | "high" | "ultra";
  /** how thick, in metres, a surface is taken to be behind what the camera sees (0.25 by default): larger darkens more
   * behind thin things */
  thickness?: number;
}

/** Screen-space ambient occlusion: creases, corners and the ground under things darken where the sky's and the room's
 * light can't reach. It darkens ambient and environment light (an <AmbientLight>, an <Environment>), not the direct
 * light of a sun or a lamp. Turns multisampling off, so the edges are smoothed by SMAA instead (see <AntiAliasing>).
 * Put it inside the Canvas3D, once; the first frame's holds for the piece.
 * @bevy bevy_pbr::ssao::ScreenSpaceAmbientOcclusion */
export function AmbientOcclusion(props: AmbientOcclusionProps) {
  if (props.quality !== undefined && !["low", "medium", "high", "ultra"].includes(props.quality)) throw new RangeError(`AmbientOcclusion: quality must be "low", "medium", "high" or "ultra", got ${JSON.stringify(props.quality)}`);
  if (props.thickness !== undefined && !(typeof props.thickness === "number" && props.thickness > 0 && Number.isFinite(props.thickness))) throw new RangeError(`AmbientOcclusion: thickness must be a positive number of metres, got ${props.thickness}`);
  return node3d("AmbientOcclusion", props as unknown as Record<string, unknown>);
}

export interface AntiAliasingProps {
  /** "msaa" (4x multisampling, the default), "smaa" (morphological, high preset: the default with AmbientOcclusion,
   * which can't multisample), "fxaa" (cheapest, softest) or "none". Each draws a frame from that frame alone, so a
   * render equals its play (temporal anti-aliasing isn't offered: it needs the frames before) */
  mode: "msaa" | "smaa" | "fxaa" | "none";
}

/** How the 3D's edges are smoothed. Put it inside the Canvas3D, once; the first frame's holds for the piece.
 * @bevy bevy_anti_alias::smaa::Smaa
 * @bevy bevy_anti_alias::fxaa::Fxaa */
export function AntiAliasing(props: AntiAliasingProps) {
  if (!["msaa", "smaa", "fxaa", "none"].includes(props.mode)) throw new RangeError(`AntiAliasing: mode must be "msaa", "smaa", "fxaa" or "none", got ${JSON.stringify(props.mode)}`);
  return node3d("AntiAliasing", props as unknown as Record<string, unknown>);
}

export interface ShadowsProps {
  /** each shadow map's size in texels (4096 by default; 8192 is sharper still, at four times the memory and about
   * 2-3 ms a frame at 1080p) */
  mapSize?: 1024 | 2048 | 4096 | 8192;
  /** how far from the camera the sun's shadows reach, in metres (150 by default): the shorter, the sharper they are */
  distance?: number;
  /** where the sun's first, sharpest cascade ends, in metres (8 by default): the distance of the nearest subject */
  firstCascade?: number;
  /** how many cascades split the distance (4 by default, 1-4) */
  cascades?: number;
  /** how much neighbouring cascades overlap, 0-1 (0.3 by default) so the seam between them doesn't show */
  overlap?: number;
  /** depth bias: raise it if surfaces facing the light show stripes of their own shadow (acne); too high and shadows
   * detach from what casts them (0.02 by default for the sun, as Bevy's) */
  bias?: number;
  /** normal bias, in texels: the same trade-off along the surface's normal (1.8 by default for the sun) */
  normalBias?: number;
  /** "gaussian" (the default: soft, smooth edges) or "hardware" (2x2, crisper and cheaper) */
  filter?: "gaussian" | "hardware";
}

/** Shadow quality for every light in the Canvas3D: map size, the sun's cascades, biases, filtering. Put it inside the
 * Canvas3D, once; the first frame's holds for the piece. mapSize sets both shadow maps' size, filter the camera's
 * filtering, and the cascade props the sun's cascades (engine/render scene3d.rs):
 * @bevy bevy_light::directional_light::DirectionalLightShadowMap
 * @bevy bevy_light::point_light::PointLightShadowMap
 * @bevy bevy_light::ShadowFilteringMethod
 * @bevy bevy_light::cascade::CascadeShadowConfig */
export function Shadows(props: ShadowsProps) {
  if (props.mapSize !== undefined && ![1024, 2048, 4096, 8192].includes(props.mapSize)) throw new RangeError(`Shadows: mapSize must be 1024, 2048, 4096 or 8192, got ${props.mapSize}`);
  for (const k of ["distance", "firstCascade"] as const) {
    const v = props[k];
    if (v !== undefined && !(typeof v === "number" && v > 0 && Number.isFinite(v))) throw new RangeError(`Shadows: ${k} must be a positive number of metres, got ${v}`);
  }
  if (props.cascades !== undefined && !(Number.isInteger(props.cascades) && props.cascades >= 1 && props.cascades <= 4)) throw new RangeError(`Shadows: cascades must be 1, 2, 3 or 4, got ${props.cascades}`);
  if (props.overlap !== undefined && !(props.overlap >= 0 && props.overlap < 1)) throw new RangeError(`Shadows: overlap must be from 0 to below 1, got ${props.overlap}`);
  for (const k of ["bias", "normalBias"] as const) {
    const v = props[k];
    if (v !== undefined && !(typeof v === "number" && v >= 0 && Number.isFinite(v))) throw new RangeError(`Shadows: ${k} must be 0 or more, got ${v}`);
  }
  if (props.filter !== undefined && !["gaussian", "hardware"].includes(props.filter)) throw new RangeError(`Shadows: filter must be "gaussian" or "hardware", got ${JSON.stringify(props.filter)}`);
  return node3d("Shadows", props as unknown as Record<string, unknown>);
}

export interface VolumetricFogProps {
  /** mode="march": samples along each ray through a fog volume (64 by default): too few stair-step a shaft's edges, so 128 for a
   * render with hard-edged shafts, or 32 with `dither`; about 0.1 ms a step at 1920x1080 on an M5 Pro */
  steps?: number;
  /** mode="march": start each pixel's march a fraction of a step in, by a fixed noise seeded from the pixel and the frame number
   * (false by default): the shafts' steps become fine grain instead of bands, so fewer steps look smooth. The noise
   * is a function of the frame, so a render still equals its play */
  dither?: boolean;
  /** mode="march": the fraction of the frame's resolution the fog is marched at (1 by default): 0.5 marches a quarter of the
   * pixels and puts the result over the frame guided by depth, so an edge in front of a shaft keeps no fog from
   * what's behind it. 0.5, 1/3 or 0.25; each frame on its own, so a render still equals its play */
  resolution?: number;
  /** light the fog has everywhere, lit or not (0 by default). Bevy lights a volume's whole box with it whatever its
   * density, so the box's edges show: keep it 0, or small with the camera inside the volume */
  ambient?: number;
  /** that light's colour (white by default) */
  ambientColor?: string;
  /** how the fog is drawn: "froxel" (the default: every volume written into one grid of cells along the view, lit
   * once a cell and integrated front to back once, with the `hero` lights' shafts integrated exactly along epipolar lines: no
   * steps, no dither, its cost the grid's whatever the resolution or the number of volumes, overlapping volumes mixed
   * as one medium, see-through surfaces fogged at their own depth) or "march" (Bevy's ray march through each
   * volume, as before). `steps`, `dither` and `resolution` are the march's; `slices`, `tile`, `samples` and `hero`
   * the grid's */
  mode?: "march" | "froxel";
  /** the grid's cells along the view, spaced evenly in log depth from the camera to the farthest volume (64 by
   * default; 32 to 256) */
  slices?: number;
  /** the grid's cells across the frame, in pixels a side (16 by default; 4, 8 or 16) */
  tile?: number;
  /** points lit in each cell, spread along its depth (1 by default; 1 to 8): more keeps thin shadows in the
   * grid's soft light (a hero light's shafts don't need them) */
  samples?: number;
  /** the grid's shafts: true (the default) integrates the first two volumetric spot lights and the first two
   * volumetric directional lights exactly, over their shadow maps texel by texel, so a blind's bars or a doorway's
   * edge are as crisp as the shadow map: integrated at samples along lines from the light across the frame (Chen et
   * al.'s epipolar sampling), more of them where the depth or the light changes, and each pixel read from the
   * samples either side of it at its depth; false lights them in the grid's cells with the rest */
  hero?: boolean;
}

/** Volumetric fog on the camera: light in the air, drawn inside every <FogVolume>, with shafts from the lights marked
 * `volumetric`. Put it inside the Canvas3D, once; without it fog volumes draw nothing.
 * @bevy bevy_light::volumetric::VolumetricFog
 * @bevy bevy_light::volumetric::VolumetricLight */
export function VolumetricFog(props: VolumetricFogProps) {
  if (props.steps !== undefined && !(Number.isInteger(props.steps) && props.steps >= 1 && props.steps <= 256)) throw new RangeError(`VolumetricFog: steps must be a whole number from 1 to 256, got ${props.steps}`);
  if (props.dither !== undefined && typeof props.dither !== "boolean") throw new TypeError(`VolumetricFog: dither must be true or false, got ${JSON.stringify(props.dither)}`);
  if (props.resolution !== undefined && !(props.resolution > 0 && props.resolution <= 1 && [1, 2, 3, 4].some((k) => Math.abs(props.resolution! * k - 1) < 1e-6))) throw new RangeError(`VolumetricFog: resolution must be 1, 0.5, 1/3 or 0.25, got ${props.resolution}`);
  if (props.ambient !== undefined && !(props.ambient >= 0)) throw new RangeError(`VolumetricFog: ambient must be 0 or more, got ${props.ambient}`);
  if (props.mode !== undefined && !["march", "froxel"].includes(props.mode)) throw new RangeError(`VolumetricFog: mode must be "march" or "froxel", got ${JSON.stringify(props.mode)}`);
  if (props.slices !== undefined && !(Number.isInteger(props.slices) && props.slices >= 32 && props.slices <= 256)) throw new RangeError(`VolumetricFog: slices must be a whole number from 32 to 256, got ${props.slices}`);
  if (props.tile !== undefined && ![4, 8, 16].includes(props.tile)) throw new RangeError(`VolumetricFog: tile must be 4, 8 or 16, got ${props.tile}`);
  if (props.samples !== undefined && !(Number.isInteger(props.samples) && props.samples >= 1 && props.samples <= 8)) throw new RangeError(`VolumetricFog: samples must be a whole number from 1 to 8, got ${props.samples}`);
  return node3d("VolumetricFog", colours("VolumetricFog", props as Record<string, unknown>, ["ambientColor"]));
}

export interface FogVolumeProps {
  /** parts are matched by name across frames: name a volume that comes and goes */
  name?: string;
  /** the box's centre */
  position?: Vec3;
  /** radians */
  rotation?: Vec3;
  /** the box's size in metres: one number for a cube, or [x, y, z] (1 by default) */
  scale?: number | Vec3;
  /** how thick the air is (0.1 by default: dusty; 0.02 a faint haze; 0.5 smoke). Animate it to let dust settle,
   * smoke roll in or a volume vanish (0) */
  density?: number;
  /** the fog's own colour, what light scattering out of it takes on (white by default; a dark red for blood in the
   * air) */
  color?: string;
  /** how much light it swallows (0.3 by default): higher reads as soot or smoke */
  absorption?: number;
  /** how much light it throws back toward the eye (0.3 by default): higher reads as bright dust or mist */
  scattering?: number;
  /** how far forward it scatters, -1 to 1 (0.5 by default): near 1 the shafts glow when you look toward the light */
  anisotropy?: number;
  /** how fast it thins going up, per metre from the box's floor (0 by default: even throughout): 0.5 is a ground
   * fog that's a third as thick two metres up. A box wide and tall with a falloff is height fog. Only
   * <VolumetricFog mode="froxel"> draws it; the march draws the box evenly */
  falloff?: number;
  /** Bevy's density_texture: a 3D .ktx2 image in the media folder whose values scale the density through the box (a
   * cloud, a bunny of smoke); the box draws evenly without one. Only the march draws it */
  density_texture?: string;
}

/** A box of air that light shows in: dust in a sunbeam, smoke, a haze of blood. Needs a <VolumetricFog> in the
 * Canvas3D; lights marked `volumetric` throw shafts through it, and every light's glow fills it softly. A part, so it
 * keeps its name across frames: animate its density, colour and placement.
 * @bevy bevy_light::volumetric::FogVolume */
export function FogVolume(props: FogVolumeProps) {
  for (const k of ["position", "rotation"] as const) if (props[k] !== undefined) checkVec("FogVolume", k, props[k]);
  if (props.density !== undefined && !(props.density >= 0 && Number.isFinite(props.density))) throw new RangeError(`FogVolume: density must be 0 or more, got ${props.density}`);
  if (props.anisotropy !== undefined && !(props.anisotropy > -1 && props.anisotropy < 1)) throw new RangeError(`FogVolume: anisotropy must be between -1 and 1, got ${props.anisotropy}`);
  const tex = props.density_texture;
  if (tex !== undefined && !(typeof tex === "string" && /\.ktx2$/i.test(tex))) throw new TypeError(`FogVolume: density_texture is a 3D .ktx2 image, got ${JSON.stringify(tex)}`);
  if (props.falloff !== undefined && !(props.falloff >= 0 && Number.isFinite(props.falloff))) throw new RangeError(`FogVolume: falloff must be 0 or more, got ${props.falloff}`);
  return node3d("FogVolume", colours("FogVolume", (tex === undefined ? props : { ...props, density_texture: assetPath(tex) }) as Record<string, unknown>, ["color"]));
}
