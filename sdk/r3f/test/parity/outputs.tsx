// The parity cases' raea compositions (fixtures.tsx), `out-*`: each drawn by raea's engine and by React Three Fiber
// through <Raea>, and the two compared by SSIM (tools/parity.sh).
import type { ReactNode } from "react";
import { AbsoluteFill, useCurrentFrame } from "raea";
import {
  Bloom, Cuboid, Entities, Grade, LensDistortion, Fog, Sphere, Text3D, ToneMapping, Torus,
} from "raea/3d";
import { createWorld } from "raea/ecs";
import {
  AmbientLight, DirectionalLight, Group, Mesh, MeshBasicMaterial, MeshPhysicalMaterial, MeshStandardMaterial,
  PerspectiveCamera, PointLight, RectAreaLight, SpotLight,
} from "@raea/render/three";

function OutBasic({ tone = "aces" }: { tone?: "aces" | "tony" }) {
  const f = useCurrentFrame();
  return (
    <AbsoluteFill style={{ background: "#1d2330" }}>
      <PerspectiveCamera position={[3, 2.5, 4]} lookAt={[0, 0.5, 0]} fov={45} />
      {tone === "aces" && <ToneMapping mode="aces" />}
      <AmbientLight intensity={0.4} />
      <DirectionalLight position={[3, 5, 2]} target={[0, 0, 0]} intensity={2.5} />
      <PointLight position={[-2, 1.5, 1]} intensity={6} color="#58a6ff" />
      <Mesh name="ground" rotation={[-Math.PI / 2, 0, 0]}>
        <Cuboid args={[12, 12, 0.001]} />
        <MeshStandardMaterial color="#cfd6e0" roughness={0.9} />
      </Mesh>
      <Mesh name="box" position={[0, 0.8, 0]} rotation={[f / 48, f / 24, 0]}>
        <Cuboid args={[1, 1, 1]} />
        <MeshStandardMaterial color="#e05a2b" roughness={0.4} />
      </Mesh>
      <Mesh name="ball" position={[-1.6, 0.5, -0.5]}>
        <Sphere args={[0.5]} />
        <MeshPhysicalMaterial color="#3fb950" clearcoat={1} roughness={0.2} />
      </Mesh>
      <Group position={[1.6, 0.6, -0.4]} rotation={[0.4, 0, 0]}>
        <Mesh name="ring">
          <Torus args={[0.4, 0.15]} />
          <MeshStandardMaterial color="#d2a8ff" metalness={0.7} roughness={0.25} />
        </Mesh>
      </Group>
    </AbsoluteFill>
  );
}

function OutLights() {
  return (
    <AbsoluteFill style={{ background: "#0b0e14" }}>
      <PerspectiveCamera position={[0, 3, 6]} lookAt={[0, 0.5, 0]} fov={50} />
      <ToneMapping mode="aces" />
      <Fog color="#0b0e14" near={6} far={14} />
      <SpotLight position={[0, 4, 1]} target={[0, 0, 0]} angle={0.6} penumbra={0.4} intensity={1.5} castShadow />
      <RectAreaLight position={[-2, 1, 2]} target={[0, 0.5, 0]} width={1.5} height={1} intensity={1} color="#ff6040" />
      <AmbientLight intensity={0.2} />
      <Mesh name="ground" rotation={[-Math.PI / 2, 0, 0]}>
        <Cuboid args={[12, 12, 0.001]} />
        <MeshStandardMaterial color="#cfd6e0" roughness={0.9} />
      </Mesh>
      <Mesh name="glow" position={[1.5, 0.5, 0.5]}>
        <Sphere args={[0.4]} />
        <MeshBasicMaterial color="#4ea1ff" />
      </Mesh>
      <Mesh name="sign" position={[-0.9, 0.3, 0]}>
        <Text3D size={0.5} depth={0.15}>R3F</Text3D>
        <MeshStandardMaterial color="#f2cc60" roughness={0.5} />
      </Mesh>
    </AbsoluteFill>
  );
}

