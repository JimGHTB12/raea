// Lights.
import { type ReactNode } from "react";
import { type Vec3 } from "../3d";
import { AmbientLight as AmbientLight3, DirectionalLight as DirectionalLight3, PointLight as PointLight3, RectAreaLight, SpotLight as SpotLight3 } from "../three";
import { OLD_LIGHT_PROPS, Quat, TransformValue } from "./all";

function refuseOld(owner: string, props: Record<string, unknown>, allowed: readonly string[] = []) {
  for (const k of OLD_LIGHT_PROPS) {
    if (props[k] !== undefined && !allowed.includes(k)) throw new TypeError(`${owner}: \`${k}\` is 3d.tsx's ${owner}'s; Bevy's takes a transform and its own fields (docs/sdk.feature S2a)`);
  }
}

function forward(t: TransformValue | undefined): { position: Vec3; target: Vec3; rotation: Quat } {
  const position: Vec3 = t?.translation ?? [0, 0, 0];
  const rotation = t?.rotation ?? Quat.IDENTITY;
  const f = Quat.mulVec3(rotation, [0, 0, -1]);
  // the rotation too, which the engine places the light by exactly (the target is for what reads where it shines)
  return { position, target: [position[0] + f[0], position[1] + f[1], position[2] + f[2]], rotation };
}

/** The props every Bevy light shares beside its own fields. */
interface LightCommon {
  /** matched by name across frames (a light that comes and goes, or one of several) */
  name?: string;
  transform?: TransformValue;
  /** Bevy's Visibility: "Hidden" lights nothing */
  visibility?: "Visible" | "Hidden" | "Inherited";
  color?: string;
  /** Bevy's shadow_depth_bias and shadow_normal_bias on this light, over a <Shadows>' (Bevy's defaults otherwise: 0.02 and
   * 1.8 for the sun and spots, 0.08 and 0.6 for a point light) */
  shadow_depth_bias?: number;
  shadow_normal_bias?: number;
  /** its light shows in the air through a <FogVolume> (Bevy's VolumetricLight component, beside the light) */
  volumetric?: boolean;
  children?: ReactNode;
}

export interface PointLightProps extends LightCommon {
  /** lumens (Bevy's default 1,000,000; a 60 W bulb is about 800) */
  intensity?: number;
  /** metres it reaches (20) */
  range?: number;
  /** the bulb's radius in metres (0): the size of its highlight on shiny surfaces (a spherical area light), and soft
   * shadows' size when soft_shadows_enabled */
  radius?: number;
  shadow_maps_enabled?: boolean;
  soft_shadows_enabled?: boolean;
  /** raea's paint look draws this light (3d.tsx's PointLight `paint`) */
  paint?: boolean;
}

/** Shadows' size as 3d.tsx takes it: soft with the radius (or the default bulb) when soft_shadows_enabled. */
const soft = (p: { radius?: number; soft_shadows_enabled?: boolean }) => (p.soft_shadows_enabled ? (p.radius !== undefined && p.radius > 0 ? p.radius : true) : undefined);

/** Bevy's PointLight: a bulb at its transform's translation, in lumens.
 * @bevy bevy_light::point_light::PointLight */
export function PointLight(p: PointLightProps) {
  refuseOld("PointLight", p as Record<string, unknown>, ["intensity"]);
  const { transform, visibility, intensity = 1_000_000, range = 20, radius, shadow_maps_enabled = false, soft_shadows_enabled, ...rest } = p;
  if (visibility === "Hidden") return null;
  const s = soft({ radius, soft_shadows_enabled });
  return <PointLight3 {...rest} position={forward(transform).position} lumens={intensity} distance={range} castShadow={shadow_maps_enabled} {...(radius ? { radius } : {})} {...(s !== undefined ? { softShadow: s } : {})} />;
}

export interface SpotLightProps extends PointLightProps {
  /** the cone's half-angle in radians (Bevy's default PI/4), up to PI/2 */
  outer_angle?: number;
  /** where the light starts to fall off, in radians from the axis (0) */
  inner_angle?: number;
}

/** Bevy's SpotLight: a cone along its transform's -Z, in lumens.
 * @bevy bevy_light::spot_light::SpotLight */
