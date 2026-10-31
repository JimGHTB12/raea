// Placement in the scene, and drawing a body: <Character>.
import type { ReactNode } from "react";
import { Model, type Vec3 } from "@raea/render/3d";
import { LinearRgba, Srgba, StandardMaterial } from "@raea/render/bevy3d";
import type { Glitch } from "@raea/physics/physics";
import { staticFile } from "@raea/video/media";
import { Goals, Pose, Side, Skeleton, add, eulerXYZ, pose, qmul, qrot, qyaw, rest, scale3, skinJoints, solve, sub } from "./all";

// ---------------------------------------------------------------- placement

export interface Placement {
  position?: Vec3;
  /** the turn about up, radians: at 0 the body faces +z */
  yaw?: number;
  scale?: number;
}

/** A point in the scene in the body's frame, for goals given in the scene's coordinates. */
export function toBody(at: Placement, p: Vec3): Vec3 {
  const d = sub(p, at.position ?? [0, 0, 0]);
  return scale3(qrot(qyaw(-(at.yaw ?? 0)), d), 1 / (at.scale ?? 1));
}

/** A point of the body's frame in the scene. */
export function toScene(at: Placement, p: Vec3): Vec3 {
  return add(at.position ?? [0, 0, 0], qrot(qyaw(at.yaw ?? 0), scale3(p, at.scale ?? 1)));
}

/** Goals in the scene's coordinates brought into the body's frame (directions turned, points moved and turned). */
export function goalsToBody(at: Placement, g: Goals): Goals {
  const pt = (v: Vec3) => toBody(at, v);
  const dir = (v: Vec3 | undefined) => (v ? qrot(qyaw(-(at.yaw ?? 0)), v) : undefined);
  const map = <T,>(o: Partial<Record<Side, T>> | undefined, f: (x: T) => T) => (o ? Object.fromEntries(Object.entries(o).map(([k, v]) => [k, v && f(v as T)])) : undefined);
  return {
    sit: g.sit && { ...g.sit, at: pt(g.sit.at), feet: map(g.sit.feet, pt), floor: g.sit.floor === undefined ? undefined : (g.sit.floor - (at.position?.[1] ?? 0)) / (at.scale ?? 1) },
    feet: map(g.feet, (f) => ({ ...f, at: pt(f.at), toes: dir(f.toes) })),
    hands: map(g.hands, (h) => ({ ...h, at: pt(h.at), point: dir(h.point), facing: dir(h.facing), elbow: dir(h.elbow) })),
    look: g.look && pt(g.look),
    weight: g.weight,
    lookWeight: g.lookWeight,
    lean: g.lean,
  };
}

// ---------------------------------------------------------------- drawing

const bodies = new Map<string, { skeleton: Skeleton; src?: string }>();

/** A body made by character.py, so Characters can name it: `registerBody(kid)` once, then `<Character body="kid">`.
 * `src` is the folder character.py wrote, staticFile("characters/<name>") by default. */
export function registerBody(skeleton: Skeleton, src?: string): void {
  bodies.set(skeleton.name, { skeleton, src });
}

export interface CharacterProps extends Placement {
  /** the body: a registered body's name ("kid") or its skeleton.json */
  body?: string | Skeleton;
  /** the body's skeleton.json (the same as `body` given a skeleton) */
  skeleton?: Skeleton;
  /** the folder character.py wrote: staticFile("characters/<name>") by default */
  src?: string;
  /** the name its parts are matched by (the skeleton's by default; two of one body need two) */
  name?: string;
  /** the pose (the rest pose by default) */
  pose?: Pose;
  /** goals met on top of the pose, in the scene's coordinates */
  goals?: Goals;
  /** its ink outline (the inverted hull; true by default, false for a ghost): looks with their own outlines leave it out */
  outline?: boolean;
  /** the skin's own triangles broken up by the frame's values (physics.tsx's Glitch: explode, scramble, jitter, snap,
   * smear, seed) while it keeps following the pose; the hull, if drawn, glitched the same */
  glitch?: Glitch;
  /** the skin drawn in one flat material instead of its own (an echo trailing the body, a hologram): a Character of
   * the same body under another `name`, posed at an earlier frame, so several can trail one body */
  ghost?: CharacterGhost;
  /** the composition pass its skin (and hull) draws in (Mesh's `pass`; "scene" by default) */
  pass?: string;
  /** the skin's materials, as a <Model>'s: <MeshStandardMaterial> re-covering its glTF materials (all, or the one
   * `material` names), with or without a `shader` over them (docs/shaders.feature SH3); the joints and hull keep theirs */
  children?: ReactNode;
}

