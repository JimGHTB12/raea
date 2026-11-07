// raea's lights and camera as three.js's, in three's physical units through units.ts (a light puts the same light on
// screen as raea's engine does at its default exposure).
import * as THREE from "three";
import { RectAreaLightUniformsLib } from "three/examples/jsm/lights/RectAreaLightUniformsLib.js";
import { relativeOf, toThree, type LightKind } from "../units";

type Props = Record<string, unknown>;
type V3 = [number, number, number];

export const vec = (v: unknown): V3 | undefined =>
  Array.isArray(v) && v.length === 3 && v.every((n) => typeof n === "number") ? (v as V3) : undefined;

let areaReady = false;

export function makeLight(kind: string): THREE.Light | undefined {
  switch (kind) {
    case "AmbientLight": return new THREE.AmbientLight();
    case "DirectionalLight": return new THREE.DirectionalLight();
    case "PointLight": return new THREE.PointLight();
    case "SpotLight": return new THREE.SpotLight();
    case "RectAreaLight":
      if (!areaReady) {
        RectAreaLightUniformsLib.init();
        areaReady = true;
      }
      return new THREE.RectAreaLight();
    default: return undefined;
  }
}

/** a light's props set: colour, intensity (units.ts), place and aim, reach, cone, shadows */
export function setLight(l: THREE.Light, kind: string, p: Props, scene: THREE.Scene, shadowMap: number) {
  if (typeof p.color === "string") l.color.set(p.color);
  const i = relativeOf(kind, p);
  const pos = vec(p.position);
  if (pos) l.position.set(...pos);
  if (l instanceof THREE.RectAreaLight) {
    l.width = (p.width as number) ?? 1;
    l.height = (p.height as number) ?? 1;
    l.intensity = toThree("RectAreaLight", i, l.width * l.height);
    l.lookAt(...(vec(p.target) ?? [0, 0, 0]));
    return;
  }
  l.intensity = toThree(kind as LightKind, i);
  if (l instanceof THREE.PointLight || l instanceof THREE.SpotLight) {
    l.distance = typeof p.distance === "number" ? p.distance : 20;
    l.decay = 2;
  }
  if (l instanceof THREE.SpotLight) {
    l.angle = typeof p.angle === "number" ? p.angle : Math.PI / 3;
    l.penumbra = typeof p.penumbra === "number" ? p.penumbra : 0;
  }
  if (l instanceof THREE.DirectionalLight || l instanceof THREE.SpotLight || l instanceof THREE.PointLight) {
    // raea's sun casts shadows unless told not to; a bulb and a spot only when told to
    l.castShadow = l instanceof THREE.DirectionalLight ? p.castShadow !== false : p.castShadow === true;
    if (l.castShadow) l.shadow.mapSize.set(shadowMap, shadowMap);
  }
  if (l instanceof THREE.DirectionalLight || l instanceof THREE.SpotLight) {
    l.target.position.set(...(vec(p.target) ?? [0, 0, 0]));
    if (!l.target.parent) scene.add(l.target);
    if (l instanceof THREE.DirectionalLight && l.castShadow) {
      // the shadow's box: around what the light aims at, as Bevy's cascades cover the view
      const c = l.shadow.camera;
      c.left = c.bottom = -15;
      c.right = c.top = 15;
      c.near = 0.1;
      c.far = Math.max(50, l.position.distanceTo(l.target.position) * 2 + 30);
      c.updateProjectionMatrix();
      l.shadow.bias = -0.0005;
      l.shadow.normalBias = 0.02;
    }
  }
}

/** raea's camera (a PerspectiveCamera part; with `ortho`, its height in metres, an orthographic one) */
export function setCamera(had: THREE.Camera | undefined, p: Props): THREE.PerspectiveCamera | THREE.OrthographicCamera {
  const ortho = typeof p.ortho === "number";
  const cam = ortho
    ? had instanceof THREE.OrthographicCamera ? had : new THREE.OrthographicCamera()
    : had instanceof THREE.PerspectiveCamera ? had : new THREE.PerspectiveCamera();
  cam.position.set(...(vec(p.position) ?? [0, 0, 5]));
  cam.up.set(...(vec(p.up) ?? [0, 1, 0]));
  cam.lookAt(...(vec(p.lookAt) ?? [0, 0, 0]));
  cam.near = typeof p.near === "number" ? p.near : 0.1;
  cam.far = typeof p.far === "number" ? p.far : 1000;
  if (cam instanceof THREE.PerspectiveCamera) {
    cam.fov = typeof p.fov === "number" ? p.fov : 50;
    const v = p.view as { width: number; height: number; x: number; y: number } | undefined;
    if (v) cam.userData.view = v;
    else delete cam.userData.view;
  } else cam.userData.height = p.ortho as number;
  if (typeof p.focus === "number") cam.userData.focus = p.focus;
  else delete cam.userData.focus;
  cam.updateMatrixWorld();
  return cam;
}

/** the camera fitted to a drawing of this size */
export function fitCamera(c: THREE.Camera, width: number, height: number) {
  const aspect = width / height;
  if (c instanceof THREE.PerspectiveCamera) {
    c.aspect = aspect;
    const v = c.userData.view as { width: number; height: number; x: number; y: number } | undefined;
    if (v) c.setViewOffset(v.width, v.height, v.x, v.y, width, height);
    else c.clearViewOffset();
    c.updateProjectionMatrix();
  } else if (c instanceof THREE.OrthographicCamera) {
    const h = (c.userData.height as number) ?? 10;
    c.top = h / 2;
    c.bottom = -h / 2;
    c.left = (-h * aspect) / 2;
    c.right = (h * aspect) / 2;
    c.updateProjectionMatrix();
  }
}

/** raea's tone mapping names as three's */
export const TONE: Record<string, THREE.ToneMapping> = {
  // TonyMcMapface (Bevy's default) isn't one of three's: AgX is the nearest (sdk/playable's host draws Tony itself)
  tony: THREE.AgXToneMapping, aces: THREE.ACESFilmicToneMapping, agx: THREE.AgXToneMapping, neutral: THREE.NeutralToneMapping,
  reinhard: THREE.ReinhardToneMapping, none: THREE.NoToneMapping,
};
