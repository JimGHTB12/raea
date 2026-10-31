// Moved to @raea/render/3d (docs/sdk.feature S1d): this path re-exports it until M5's codemod.
export * from "@raea/render/3d";
// /3d has always given physics's components too; @raea/render/3d leaves them to @raea/physics
export { Physics, Body, Part, Echoes, ZFight, MeshSequence, Goo, useGoo, gooSpec, useBake, usePhysics, bodyPose, physicsRequest, part, teleport, snapPose, lastBeat, hash01, about, quatAxis, quatMul, type PhysicsProps, type BodyProps, type PartProps, type PartInfo, type PartMaterial, type Glitch, type Impulse, type Shape, type Baked, type MeshSequenceProps, type GooProps, type GooObject, type GooBake } from "@raea/physics/physics";
