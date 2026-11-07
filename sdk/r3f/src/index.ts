// raea/r3f: React Three Fiber both ways (docs/r3f.feature). <R3F> draws an R3F scene with raea (R3F as raea's input);
// <Raea> draws a raea composition's 3D inside an R3F <Canvas>, and Scene3d (the three.js host) draws raea's parts for
// any three.js app (raea as R3F's input). loadShapes brings in raea's solids (minus, hull, extrude, Text3D) for both.
export { R3F, type R3FProps } from "./input/R3F";
export { convert, type Converted } from "./input/scene";
export { Raea, type RaeaProps } from "./output/Raea";
export { Scene3d, type Settings } from "./three/scene";
export { KINDS, PRIMITIVES, type Status } from "./three/kinds";
export { CLASSES, type InputStatus } from "./input/kinds";
export { loadShapes, shapesReady } from "./three/geometry";
export { elementsOf, COMPONENTS, type Part, type Props } from "./parts";
export { toThree, toRaea, relativeOf, EXPOSURE } from "./units";