/** A ghost's material: one colour over the whole skin (its own picture taken off). */
export interface CharacterGhost {
  /** hex colour ("#33e6ff") */
  color: string;
  /** 0..1 (1 by default: solid); below 1 it is blended over what's behind */
  opacity?: number;
  /** a glow added to the colour (with Bloom it blooms) */
  emissive?: string;
  /** how strong the glow is (1 by default) */
  emissiveIntensity?: number;
  /** no light on it, just the colour (true by default); false keeps the scene's light on its shape */
  unlit?: boolean;
}

/** An Anny body posed. Each joint is a hidden model where the pose puts it, and the skin follows them (Model's
 * `bones`). Groups are flattened before the engine sees them while a skin's joints are placed in the skin's own frame,
 * so a Character is placed by its own position, yaw and scale, never inside a <Group>. */
export function Character(props: CharacterProps) {
  const reg = typeof props.body === "string" ? bodies.get(props.body) : undefined;
  if (typeof props.body === "string" && !reg) throw new Error(`Character: no body ${props.body} (registerBody its skeleton.json first; registered: ${[...bodies.keys()].join(", ") || "none"})`);
  const skel = props.skeleton ?? (typeof props.body === "object" ? props.body : reg?.skeleton);
  if (!skel) throw new Error("Character: give it a body (a registered name or a skeleton.json)");
  const src = props.src ?? reg?.src ?? staticFile(`characters/${skel.name}`);
  const name = props.name ?? skel.name;
  const at: Placement = { position: props.position, yaw: props.yaw, scale: props.scale };
  const k = at.scale ?? 1;
  let p = props.pose ?? rest(skel);
  if (props.goals) p = solve(skel, p, goalsToBody(at, props.goals));
  const w = pose(skel, p);
  const turn = qyaw(at.yaw ?? 0);
  // the skin stands at the origin scaled by k, so its joints go where they are in the scene over k
  const sj = skinJoints(skel, w);
  const bones = Object.fromEntries(sj.map((j) => [j.name, `${name}-${j.name}`]));
  const g = props.glitch, ghost = props.ghost;
  const pass = props.pass !== undefined ? { pass: props.pass } : {};
  const glitch = g ? { explode: g.explode ?? 0, scramble: g.scramble ?? 0, jitter: g.jitter ?? 0, snap: g.snap ?? 0, smear: g.smear ?? [0, 0, 0], seed: g.seed ?? 0 } : undefined;
  return (
    <>
      {sj.map((j) => (
        <Model key={j.name} name={`${name}-${j.name}`} src={`${src}/joint.glb`} position={scale3(toScene(at, j.pos), 1 / k)} rotation={eulerXYZ(qmul(turn, j.rot))} scale={0.001} />
      ))}
      <Model name={`${name}-body`} src={`${src}/body.glb`} bones={bones} scale={k} allowOverlap={ghost ? true : undefined} flat={ghost ? ghost.unlit !== false : undefined} {...(glitch ? { glitch } : {})} {...pass}>
        {props.children}
        {ghost && <StandardMaterial base_color_texture={null} base_color={ghost.opacity !== undefined ? Srgba.hex(ghost.color).with_alpha(ghost.opacity) : ghost.color} {...(ghost.opacity !== undefined && ghost.opacity < 1 ? { alpha_mode: "Blend" as const } : {})} {...(ghost.emissive ? { emissive: LinearRgba.from(Srgba.hex(ghost.emissive)).mul(ghost.emissiveIntensity ?? 1) } : {})} />}
      </Model>
      {(props.outline ?? !ghost) && <Model name={`${name}-hull`} src={`${src}/body.hull.glb`} bones={bones} scale={k} flat {...(glitch ? { glitch } : {})} {...pass} />}
    </>
  );
}
