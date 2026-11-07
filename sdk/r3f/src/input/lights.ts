// three.js's lights, cameras, fog and environment as raea's parts, in world space (raea places a light and the camera
// by position and target, not inside a group). Intensities go through units.ts: a three light and the raea light it
// becomes put the same light on screen.
import * as THREE from "three";
import { toRaea, stopsOf } from "../units";
import type { Part, Props } from "./material";
import { srcOf } from "./texture";

const hex = (c: THREE.Color) => "#" + c.getHexString();
const at = (o: THREE.Object3D) => o.getWorldPosition(new THREE.Vector3()).toArray() as [number, number, number];
/** a point one unit along the object's -z: where a camera, a light and an area light face (three's lookAt turns their
 * -z to the target; a camera's getWorldDirection already gives -z, a light's +z, so the axis is read directly) */
const ahead = (o: THREE.Object3D) => {
  const p = o.getWorldPosition(new THREE.Vector3());
  return p.add(new THREE.Vector3(0, 0, -1).applyQuaternion(o.getWorldQuaternion(new THREE.Quaternion()))).toArray() as [number, number, number];
};

const warned = new Set<string>();
const warnOnce = (k: string, why: string) => {
  if (warned.has(k)) return;
  warned.add(k);
  console.warn(`raea/r3f: ${k} ${why}`);
};

/** how far a three point light reaches (its `distance`, or where inverse-square falloff leaves under half a level of
 * 8-bit colour: raea's lights end at a range, three's with distance 0 never do) */
export const reachOf = (intensity: number, distance: number) => (distance > 0 ? distance : Math.max(20, Math.sqrt(intensity * 512)));

/** a light as raea's lights (a hemisphere light is two) */
export function lightsOf(l: THREE.Light, shadows: boolean, name: string): Part[] {
  const p = lightOf(l, shadows, name);
  return p === undefined ? [] : Array.isArray(p) ? p : [p];
}

/** a light as raea's, or undefined for one raea has no light for */
function lightOf(l: THREE.Light, shadows: boolean, name: string): Part | Part[] | undefined {
  const color = hex(l.color);
  const cast = shadows && l.castShadow;
  if ((l as THREE.DirectionalLight).isDirectionalLight) {
    const d = l as THREE.DirectionalLight;
    return { type: "DirectionalLight", props: { name, position: at(d), target: at(d.target), color, intensity: toRaea("DirectionalLight", d.intensity), castShadow: cast } };
  }
  if ((l as THREE.SpotLight).isSpotLight) {
    const s = l as THREE.SpotLight;
    if (s.decay !== 2) warnOnce("SpotLight decay", `${s.decay} falls off as the inverse square in raea (Bevy's lights are physical)`);
    return { type: "SpotLight", props: { name, position: at(s), target: at(s.target), color, intensity: toRaea("SpotLight", s.intensity), distance: reachOf(s.intensity, s.distance), angle: Math.min(Math.PI / 2, s.angle), penumbra: s.penumbra, castShadow: cast } };
  }
  if ((l as THREE.PointLight).isPointLight) {
    const p = l as THREE.PointLight;
    if (p.decay !== 2) warnOnce("PointLight decay", `${p.decay} falls off as the inverse square in raea (Bevy's lights are physical)`);
    return { type: "PointLight", props: { name, position: at(p), color, intensity: toRaea("PointLight", p.intensity), distance: reachOf(p.intensity, p.distance), castShadow: cast } };
  }
  if ((l as THREE.RectAreaLight).isRectAreaLight) {
    const r = l as THREE.RectAreaLight;
    const s = r.getWorldScale(new THREE.Vector3());
    const width = r.width * Math.abs(s.x), height = r.height * Math.abs(s.y);
    return { type: "RectAreaLight", props: { position: at(r), target: ahead(r), width, height, color, intensity: toRaea("RectAreaLight", r.intensity, width * height) } };
  }
  if ((l as THREE.HemisphereLight).isHemisphereLight) {
    // three lights a surface with mix(ground, sky, (1 + n.up) / 2): their average from everywhere, plus half their
    // difference as a light from straight up (or down, when the ground is the brighter). Exact for surfaces facing that
    // light or sideways; one facing away gets the average where three gives it less.
    const h = l as THREE.HemisphereLight;
    warnOnce("HemisphereLight", "is lit as an ambient light of its colours' average and a light from above of half their difference (exact but for surfaces facing away from it)");
    const avg = h.color.clone().lerp(h.groundColor, 0.5);
    const lum = (c: THREE.Color) => 0.2126 * c.r + 0.7152 * c.g + 0.0722 * c.b;
    const up = lum(h.color) >= lum(h.groundColor);
    const [hi, lo] = up ? [h.color, h.groundColor] : [h.groundColor, h.color];
    const d = new THREE.Color(Math.max(0, hi.r - lo.r) / 2, Math.max(0, hi.g - lo.g) / 2, Math.max(0, hi.b - lo.b) / 2);
    const m = Math.max(d.r, d.g, d.b);
    const amb: Part = { type: "AmbientLight", props: { name, color: hex(avg), intensity: toRaea("AmbientLight", h.intensity) } };
    if (m < 1e-4) return amb;
    // three's hemisphere axis is the light's own up (its position, normalised, as three reads it)
    const axis = h.getWorldPosition(new THREE.Vector3()).normalize();
    if (axis.lengthSq() === 0) axis.set(0, 1, 0);
    if (!up) axis.negate();
    const dir: Part = { type: "DirectionalLight", props: { name: `${name}~sky`, position: axis.toArray(), target: [0, 0, 0], color: hex(new THREE.Color(d.r / m, d.g / m, d.b / m)), intensity: toRaea("DirectionalLight", h.intensity * m) } };
    return [amb, dir];
  }
  if ((l as THREE.LightProbe).isLightProbe) {
    // its spherical harmonics' constant band: the light it gives from every way on average
    const p = l as THREE.LightProbe;
    const c = p.sh.coefficients[0].clone().multiplyScalar(0.282095 * p.intensity);
    const m = Math.max(c.x, c.y, c.z, 1e-6);
    warnOnce("LightProbe", "is lit as an ambient light of its average colour");
    return { type: "AmbientLight", props: { name, color: hex(new THREE.Color(c.x / m, c.y / m, c.z / m)), intensity: toRaea("AmbientLight", m) } };
  }
  if ((l as THREE.AmbientLight).isAmbientLight) return { type: "AmbientLight", props: { name, color, intensity: toRaea("AmbientLight", l.intensity) } };
  return undefined;
}