export function SpotLight(p: SpotLightProps) {
  refuseOld("SpotLight", p as Record<string, unknown>, ["intensity"]);
  const { transform, visibility, intensity = 1_000_000, range = 20, radius, shadow_maps_enabled = false, soft_shadows_enabled, outer_angle = Math.PI / 4, inner_angle = 0, ...rest } = p;
  if (visibility === "Hidden") return null;
  const s = soft({ radius, soft_shadows_enabled });
  // the rows take a penumbra: inner = outer * (1 - penumbra) (scene3d.rs)
  const penumbra = 1 - Math.min(inner_angle, outer_angle) / outer_angle;
  return <SpotLight3 {...rest} {...forward(transform)} lumens={intensity} distance={range} castShadow={shadow_maps_enabled} {...(radius ? { radius } : {})} angle={outer_angle} penumbra={penumbra} {...(s !== undefined ? { softShadow: s } : {})} />;
}

export interface DirectionalLightProps extends Omit<LightCommon, "children"> {
  /** lux on a surface facing it (Bevy's default 10,000, light_consts::lux::AMBIENT_DAYLIGHT; full sun is about 100,000) */
  illuminance?: number;
  shadow_maps_enabled?: boolean;
  /** soft shadows (PCSS): the sun's size as its shadows see it, in metres */
  soft_shadow_size?: number | null;
  children?: ReactNode;
}

/** Bevy's DirectionalLight: parallel light shining along its transform's -Z, in lux.
 * @bevy bevy_light::directional_light::DirectionalLight */
export function DirectionalLight(p: DirectionalLightProps) {
  refuseOld("DirectionalLight", p as Record<string, unknown>);
  const { transform, visibility, illuminance = 10_000, shadow_maps_enabled = false, soft_shadow_size, ...rest } = p;
  if (visibility === "Hidden") return null;
  // 3d.tsx's sun shines from position to target; a direction needs only a point ahead of where it is
  return <DirectionalLight3 {...rest} {...forward(transform)} lux={illuminance} castShadow={shadow_maps_enabled} {...(soft_shadow_size ? { softShadow: soft_shadow_size } : {})} />;
}

export interface AmbientLightProps {
  name?: string;
  color?: string;
  /** cd/m² (Bevy's default 80) */
  brightness?: number;
}

/** Bevy's AmbientLight (on the camera in 0.20; GlobalAmbientLight is the same light as a resource): light from
 *  everywhere, in cd/m².
 * @bevy bevy_light::ambient_light::AmbientLight */
export function AmbientLight(p: AmbientLightProps) {
  return <AmbientLight3 {...ambientOf(p)} />;
}

/** An AmbientLight's props as 3d.tsx's light takes them: its brightness as `nits`, which the rows take as written. */
export function ambientOf(p: AmbientLightProps): { name?: string; color?: string; nits: number } {
  refuseOld("AmbientLight", p as Record<string, unknown>);
  const { brightness = 80, ...rest } = p;
  return { ...rest, nits: brightness };
}

/** The same light as a resource, as Bevy 0.20 names it. (A function, not a const alias: an alias would keep 3D code in
 *  a 2D bundle through the prelude.) */
export function GlobalAmbientLight(p: AmbientLightProps) {
  return AmbientLight(p);
}

export interface RectLightProps extends Omit<LightCommon, "children" | "volumetric" | "name"> {
  /** lumens (1,000,000) */
  intensity?: number;
  range?: number;
  /** its face's size in metres (1 x 1) */
  width?: number;
  height?: number;
}

/** Bevy's RectLight: a rectangle of light facing its transform's -Z, in lumens (no shadows in Bevy 0.20).
 * @bevy bevy_light::rect_light::RectLight */
export function RectLight(p: RectLightProps) {
  refuseOld("RectLight", p as Record<string, unknown>, ["intensity"]);
  const { transform, visibility, intensity = 1_000_000, range = 20, width = 1, height = 1, ...rest } = p;
  if (visibility === "Hidden") return null;
  return <RectAreaLight {...rest} {...forward(transform)} lumens={intensity} distance={range} width={width} height={height} />;
}
