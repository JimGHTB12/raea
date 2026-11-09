// Rigid bodies baked by Rapier.
import { Children, createContext, Fragment, isValidElement, useContext, type ReactElement, type ReactNode } from "react";
import { useCurrentFrame, useVideoConfig } from "@raea/app/clock";
import { hasNative, native } from "@raea/app/native";
import { type Vec3 } from "raea/3d";
import { Glitch, Part, PartMaterial, Quat, node } from "./all";

// ---- rigid bodies ---------------------------------------------------------------------------------------------

export type Shape = "hull" | "box" | "mesh" | { box: Vec3 } | { sphere: number } | { capsule: [number, number] };

export interface Impulse {
  /** seconds from the simulation's start */
  t: number;
  /** N·s, world */
  impulse?: Vec3;
  /** where it's applied (world, metres): off the centre it spins the body too */
  at?: Vec3;
  /** an angular impulse, N·m·s, world */
  torque?: Vec3;
}

export interface BodyProps {
  /** the body's name, unique in its <Physics>: the bake keys its track by it */
  name: string;
  /** a glTF's nodes as the body (drawn as a <Part>, its collider their hull) */
  src?: string;
  /** the glTF nodes that make the body */
  nodes?: string[];
  /** "hull" (default for nodes), "box" (their bounds), "mesh" (exact triangles, fixed bodies only), or a shape */
  shape?: Shape;
  /** where a given shape's centre stands at rest, world (a nodes' body: the middle of their bounds) */
  centre?: Vec3;
  /** it never moves: a floor, a wall, a table */
  fixed?: boolean;
  /** kg (else volume x density) */
  mass?: number;
  /** kg/m³ (1000) */
  density?: number;
  /** 0 slides, 1 grips (0.5) */
  friction?: number;
  /** 0 lands dead, 1 bounces back fully (0.2) */
  restitution?: number;
  /** m/s at the start, world */
  velocity?: Vec3;
  /** radians a second at the start, world */
  angularVelocity?: Vec3;
  /** how fast its motion bleeds off (0) */
  linearDamping?: number;
  /** how fast its spin bleeds off (0.05) */
  angularDamping?: number;
  /** seconds: held where it is until then (a cup sits until it's knocked) */
  release?: number;
  /** kicks at given seconds: an impulse, where it's applied, and a torque */
  impulses?: Impulse[];
  /** a kinematic mover: its rigid motion from rest at t seconds (see `about`); it pushes the others */
  path?: (t: number) => { position: Vec3; quaternion: Quat };
  /** drawn as the body's nodes are, or by its children (inside its motion: draw them where it stands at rest) */
  material?: PartMaterial;
  /** how its part breaks up, or a function of seconds giving that */
  glitch?: Glitch | ((t: number) => Glitch | undefined);
  /** it casts shadows */
  castShadow?: boolean;
  /** what draws the body, in place of its nodes */
  children?: ReactNode;
}

export interface PhysicsProps {
  /** how long to simulate, seconds */
  seconds: number;
  /** m/s² (0, -9.81, 0) */
  gravity?: Vec3;
  /** steps per frame (8) */
  substeps?: number;
  /** static boxes (world, metres): the floor, the table top */
  colliders?: { min: Vec3; max: Vec3; friction?: number; restitution?: number }[];
  /** the frame (of the enclosing Sequence) the simulation starts on (0); before it everything is at rest */
  from?: number;
  /** the <Body> elements to simulate, and anything else to draw */
  children?: ReactNode;
}

export interface Baked {
  /** the frame rate it was baked at */
  fps: number;
  /** how many frames it holds */
  frames: number;
  /** the bake's file in the asset store */
  file: string;
  /** each body's track by name: positions and quaternions a frame, flat, and its rest centre */
  bodies: Record<string, { p: number[]; q: number[]; centre: Vec3 }>;
}

const bakes = new Map<string, Baked>();

const Ctx = createContext<{ baked: Baked; k: number; t: number } | null>(null);

function bodiesIn(children: ReactNode, out: ReactElement<BodyProps>[] = []): ReactElement<BodyProps>[] {
  Children.forEach(children, (c) => {
    if (!isValidElement(c)) return;
    if (c.type === Body) out.push(c as ReactElement<BodyProps>);
    else if (c.type === Fragment) bodiesIn((c.props as { children?: ReactNode }).children, out);
  });
  return out;
}

