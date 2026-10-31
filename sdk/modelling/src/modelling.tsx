// Modelling shown as it happens: a model written as a sequence of Manifold steps (add, transform, booleans, mirror,
// extrude, revolve, sculpt, smooth, array) and rig steps (bones, automatic weights, poses, a dance), played back by
// the beat in a faux 3D-modelling viewport: grey studio shading with a wireframe over it, an infinite grid with its
// axis lines, the 3D cursor, an orange outline on the active part, gizmos, red and green boolean ghosts, octahedral
// bones drawn in front, a weight heatmap, and a camera that eases to frame each step like a person turning the view.
// No panels, menus or text: only the environment.
//
//   const sun = modelling((m) => {
//     m.add("body", manifold.sphere(1), { at: [0, 1.6, 0] });       // pops in at the 3D cursor
//     m.scale("body", [1.05, 0.95, 0.9]);                           // the scale gizmo
//     m.minus("body", manifold.sphere(0.2).move(0.3, 1.8, 0.9));    // a red ghost sweeps in, then the cut
//     m.mirror("eye", "x");                                          // the mirror plane, the copy unfolding from it
//     m.bone("spine", [0, 0.8, 0], [0, 2.4, 0]);
//     m.weights({ bones: ["spine"] });                               // the heatmap, bone by bone
//     m.pose((u) => ({ spine: [0, 0, 20 * u] }));
//     m.finish();                                                    // the viewport gives way to the finished look
//     m.animate((beat) => ({ spine: [0, 0, 10 * Math.sin(beat * Math.PI)] }), { beats: 16 });
//   });
//   <Canvas3D><Build build={sun} at={beats} /></Canvas3D>
//
// The build runs once (each step's solids made then, every overlay written once as a .glb named by its bytes); a
// frame only picks what to show from `at`, so frame N is always the same picture.
export { modelling, autoWeights, weightColor, Build } from "./all";
export type { V3, Xf, View, Pose, Step, Bone, BuildLook, Built, Modeller, BuildProps } from "./all";