/** the camera R3F draws with, as raea's */
export function cameraOf(c: THREE.Camera): Part | undefined {
  c.updateMatrixWorld();
  const position = at(c);
  const lookAt = ahead(c);
  const up = new THREE.Vector3(0, 1, 0).applyQuaternion(c.getWorldQuaternion(new THREE.Quaternion())).toArray() as [number, number, number];
  if ((c as THREE.PerspectiveCamera).isPerspectiveCamera) {
    const p = c as THREE.PerspectiveCamera;
    // zoom narrows the field of view as three's projection does
    const fov = (2 * Math.atan(Math.tan((p.fov * Math.PI) / 360) / p.zoom) * 180) / Math.PI;
    const props: Props = { position, lookAt, up, fov, near: p.near, far: p.far };
    if (p.view?.enabled) props.view = { width: p.view.fullWidth, height: p.view.fullHeight, x: p.view.offsetX, y: p.view.offsetY };
    return { type: "PerspectiveCamera", props };
  }
  if ((c as THREE.OrthographicCamera).isOrthographicCamera) {
    const o = c as THREE.OrthographicCamera;
    return { type: "OrthographicCamera", props: { position, lookAt, up, height: (o.top - o.bottom) / o.zoom, near: o.near, far: o.far } };
  }
  return undefined;
}

/** three's fog as raea's */
export function fogOf(f: THREE.Fog | THREE.FogExp2 | null): Part | undefined {
  if (!f) return undefined;
  if ((f as THREE.FogExp2).isFogExp2) return { type: "Fog", props: { color: hex(f.color), density: (f as THREE.FogExp2).density } };
  const l = f as THREE.Fog;
  return { type: "Fog", props: { color: hex(l.color), near: l.near, far: l.far } };
}

const PANORAMA = /\.(hdr|exr|png|jpe?g)(\?.*)?$/i;

/** the scene's environment (and a panorama behind it) as raea's <Environment>, or the background colour */
export function skyOf(scene: THREE.Scene): { parts: Part[]; background?: string } {
  const parts: Part[] = [];
  let background: string | undefined;
  const bg = scene.background;
  if (bg instanceof THREE.Color) background = hex(bg);
  const env = srcOf(scene.environment);
  const sky = bg instanceof THREE.Texture ? srcOf(bg) : undefined;
  if (scene.environment && !(env && PANORAMA.test(env))) warnOnce("scene.environment", "isn't a panorama file raea can load (.hdr, .exr, .png, .jpg): left out");
  if (bg instanceof THREE.Texture && !(sky && PANORAMA.test(sky))) warnOnce("scene.background", "texture isn't a panorama file raea can load: left out");
  const rotation = scene.environmentRotation?.y ?? 0;
  if (env && PANORAMA.test(env)) {
    const props: Props = { src: env, intensity: scene.environmentIntensity, rotation };
    if (sky === env) Object.assign(props, { background: true, backgroundIntensity: scene.backgroundIntensity });
    parts.push({ type: "Environment", props });
  } else if (sky && PANORAMA.test(sky)) {
    // a panorama behind the scene that doesn't light it
    parts.push({ type: "Environment", props: { src: sky, intensity: 0, rotation: scene.backgroundRotation?.y ?? 0, background: true, backgroundIntensity: scene.backgroundIntensity } });
  }
  return { parts, background };
}

/** three's renderer settings (R3F sets them from <Canvas flat linear>) as raea's tone mapping */
export function toneOf(gl: THREE.WebGLRenderer): Part {
  const modes: Record<number, string> = { [THREE.ACESFilmicToneMapping]: "aces", [THREE.AgXToneMapping]: "agx", [THREE.NeutralToneMapping]: "neutral", [THREE.ReinhardToneMapping]: "reinhard", [THREE.CineonToneMapping]: "aces", [THREE.LinearToneMapping]: "none", [THREE.NoToneMapping]: "none" };
  const props: Props = { mode: modes[gl.toneMapping] ?? "none" };
  // three's ACES scales its input by 1/0.6 where Bevy's ACESFitted doesn't: the same picture is 0.74 stops up in raea
  const stops = stopsOf(gl.toneMappingExposure / (gl.toneMapping === THREE.ACESFilmicToneMapping ? 0.6 : 1));
  if (stops !== 0) props.exposure = stops;
  return { type: "ToneMapping", props };
}
