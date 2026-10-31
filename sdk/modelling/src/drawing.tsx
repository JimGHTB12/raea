// Drawing a build: <Build>.
import { Fragment, type ReactNode } from "react";
import { useVideoConfig } from "@raea/app/clock";
import { Model, ToneMapping } from "@raea/render/3d";
import { manifold, Solid } from "@raea/render/solid";
import { writeGlb } from "@raea/asset/glb";
import { AXIS, AXIS_ROT, Bone, BuildLook, Built, M3, S, V3, XfGroup, add3, alignY, backOut, camAt, cross3, dotsOf, easeOut, eulerXYZ, flatBackdrop, focusOf, gizmo, hullOf, jointWorlds, len3, lerp3, meshOf, mm3, mul3, norm3, rigOf, smooth, sortedBones, span, sub3, viewportOf, weightMapOf, wireGlb, wireOf, xfPoint } from "./all";
import { AmbientLight, DirectionalLight } from "raea/light";
import { Camera3d } from "raea/camera";
import { Quat, Transform } from "raea/transform";
import { Entity } from "raea/3d";
import { LinearRgba, Srgba } from "raea/color";
import { StandardMaterial } from "raea/pbr";
import { Mesh3d } from "raea/mesh";

// ---------------------------------------------------------------- drawing

export interface BuildProps {
  build: Built;
  /** beats into the build */
  at: number;
  /** the camera's vertical field of view (degrees) */
  fov?: number;
  /** draw the build's own camera (true) */
  camera?: boolean;
  /** the wireframe over the shading (true) */
  wire?: boolean;
  look?: BuildLook;
}

/** A solid in the viewport: grey studio shading, its wireframe, and the orange outline when it's active. */
function Part({ id, solid, active, wire, clay, dots, ghost }: { id: string; solid: Solid; active?: boolean; wire: boolean; clay: string; dots?: boolean; ghost?: { color: string; opacity: number } }) {
  if (ghost) {
    return (
      <>
        <Mesh3d name={`${id}-ghost-${solid.id}`} shape={solid} allowOverlap={true}>
          <StandardMaterial base_color={Srgba.hex(ghost.color).with_alpha(ghost.opacity)} perceptual_roughness={0.5} emissive={LinearRgba.from(Srgba.hex(ghost.color)).mul(0.35)} alpha_mode="Blend" />
        </Mesh3d>
        {ghost.opacity > 0.15 ? <Model name={`${id}-ghostwire-${solid.id}`} src={wireOf(solid, ghost.color)} /> : null}
      </>
    );
  }
  const d = dots ? dotsOf(solid) : null;
  return (
    <>
      <Mesh3d name={`${id}-${solid.id}`} shape={solid} allowOverlap={true}>
        <StandardMaterial base_color={clay} perceptual_roughness={0.62} metallic={0} />
      </Mesh3d>
      {wire ? <Model name={`${id}-wire-${solid.id}`} src={wireOf(solid)} /> : null}
      {active ? <Model name={`${id}-sel-${solid.id}`} src={hullOf(solid)} /> : null}
      {d ? <Model name={`${id}-dots-${solid.id}`} src={d} /> : null}
    </>
  );
}

/** Draws children `s` of the way from the eye toward them: the same picture, but nearer than everything (Blender's
 * "in front"), for bones and gizmos. */
function InFront({ eye, s, children }: { eye: V3; s: number; children: ReactNode }) {
  return (
    <Entity transform={Transform.fromTranslation(mul3(eye, 1 - s)).withScale(s)}>
      {children}
    </Entity>
  );
}

function Gizmo({ kind, at, size, axis, u }: { kind: "move" | "rotate" | "scale"; at: V3; size: number; axis: number; u: number }) {
  const g = gizmo();
  const part = kind === "move" ? g.arrow : kind === "scale" ? g.handle : g.ring;
  const pop = backOut(span(u, 0, 0.18)) * (1 - smooth(span(u, 0.88, 1)));
  if (pop <= 0.001) return null;
  return (
    <Entity transform={Transform.fromTranslation(at).withScale(Math.max(0.001, size * pop))}>
      {[0, 1, 2].map((k) => (
        <Entity key={k} transform={Transform.IDENTITY.withRotation(Quat.fromEuler("XYZ", ...(AXIS_ROT[k])))}>
          <Mesh3d name={`gizmo-${kind}-${k}`} shape={part}>
            <StandardMaterial base_color={Srgba.hex(AXIS[k]).with_alpha(k === axis ? 1 : 0.45)} unlit alpha_mode={k !== axis ? "Blend" : "Opaque"} />
          </Mesh3d>
        </Entity>
      ))}
      <Mesh3d name="gizmo-ball" shape={g.ball}>
        <StandardMaterial base_color="#ffffff" unlit />
      </Mesh3d>
    </Entity>
  );
}

