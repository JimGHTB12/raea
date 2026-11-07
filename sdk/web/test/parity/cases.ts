// raea-web parity: the kinds raea's own engine draws that three.js can't (sdk/r3f/src/three/kinds.ts "engine"), each
// drawn by `raea render` and by raea-web in headless Chrome on WebGPU, compared by SSIM (tools/parity.sh). The first five
// are sdk/raea/bench/bevy-ts's cases (64x64); entities and fog are beside this file (320x180).
const BENCH = "../../../raea/bench/bevy-ts/cases";
export interface Case {
  name: string;
  /** the kinds it measures */
  kinds: string[];
  /** the edit, relative to this directory; its composition "main" */
  edit: string;
  frame: number;
}
export const CASES: Case[] = [
  { name: "surface-shader", kinds: ["SurfaceShader"], edit: `${BENCH}/shader-scene.tsx`, frame: 0 },
  { name: "frame-pass", kinds: ["FramePass"], edit: `${BENCH}/frame-pass.tsx`, frame: 0 },
  { name: "compute", kinds: ["Compute"], edit: `${BENCH}/compute.tsx`, frame: 1 },
  { name: "component", kinds: ["BevyComponent"], edit: `${BENCH}/reflect-on-camera.tsx`, frame: 0 },
  { name: "resource", kinds: ["BevyResource"], edit: `${BENCH}/reflect-keyed-resource-still.tsx`, frame: 15 },
  { name: "entities", kinds: ["Entities"], edit: "./entities.tsx", frame: 10 },
  { name: "fog", kinds: ["VolumetricFog", "FogVolume"], edit: "./fog.tsx", frame: 0 },
];
