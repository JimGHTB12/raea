// Placement in the scene, and drawing a body: <Character>.
import type { ReactNode } from "react";
import { Model, type Vec3 } from "@raea/render/3d";
import { HAIR_LOOK, HAIR_WESL, type HairLook } from "./hair";
import { SKIN_LOOK, SKIN_SURFACE, SKIN_WESL, type SkinLook, type SkinSurface } from "./skin";
import { ExtendedMaterial, LinearRgba, ShaderExtension, Srgba, StandardMaterial } from "@raea/render/bevy3d";
import type { Glitch } from "@raea/physics/physics";
import { staticFile } from "@raea/video/media";
import { Goals, Pose, Side, Skeleton, add, eulerXYZ, faceWeights, pose, qmul, qrot, qyaw, rest, scale3, skinJoints, solve, sub, type Face } from "./all";

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
  /** its ink outline (the inverted hull): off unless asked for (true), as a look that wants an ink line does */
  outline?: boolean;
  /** the skin's own triangles broken up by the frame's values (physics.tsx's Glitch: explode, scramble, jitter, snap,
   * smear, seed) while it keeps following the pose; the hull, if drawn, glitched the same */
  glitch?: Glitch;
  /** the skin drawn in one flat material instead of its own (an echo trailing the body, a hologram): a Character of
   * the same body under another `name`, posed at an earlier frame, so several can trail one body */
  ghost?: CharacterGhost;
  /** the composition pass its skin (and hull) draws in (Mesh's `pass`; "scene" by default) */
  pass?: string;
  /** the face (a psx body's morph targets): any of MakeHuman's 34 expression units by name (FACE_UNITS:
   * "mouth-open", "eyebrows-left-up" ...) and presets mixed from them (FACE_PRESETS: blink, smile, frown, surprise,
   * sad, and the lips' open, pucker and wide), 0 to 1; they add */
  face?: Face;
  /** the hair drawn as hair (character/hair.ts: strand highlights on the body's "hair" material, after lighting): on by
   * default, false for the plain surface, or settings over HAIR_LOOK; `coverage: true` softens the cards' cut edges
   * (alphaMode "coverage", for a scene with <AntiAliasing mode="msaa">: without MSAA the cards draw solid); `color`
   * tints the hair's texture (a hex colour: a creator's hair colour) */
  hair?: boolean | (Partial<HairLook> & { coverage?: boolean; color?: string });
  /** the skin drawn as skin (character/skin.ts, on the body's "skin" material): the fine relief of real skin (fading out with distance), the photo's own surface detail,
   * a second, tighter shine (clearcoat) and light scattered under the skin (a soft, reddened falloff at shadow edges).
   * Off by default; true, or settings over SKIN_LOOK and SKIN_SURFACE. `mask` is the fine relief's strength by region (the
   * body's skin_mask.png, which projects/psx bodies have; false for even relief everywhere); `detailMap` the photo's surface
   * detail (the body's skin_detail.png, psx bodies since 11-05; false for none); `microMap` the fine relief tile (the
   * body's skin_micro.png, from a photograph of real skin; false for none) */
  skin?: boolean | (Partial<SkinLook> & Partial<SkinSurface> & { mask?: string | false; detailMap?: string | false; microMap?: string | false });
  /** more of the body's morph targets by name, beside the face's: a character kit's face-*, weight-*, age-* and tuck-*
   * (corpo's people/kit.py). Name the same set every frame: the engine fixes a model's morph names when it makes it */
  morphs?: Record<string, number>;
  /** more skinned models on the same joints, drawn with the body: a kit's garments and hair (parts/*.glb), each with
   * its own morph weights and materials. A part named "hair" in its src is shaded as hair, as the body's own is */
  parts?: CharacterPart[];
  /** the skin's materials, as a <Model>'s: <MeshStandardMaterial> re-covering its glTF materials (all, or the one
   * `material` names), with or without a `shader` over them (docs/shaders.feature SH3); the joints and hull keep theirs */
  children?: ReactNode;
}