function OutEffects({ fx }: { fx: "bloom" | "grade" | "lens" }) {
  return (
    <AbsoluteFill style={{ background: "#0b0e14" }}>
      <PerspectiveCamera position={[0, 1.5, 5]} lookAt={[0, 0.5, 0]} fov={45} />
      <ToneMapping mode="aces" />
      {fx === "bloom" && <Bloom intensity={2} threshold={0.6} />}
      {fx === "grade" && <Grade saturation={1.3} vignette={0.5} midtones={{ contrast: 1.1 }} />}
      {fx === "lens" && <LensDistortion k1={0.15} />}
      <AmbientLight intensity={0.3} />
      <DirectionalLight position={[2, 4, 3]} target={[0, 0, 0]} intensity={1.5} />
      <Mesh name="ground" rotation={[-Math.PI / 2, 0, 0]}>
        <Cuboid args={[12, 12, 0.001]} />
        <MeshStandardMaterial color="#8090a0" roughness={0.9} />
      </Mesh>
      <Mesh name="lamp" position={[-1, 0.6, 0]}>
        <Sphere args={[0.5]} />
        <MeshStandardMaterial color="#000000" emissive="#ffb050" emissiveIntensity={4} />
      </Mesh>
      <Mesh name="box" position={[1, 0.5, 0]} rotation={[0, 0.6, 0]}>
        <Cuboid args={[1, 1, 1]} />
        <MeshStandardMaterial color="#58a6ff" roughness={0.4} />
      </Mesh>
    </AbsoluteFill>
  );
}

function OutCatch() {
  return (
    <AbsoluteFill style={{ background: "#d8c9a8" }}>
      <PerspectiveCamera position={[0, 2.5, 4.5]} lookAt={[0, 0.3, 0]} fov={45} />
      <ToneMapping mode="aces" />
      <AmbientLight intensity={0.5} />
      <DirectionalLight position={[2, 5, 1]} target={[0, 0, 0]} intensity={2.5} castShadow />
      <Mesh name="floor" rotation={[-Math.PI / 2, 0, 0]} shadow="catch">
        <Cuboid args={[12, 12, 0.001]} />
        <MeshBasicMaterial color="#1a1030" opacity={0.6} />
      </Mesh>
      <Mesh name="box" position={[0, 0.6, 0]} rotation={[0, 0.5, 0]}>
        <Cuboid args={[1, 1.2, 1]} />
        <MeshStandardMaterial color="#e05a2b" roughness={0.5} />
      </Mesh>
    </AbsoluteFill>
  );
}

// an ECS world drawn with no React element per entity (<Entities>): a ship and its gun, turned by a system
const fleet = createWorld();
const ship = fleet.add({ Mesh: { shape: "rounded_box(2, 0.6, 1, 0.1)", color: "#2563eb", roughness: 0.5, metalness: 0 }, Transform: { x: 0, y: 0.5, z: 0, ry: 0 } });
fleet.add({ Mesh: { shape: "cylinder(0.15, 1.2)", color: "#f59e0b", roughness: 0.4, metalness: 0 }, Transform: { x: 0, y: 0.6, z: 0, rx: 0.4 } }, { parent: ship });
fleet.system((w, f) => w.insert(ship, { Transform: { x: 0, y: 0.5, z: 0, ry: f * 0.05 } }));

function OutEntities() {
  const f = useCurrentFrame();
  return (
    <AbsoluteFill style={{ background: "#0b1020" }}>
      <PerspectiveCamera position={[0, 2, 6]} lookAt={[0, 0.5, 0]} fov={45} />
      <ToneMapping mode="aces" />
      <AmbientLight intensity={0.4} />
      <DirectionalLight position={[2, 4, 3]} target={[0, 0, 0]} intensity={2} />
      <Mesh name="ground" rotation={[-Math.PI / 2, 0, 0]}>
        <Cuboid args={[12, 12, 0.001]} />
        <MeshStandardMaterial color="#8090a0" roughness={0.9} />
      </Mesh>
      <Entities world={fleet.at(f)} />
    </AbsoluteFill>
  );
}

export const OUTPUTS: Record<string, { component: () => ReactNode; frame: number }> = {
  basic: { component: () => <OutBasic />, frame: 24 },
  // raea's default tone mapping, TonyMcMapface
  tony: { component: () => <OutBasic tone="tony" />, frame: 24 },
  lights: { component: OutLights, frame: 0 },
  // one effect a case: each pass measured on its own
  bloom: { component: () => <OutEffects fx="bloom" />, frame: 0 },
  grade: { component: () => <OutEffects fx="grade" />, frame: 0 },
  lens: { component: () => <OutEffects fx="lens" />, frame: 0 },
  // shadow="catch" (three's ShadowMaterial): only the shadow on the floor
  catch: { component: OutCatch, frame: 0 },
  // an ECS world's entities (<Entities>), frame 10 of its system
  entities: { component: OutEntities, frame: 10 },
};