function Cursor({ at, cam, size }: { at: V3; cam: { yaw: number; pitch: number }; size: number }) {
  const g = gizmo();
  return (
    <Entity transform={Transform.fromTranslation(at).withRotation(Quat.fromEuler("XYZ", 0, cam.yaw, 0))}>
      <Entity transform={Transform.IDENTITY.withRotation(Quat.fromEuler("XYZ", -cam.pitch, 0, 0)).withScale(size)}>
        <Mesh3d name="cursor-red" shape={g.cursorRed}><StandardMaterial base_color="#e83a3a" unlit /></Mesh3d>
        <Mesh3d name="cursor-white" shape={g.cursorWhite}><StandardMaterial base_color="#f2f2f2" unlit /></Mesh3d>
        <Mesh3d name="cursor-ticks" shape={g.cursorTicks}><StandardMaterial base_color="#111111" unlit /></Mesh3d>
      </Entity>
    </Entity>
  );
}

/** A bone drawn from its joint's world matrix: the octahedron, its wire, and the head and tail balls. */
function BoneShape({ id, b, J, grow, selected }: { id: string; b: Bone; J: { r: M3; t: V3 }; grow: number; selected: boolean }) {
  const g = gizmo();
  if (!S.boneWire) S.boneWire = writeGlb({ primitives: [wireGlb(g.bone, "#0c0c0c")] });
  if (!S.boneWireSel) S.boneWireSel = writeGlb({ primitives: [wireGlb(g.bone, "#ffb13b")] });
  const dir = sub3(b.tail, b.head);
  const L = len3(dir) * Math.max(0.001, grow);
  const R = mm3(J.r, alignY(dir));
  const rot = eulerXYZ(R);
  return (
    <Entity transform={Transform.fromTranslation(J.t).withRotation(Quat.fromEuler("XYZ", rot[0], rot[1], rot[2])).withScale(L)}>
      <Mesh3d name={`${id}-body`} shape={g.bone}>
        <StandardMaterial base_color={selected ? "#6fc2ff" : "#9c9c9c"} perceptual_roughness={0.7} />
      </Mesh3d>
      <Model name={`${id}-wire${selected ? "-sel" : ""}`} src={selected ? S.boneWireSel : S.boneWire} />
      <Mesh3d name={`${id}-head`} shape={g.boneEnd}><StandardMaterial base_color={selected ? "#ffb13b" : "#2a2a2a"} unlit /></Mesh3d>
      <Entity transform={Transform.fromXyz(0, 1, 0)}>
        <Mesh3d name={`${id}-tail`} shape={g.boneEnd}><StandardMaterial base_color={selected ? "#ffb13b" : "#2a2a2a"} unlit /></Mesh3d>
      </Entity>
    </Entity>
  );
}