/** A model worn on the body's joints (a garment, hair): its glTF's joints named as the body's. */
export interface CharacterPart {
  /** its .glb */
  src: string;
  /** its morph targets' weights by name (a kit part carries the body's weight-*, age-* and, for hair, face-*) */
  morphs?: Record<string, number>;
  /** its cloth: a baked simulation of its vertices in its bind pose and the frame drawn (<Model cloth>, docs/cloth.md) */
  cloth?: { src: string; frame: number };
  /** <MeshStandardMaterial> re-covering its materials, as the body's children do */
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
/** the body's "hair" material shaded as hair (character/hair.ts) */
function hairMaterial(h: CharacterProps["hair"]) {
  const { coverage, color, ...look } = typeof h === "object" ? h : ({} as { coverage?: boolean; color?: string });
  const base = <StandardMaterial material="hair" perceptual_roughness={1} reflectance={0.04} {...(color ? { base_color: color } : {})} {...(coverage ? { alpha_mode: "AlphaToCoverage" as const } : {})} />;
  return <ExtendedMaterial base={base} extension={<ShaderExtension wesl={HAIR_WESL} uniforms={{ ...HAIR_LOOK, ...look } as Record<string, number | number[]>} />} />;
}

/** the body's "skin" material shaded as skin (character/skin.ts) */
function skinMaterial(sk: CharacterProps["skin"], src: string) {
  const { mask, detailMap, microMap, clearcoat, clearcoatRoughness, reflectance, ...look } = typeof sk === "object" ? sk : ({} as Exclude<CharacterProps["skin"], boolean | undefined>);
  const dir = src.replace(/^\//, "");  // (an asset path: no leading slash)
  const m = mask === false ? undefined : (mask ?? `${dir}/skin_mask.png`);
  const dm = detailMap === false ? undefined : (detailMap ?? `${dir}/skin_detail.png`);
  const mm = microMap === false ? undefined : (microMap ?? `${dir}/skin_micro.png`);
  const textures = { ...(m ? { mask: m } : {}), ...(dm ? { detail: dm } : {}), ...(mm ? { micro: mm } : {}) };
  const defs = { ...(m ? { SKIN_MASK: true } : {}), ...(dm ? { SKIN_DETAIL: true } : {}), ...(mm ? { SKIN_MICRO: true } : {}) };
  const base = (
    <StandardMaterial
      material="skin"
      clearcoat={clearcoat ?? SKIN_SURFACE.clearcoat}
      clearcoat_perceptual_roughness={clearcoatRoughness ?? SKIN_SURFACE.clearcoatRoughness}
      reflectance={reflectance ?? SKIN_SURFACE.reflectance}
      diffuse_transmission={0}
    />
  );
  return <ExtendedMaterial base={base} extension={<ShaderExtension wesl={SKIN_WESL} uniforms={{ ...SKIN_LOOK, ...look } as Record<string, number | number[]>} {...(m || dm || mm ? { textures, defs } : {})} />} />;
}

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
  // (all 34 units always named, so the model's morph names never change: the engine fixes them when it makes the model)
  const named = { ...(props.face ? faceWeights(props.face) : {}), ...(props.morphs ?? {}) };
  const morphs = Object.keys(named).length ? { morphs: named } : {};
  const glitch = g ? { explode: g.explode ?? 0, scramble: g.scramble ?? 0, jitter: g.jitter ?? 0, snap: g.snap ?? 0, smear: g.smear ?? [0, 0, 0], seed: g.seed ?? 0 } : undefined;
  return (
    <>
      {sj.map((j) => (
        <Model key={j.name} name={`${name}-${j.name}`} src={`${src}/joint.glb`} position={scale3(toScene(at, j.pos), 1 / k)} rotation={eulerXYZ(qmul(turn, j.rot))} scale={0.001} />
      ))}
      <Model name={`${name}-body`} src={`${src}/body.glb`} bones={bones} scale={k} allowOverlap={ghost ? true : undefined} flat={ghost ? ghost.unlit !== false : undefined} {...(glitch ? { glitch } : {})} {...pass} {...morphs}>
        {props.children}
        {!ghost && props.hair !== false && hairMaterial(props.hair)}
        {!ghost && props.skin && skinMaterial(props.skin, src)}
        {ghost && <StandardMaterial base_color_texture={null} base_color={ghost.opacity !== undefined ? Srgba.hex(ghost.color).with_alpha(ghost.opacity) : ghost.color} {...(ghost.opacity !== undefined && ghost.opacity < 1 ? { alpha_mode: "Blend" as const } : {})} {...(ghost.emissive ? { emissive: LinearRgba.from(Srgba.hex(ghost.emissive)).mul(ghost.emissiveIntensity ?? 1) } : {})} />}
      </Model>
      {(props.parts ?? []).map((part) => {
        const id = part.src.split("/").pop()!.replace(/\.glb$/, "");
        return (
          <Model key={id} name={`${name}-${id}`} src={part.src} bones={bones} scale={k} {...(glitch ? { glitch } : {})} {...pass} {...(part.morphs && Object.keys(part.morphs).length ? { morphs: part.morphs } : {})} {...(part.cloth ? { cloth: part.cloth } : {})}>
            {part.children}
            {id.includes("hair") && props.hair !== false && hairMaterial(props.hair)}
          </Model>
        );
      })}
      {props.outline === true && !ghost && <Model name={`${name}-hull`} src={`${src}/body.hull.glb`} bones={bones} scale={k} flat {...(glitch ? { glitch } : {})} {...pass} />}
    </>
  );
}
