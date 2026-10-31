// Physics in raea/3d: rigid bodies, glitches and goo, every one baked or closed-form so frame N is always the
// same picture (a render, `render --at N` and a play agree).
//
//   <Physics seconds={6} colliders={[{ min: [-0.47, 0, -0.73], max: [0.47, 0.75, 0.73] }]}>
//     <Body name="egg" src={TABLE} nodes={["egg_001"]} release={1.5} impulses={[{ t: 1.5, impulse: [0.02, 0, 0] }]} />
//     <Body name="ball" shape={{ sphere: 0.05 }} centre={[0, 1, 0]} restitution={0.7}>
//       <Mesh><Sphere args={[0.05]} /><MeshStandardMaterial color="#d33" /></Mesh>
//     </Body>
//   </Physics>
//
// - Rigid bodies (<Physics>, <Body>): Rapier in the engine (systems/bake), stepped at fps x substeps
//   from rest, baked once per request (the bodies, their files, the events) into the composition's asset store and
//   played back by the frame. A body is a glTF's nodes (`src` + `nodes`: cut out with physics.part, its collider their
//   convex hull, its mass their volume x density) or a shape you give (`shape`, `centre`) drawn by its children.
//   Events: `release` (held still until then), `impulses` at times, a kinematic `path` (a function of seconds).
// - Glitches (`glitch` on <Part>, <Body>, or <Model glitch> through <Part>): the part's own triangles exploded,
//   scrambled, jittered, snapped or smeared by values of the frame (the engine's meshseq.rs); `material` variants
//   (the missing-texture checker, a flat glowing echo); `teleport`, `echoes`, `snapPose` and `zFight` helpers below.
// - Goo (<Goo>): objects melted by MLS-MPM (tools/goo, Taichi on Metal): elastic, then plastic, then a viscous fluid,
//   each from its own time, baked to a mesh sequence and drawn as <MeshSequence> with a glossy, slightly see-through
//   surface. `useGoo` says when each object's goo has taken over, so its own part can go. `solver: "sph"` is for thin
//   liquids instead (SPlisHSPlasH: coffee, water, honey), poured from cups and emitters over boxes and glTF obstacles.
export { hash01, lastBeat, teleport, snapPose, quatMul, quatAxis, about, part, Part, Echoes, ZFight, physicsRequest, useBake, bodyPose, Physics, Body, usePhysics, MeshSequence, gooSpec, useGoo, Goo } from "./physics/all";
export type { PartInfo, PartMaterial, Glitch, PartProps, Shape, Impulse, BodyProps, PhysicsProps, Baked, MeshSequenceProps, GooObject, GooCup, GooProps, GooBake } from "./physics/all";