/** The simulation's request: every body's props as Rapier takes them, events in frames. */
export function physicsRequest(props: PhysicsProps, fps: number): Record<string, unknown> {
  const frames = Math.max(1, Math.round(props.seconds * fps));
  const bodies = bodiesIn(props.children).map(({ props: b }) => {
    const out: Record<string, unknown> = { name: b.name };
    for (const k of ["src", "nodes", "shape", "centre", "fixed", "mass", "density", "friction", "restitution", "velocity", "angularVelocity", "linearDamping", "angularDamping"] as const) {
      if (b[k] !== undefined) out[k] = b[k];
    }
    if (b.release !== undefined) out.release = Math.round(b.release * fps);
    if (b.impulses) out.impulses = b.impulses.map((e) => ({ frame: Math.round(e.t * fps), impulse: e.impulse ?? [0, 0, 0], ...(e.at ? { at: e.at } : {}), ...(e.torque ? { torque: e.torque } : {}) }));
    if (b.path) {
      const p: number[] = [];
      const q: number[] = [];
      for (let f = 0; f <= frames; f++) {
        const m = b.path(f / fps);
        p.push(...m.position);
        q.push(...m.quaternion);
      }
      out.path = { p, q };
    }
    return out;
  });
  const names = new Set<string>();
  for (const b of bodies) {
    if (names.has(b.name as string)) throw new TypeError(`Physics: two bodies named ${JSON.stringify(b.name)}`);
    names.add(b.name as string);
  }
  return { fps, frames, substeps: props.substeps ?? 8, gravity: props.gravity ?? [0, -9.81, 0], colliders: props.colliders ?? [], bodies };
}

/** The bake for these props (made once and kept). */
export function useBake(props: PhysicsProps): Baked {
  const { fps } = useVideoConfig();
  const req = physicsRequest(props, fps);
  const key = JSON.stringify(req);
  let b = bakes.get(key);
  if (!b) {
    if (!hasNative()) throw new Error("Physics: baking needs the raea engine (raea render)");
    b = native("physics.bake", req as never) as Baked;
    bakes.set(key, b);
  }
  return b;
}

/** A body's rigid motion from rest at the current frame. */
export function bodyPose(baked: Baked, name: string, k: number): { position: Vec3; quaternion: Quat } {
  const b = baked.bodies[name];
  if (!b) throw new Error(`Physics: no body named ${JSON.stringify(name)}`);
  const i = Math.min(Math.max(k, 0), baked.frames - 1);
  return { position: [b.p[i * 3], b.p[i * 3 + 1], b.p[i * 3 + 2]], quaternion: [b.q[i * 4], b.q[i * 4 + 1], b.q[i * 4 + 2], b.q[i * 4 + 3]] };
}

/** Rigid bodies, simulated once and played back by the frame: the <Body>s among its children (directly, or in
 *  fragments). Before `from` and after `seconds` they hold their first and last poses. */
export function Physics(props: PhysicsProps) {
  const baked = useBake(props);
  const frame = useCurrentFrame();
  const k = frame - (props.from ?? 0);
  return <Ctx.Provider value={{ baked, k, t: k / baked.fps }}>{props.children}</Ctx.Provider>;
}

/** A rigid body inside <Physics>: drawn where the simulation has it this frame. */
export function Body(p: BodyProps) {
  const ctx = useContext(Ctx);
  if (!ctx) throw new Error(`Body ${p.name}: inside a <Physics>`);
  const m = bodyPose(ctx.baked, p.name, ctx.k);
  const glitch = typeof p.glitch === "function" ? p.glitch(ctx.t) : p.glitch;
  if (p.src && p.nodes && !p.children) {
    return <Part name={p.name} src={p.src} nodes={p.nodes} position={m.position} quaternion={m.quaternion} material={p.material} glitch={glitch} castShadow={p.castShadow} />;
  }
  return node("Group", { position: m.position, quaternion: m.quaternion }, p.children);
}

/** Inside <Physics>: the bake and the frame in it. */
export function usePhysics(): { baked: Baked; frame: number; t: number } {
  const ctx = useContext(Ctx);
  if (!ctx) throw new Error("usePhysics: inside a <Physics>");
  return { baked: ctx.baked, frame: ctx.k, t: ctx.t };
}
