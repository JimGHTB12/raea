// Elements the page host draws that React's JSX doesn't type, typed as raea's own so `<view3d>` and its parts check
// in a space's TSX (tools/sdk-coverage counts them as typed). Each prop is what runtime/js view3d.rs reads
// (clear_color, projection, eye, spawn_part, mesh_of); the Blitz page's host builds its views with the same functions
// (engine/page view3d.rs, run from host.rs). A composition's render doesn't draw them: its 3D is raea/3d's
// (a Camera3d and its parts). Vectors are [x, y, z]; angles are degrees; colours are CSS colours.
import type { HTMLAttributes } from "react";

type Vec3 = [number, number, number];

/** A box a camera of its own renders its <mesh3d> and <light3d> children into (laid out and styled as a div is). */
export interface View3dAttributes extends HTMLAttributes<HTMLElement> {
  /** what shows behind the scene (transparent by default) */
  clearColor?: string;
  /** perspective by default */
  projection?: "perspective" | "orthographic";
  /** perspective: the vertical field of view in degrees (45 by default) */
  fov?: number;
  /** orthographic: how many units tall the view is (2 by default) */
  viewHeight?: number;
  /** where the camera is ([0, 0, 10] by default) */
  eye?: Vec3;
  /** what it looks at (the origin by default) */
  target?: Vec3;
}

interface Part3dAttributes {
  key?: string | number | null;
  name?: string;
  /** [0, 0, 0] by default */
  position?: Vec3;
  /** Euler angles in degrees, XYZ */
  rotation?: Vec3;
  /** [1, 1, 1] by default */
  scale?: Vec3;
}

/** A shape in its <view3d>, with a standard material. */
export interface Mesh3dAttributes extends Part3dAttributes {
  /** a cuboid by default */
  shape?: "cuboid" | "sphere" | "cylinder" | "capsule" | "plane" | "torus";
  /** by shape: a cuboid's [x, y, z]; a sphere's [radius]; a cylinder's or capsule's [radius, height]; a plane's
   *  [x, y, z], of which x and z; a torus's [inner radius, outer radius]. [1, 1, 1] by default */
  size?: Vec3 | [number] | [number, number];
  /** white by default; see-through below alpha 1 */
  color?: string;
  /** its colour as it is, with no lighting */
  unlit?: boolean;
  /** 0 by default */
  metallic?: number;
  /** 0.5 by default */
  roughness?: number;
}

/** A light in its <view3d>. */
export interface Light3dAttributes extends Part3dAttributes {
  /** directional by default; ambient lights the whole view */
  kind?: "directional" | "point" | "ambient";
  /** white by default */
  color?: string;
  /** directional: lux (3000 by default) */
  illuminance?: number;
  /** point: lumens (1,000,000 by default); ambient: brightness (500 by default) */
  intensity?: number;
  /** a point it faces (a directional light shines along it) */
  target?: Vec3;
}

declare module "react" {
  namespace JSX {
    interface IntrinsicElements {
      view3d: View3dAttributes;
      mesh3d: Mesh3dAttributes;
      light3d: Light3dAttributes;
    }
  }
}