/** The build at `at` beats, in the modelling viewport (inside a Canvas3D). */
export function Build({ build: b, at, fov = 34, camera = true, wire = true, look = {} }: BuildProps) {
  const { width, height } = useVideoConfig();
  const L: Required<BuildLook> = { clay: look.clay ?? "#a3a3a3", background: look.background ?? "#8fd3f4", line: look.line ?? "#2b1a10" };
  const cam = camAt(b, at, width / height, fov);
  const vp = viewportOf();
  const step = b.steps.find((s) => at >= s.start && at < s.start + s.beats);
  const u = step ? (at - step.start) / step.beats : 1;
  const finished = b.steps.some((s) => s.kind === "finish" && at >= s.start);
  const rigStart = b.steps.find((s) => s.kind === "pose" || s.kind === "animate" || s.kind === "finish");
  const posing = !!rigStart && at >= rigStart.start;
  const state = b.stateAt(at);
  const shown = new Map(state);
  const parts: ReactNode[] = [];
  const overlay: ReactNode[] = [];
  const inFront: ReactNode[] = [];
  const gsize = cam.dist * 0.16;
  let cursorAt: V3 = [0, 0, 0];
  for (const s of b.steps) if (s.kind === "add" && s.start <= at) cursorAt = s.at;

  // the active step's own drawing
  if (step && !finished && "object" in step) {
    shown.delete(step.object);
    if (step.kind === "array" && step.template) shown.delete(step.template);
    const id = `obj-${step.object}`;
    switch (step.kind) {
      case "add": {
        cursorAt = lerp3(step.from, step.at, smooth(span(u, 0, 0.3)));
        const k = backOut(span(u, 0.3, 0.7));
        if (k > 0.001) {
          const prev = state.get(step.object) ? null : null;
          void prev;
          parts.push(
            <Entity key={id} transform={Transform.fromTranslation(step.at).withScale(Math.max(0.001, k))}>
              <Entity transform={Transform.fromTranslation(mul3(step.at, -1))}>
                <Part id={id} solid={step.after} active wire={wire} clay={L.clay} />
              </Entity>
            </Entity>,
          );
        }
        break;
      }
      case "transform": {
        const k = smooth(span(u, 0.18, 0.8));
        const pivot = step.xf.pivot ?? meshOf(step.before).center;
        const xf = step.xf;
        const amounts = xf.rotate ?? xf.move ?? (typeof xf.scale === "number" ? [xf.scale - 1, xf.scale - 1, xf.scale - 1] : xf.scale ? [xf.scale[0] - 1, xf.scale[1] - 1, xf.scale[2] - 1] : [0, 0, 0]);
        const axis = [0, 1, 2].sort((a, c) => Math.abs(amounts[c]) - Math.abs(amounts[a]))[0];
        if (u < 0.8) parts.push(<XfGroup key={id} xf={xf} t={k}><Part id={id} solid={step.before} active wire={wire} clay={L.clay} dots /></XfGroup>);
        else parts.push(<Part key={id} id={id} solid={step.after} active wire={wire} clay={L.clay} dots />);
        const gat = xfPoint(pivot, { ...xf, move: xf.move ? mul3(xf.move, k) : undefined, rotate: undefined, scale: undefined });
        overlay.push(<InFront key="gizmo" eye={cam.eye} s={0.5}><Gizmo kind={step.gizmo} at={gat} size={gsize} axis={axis} u={u} /></InFront>);
        break;
      }
      case "boolean": {
        const color = step.fn === "minus" ? "#ff4a4a" : step.fn === "union" ? "#52e05a" : "#4aa8ff";
        const done = u >= 0.62;
        parts.push(<Part key={id} id={id} solid={done ? step.after : step.before} active wire={wire} clay={L.clay} />);
        const mb = meshOf(step.before), mt = meshOf(step.tool);
        let dir = sub3(mt.center, mb.center);
        if (len3(dir) < 1e-3) dir = [0, 1, 0];
        const away = mul3(norm3(dir), mb.radius * 1.4 + mt.radius);
        const sweep = 1 - easeOut(span(u, 0.04, 0.55));
        const fade = done ? 1 - smooth(span(u, 0.62, 0.9)) : 1;
        if (fade > 0.01) {
          parts.push(
            <Entity key="ghost" transform={Transform.fromTranslation(mul3(away, sweep))}>
              <Part id={`tool-${step.index}`} solid={step.tool} wire clay={color} ghost={{ color, opacity: 0.42 * fade * smooth(span(u, 0, 0.12)) + 0.001 }} />
            </Entity>,
          );
        }
        break;
      }
      case "mirror": {
        const k = easeOut(span(u, 0.22, 0.7));
        if (u < 0.72) {
          parts.push(<Part key={id} id={id} solid={step.before} active wire={wire} clay={L.clay} />);
          const sc: V3 = [1, 1, 1];
          sc[step.axis] = Math.max(0.001, k);
          if (k > 0.001) parts.push(<Entity key="copy" transform={Transform.IDENTITY.withScale(sc)}><Part id={`${id}-copy`} solid={step.copy} wire={wire} clay={L.clay} /></Entity>);
        } else parts.push(<Part key={id} id={id} solid={step.after} active wire={wire} clay={L.clay} />);
        // the plane: a thin translucent sheet through the origin, across the axis, sized to the part
        const m = meshOf(step.after);
        const size = m.radius * 2.6;
        const dims: V3 = [size, size, size];
        dims[step.axis] = 0.004;
        const op = 0.28 * smooth(span(u, 0, 0.15)) * (1 - smooth(span(u, 0.8, 1)));
        const c: V3 = [...m.center];
        c[step.axis] = 0;
        if (op > 0.005)
          overlay.push(
            <Mesh3d key="plane" transform={Transform.fromTranslation(c)} name="mirror-plane" allowOverlap={true} shape={manifold.box(dims[0], dims[1], dims[2])}>
              <StandardMaterial base_color={Srgba.hex("#7fc4ff").with_alpha(op)} unlit alpha_mode="Blend" />
            </Mesh3d>,
          );
        break;
      }
      case "extrude": {
        if (step.before) parts.push(<Part key={`${id}-b`} id={`${id}-b`} solid={step.before} wire={wire} clay={L.clay} />);
        const k = easeOut(span(u, 0.3, 0.8));
        if (u < 0.82) {
          parts.push(
            <XfGroup key={id} xf={step.place}>
              <Entity transform={Transform.IDENTITY.withScale([1, 1, Math.max(0.001, k)])}>
                <Part id={`${id}-x`} solid={step.local} active wire={wire} clay={L.clay} />
              </Entity>
            </XfGroup>,
          );
        } else parts.push(<Part key={id} id={id} solid={step.after} active wire={wire} clay={L.clay} />);
        break;
      }
      case "revolve": {
        if (step.before) parts.push(<Part key={`${id}-b`} id={`${id}-b`} solid={step.before} wire={wire} clay={L.clay} />);
        const i = Math.min(step.stages.length - 1, Math.floor(span(u, 0.1, 0.75) * step.stages.length));
        parts.push(<Part key={id} id={`${id}-r`} solid={u >= 0.8 ? step.after : step.stages[i]} active wire={wire} clay={L.clay} />);
        break;
      }
      case "sculpt": {
        const i = Math.min(step.stages.length - 1, Math.floor(span(u, 0.08, 0.7) * step.stages.length));
        const ghostOp = step.before ? 0.3 * (1 - smooth(span(u, 0.1, 0.7))) : 0;
        parts.push(<Part key={id} id={id} solid={step.stages[i]} active wire={wire} clay={L.clay} />);
        if (step.before && ghostOp > 0.01) parts.push(<Part key="was" id={`${id}-was`} solid={step.before} wire clay={L.clay} ghost={{ color: "#c8c8c8", opacity: ghostOp }} />);
        break;
      }
      case "smooth": {
        const done = u >= 0.5;
        const pulse = 1 + 0.035 * Math.sin(Math.PI * span(u, 0.42, 0.66));
        const m = meshOf(step.before);
        parts.push(
          <Entity key={id} transform={Transform.fromTranslation(m.center).withScale(pulse)}>
            <Entity transform={Transform.fromTranslation(mul3(m.center, -1))}>
              <Part id={id} solid={done ? step.after : step.before} active wire={wire} clay={L.clay} dots={!done} />
            </Entity>
          </Entity>,
        );
        break;
      }
      case "array": {
        const n = step.copies.length;
        const shownN = Math.floor(span(u, 0.05, 0.85) * n + 1e-6);
        if (u >= 0.9) parts.push(<Part key={id} id={id} solid={step.after} active wire={wire} clay={L.clay} />);
        else {
          step.copies.forEach((c, k) => {
            const local = span(u, 0.05 + (0.8 * k) / n, 0.05 + (0.8 * (k + 1.6)) / n);
            if (k > shownN || local <= 0) return;
            const s = backOut(local);
            const m = meshOf(c);
            parts.push(
              <Entity key={`copy-${k}`} transform={Transform.fromTranslation(m.center).withScale(Math.max(0.001, s))}>
                <Entity transform={Transform.fromTranslation(mul3(m.center, -1))}>
                  <Part id={`${id}-c${k}`} solid={c} active={k === shownN - 1} wire={wire} clay={L.clay} />
                </Entity>
              </Entity>,
            );
          });
        }
        break;
      }
    }
  }

  // the rest of the objects as they stand
  if (!posing) {
    const weighting = step?.kind === "weights";
    if (weighting && step.kind === "weights") {
      const k = Math.min(step.bones.length - 1, Math.floor(u * step.bones.length));
      parts.push(<Model key="weights" name={`weights-${step.bones[k]}`} src={weightMapOf(b, step.bones[k])} />);
    } else {
      for (const [name, s] of shown) parts.push(<Part key={`obj-${name}`} id={`obj-${name}`} solid={s} wire={wire} clay={L.clay} />);
    }
  }

  // the rig: posed and skinned once posing starts (the viewport's clay, or the finished look)
  let worlds: { r: M3; t: V3 }[] | null = null;
  const sorted = b.bones.length ? sortedBones(b) : [];
  if (posing && b.bones.length) {
    const rig = rigOf(b, L);
    const time = Math.max(0, Math.min(rig.to - rig.from, at - rig.from));
    parts.push(<Model key="rig" name={finished ? "rig-final" : "rig-viewport"} src={finished ? rig.final : rig.viewport} animation="rig" time={time} loop={false} />);
  }
  if (b.bones.length && !finished) {
    const pose = posing ? b.poseAt(at) : {};
    worlds = jointWorlds(sorted, pose);
    const bonesShown = new Set<string>();
    let growing: string[] = [];
    for (const s of b.steps) {
      if (s.kind !== "bone" || s.start > at) continue;
      if (at >= s.start + s.beats) s.bones.forEach((x) => bonesShown.add(x));
      else growing = s.bones;
    }
    const selected = step?.kind === "weights" ? step.bones[Math.min(step.bones.length - 1, Math.floor(u * step.bones.length))] : step?.kind === "bone" ? growing[Math.min(growing.length - 1, Math.floor(u * growing.length))] : null;
    const items: ReactNode[] = [];
    sorted.forEach((bone, i) => {
      let grow = bonesShown.has(bone.name) ? 1 : 0;
      if (step?.kind === "bone" && growing.includes(bone.name)) {
        const j = growing.indexOf(bone.name), n = growing.length;
        grow = easeOut(span(u, (0.85 * j) / n, (0.85 * (j + 1.5)) / n));
      }
      if (grow <= 0.001) return;
      // a weight map shows the selected bone only
      if (step?.kind === "weights" && bone.name !== selected) return;
      items.push(<BoneShape key={bone.name} id={`bone-${bone.name}`} b={bone} J={worlds![i]} grow={grow} selected={bone.name === selected} />);
    });
    // nearer than the model's front: the camera's distance less the model's radius
    const f = focusOf(b, undefined);
    const nearest = Math.max(0.3, len3(sub3(cam.eye, f.c)) - f.r * 1.1);
    inFront.push(<InFront key="bones" eye={cam.eye} s={Math.min(1, nearest / len3(sub3(cam.eye, f.c)))}>{items}</InFront>);
  }

  const key = cam;
  const keyDir = norm3(add3(sub3(key.eye, key.target), [0, 0, 0]));
  const right = norm3(cross3([0, 1, 0], keyDir));
  const up = cross3(keyDir, right);
  const keyPos = add3(key.target, mul3(add3(add3(mul3(keyDir, 1), mul3(right, -0.8)), mul3(up, 1.1)), 10));
  const fillPos = add3(key.target, mul3(add3(add3(mul3(keyDir, 0.6), mul3(right, 1)), mul3(up, -0.3)), 10));
  const rimPos = add3(key.target, mul3(add3(mul3(keyDir, -1), mul3(up, 0.6)), 10));

  return (
    <>
      {camera ? <Camera3d transform={Transform.fromTranslation(cam.eye).lookingAt(cam.target)} projection={{ Perspective: { fov: fov * Math.PI / 180, near: 0.05, far: 300 } }} /> : null}
      <ToneMapping mode="none" />
      <AmbientLight color={finished ? "#fff4e6" : "#ffffff"} brightness={(finished ? 0.55 : 0.42) * 400} />
      <DirectionalLight color={finished ? "#fff1dc" : "#ffffff"} transform={Transform.fromTranslation(keyPos).lookingAt(key.target)} illuminance={(finished ? 1.35 : 1.1) * 8000} />
      <DirectionalLight color={finished ? "#d9ecff" : "#e9eef5"} transform={Transform.fromTranslation(fillPos).lookingAt(key.target)} illuminance={(finished ? 0.35 : 0.4) * 8000} />
      <DirectionalLight transform={Transform.fromTranslation(rimPos).lookingAt(key.target)} illuminance={(finished ? 0.6 : 0.35) * 8000} />
      <Model name={finished ? "backdrop-final" : "backdrop"} src={finished ? flatBackdrop(L.background) : vp.bg} />
      {!finished ? <Model name="grid" src={vp.grid} /> : null}
      {parts.map((p, i) => <Fragment key={i}>{p}</Fragment>)}
      {overlay.map((p, i) => <Fragment key={`o${i}`}>{p}</Fragment>)}
      {!finished && !posing ? <Cursor at={cursorAt} cam={cam} size={cam.dist * 0.022} /> : null}
      {inFront.length ? <InFront eye={cam.eye} s={1}>{inFront.map((p, i) => <Fragment key={`f${i}`}>{p}</Fragment>)}</InFront> : null}
    </>
  );
}
