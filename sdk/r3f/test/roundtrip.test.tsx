// Both ways at once (docs/r3f.feature R3F8): a raea scene drawn by the three.js host and read back by the R3F input
// gives the same scene (places, surfaces, lights in raea's units, the camera); an R3F scene read by the input and drawn
// by the host gives three.js objects equal to R3F's own.
import { describe, expect, test } from "bun:test";
import * as THREE from "three";
import {
  AmbientLight, BufferMesh, Cuboid, DirectionalLight, Fog, Group, Mesh, MeshBasicMaterial, MeshPhysicalMaterial,
  MeshStandardMaterial, PerspectiveCamera, PointLight, RectAreaLight, Sphere, SpotLight, ToneMapping, Torus,
} from "raea/3d";
import { Scene3d } from "../src/three/scene";
import { convert } from "../src/input/scene";
import { stepTo, end, standInRenderer } from "../src/input/session";
import { elementsOf, type Part } from "../src/parts";
import { page } from "./dom";

const near = (a: number, b: number, eps = 1e-4) => Math.abs(a - b) <= eps * Math.max(1, Math.abs(a), Math.abs(b));
const nearAll = (a: number[], b: number[], eps = 1e-4) => a.length === b.length && a.every((x, i) => near(x, b[i], eps));

function hostOf(node: React.ReactNode) {
  const host = new Scene3d();
  host.sync(page(node) as unknown as Element);
  host.fit(640, 360);
  return host;
}

function readBack(host: Scene3d) {
  const gl = standInRenderer();
  host.apply(gl);
  return convert(host.scene, host.camera, gl, 360).parts;
}

const flat = (parts: Part[]): Part[] => parts.flatMap((p) => [p, ...flat(p.children ?? [])]);
const ofType = (parts: Part[], t: string) => flat(parts).filter((p) => p.type === t);

const SCENE = (
  <>
    <PerspectiveCamera position={[4, 3, 6]} lookAt={[0, 0.5, 0]} fov={40} />
    <ToneMapping mode="aces" exposure={0.5} />
    <Fog color="#203040" near={4} far={30} />
    <AmbientLight intensity={0.6} color="#c8d4ff" />
    <DirectionalLight position={[3, 6, 2]} target={[0, 0, 0]} intensity={2} color="#fff4e4" />
    <PointLight position={[-2, 2, 1]} intensity={3} color="#58a6ff" distance={12} />
    <SpotLight position={[0, 5, 0]} target={[0, 0, 0]} intensity={4} angle={0.5} penumbra={0.3} />
    <RectAreaLight position={[0, 2, -3]} target={[0, 1, 0]} width={2} height={1} intensity={2} />
    <Mesh name="box" position={[0, 0.5, 0]} rotation={[0.1, 0.7, 0]}>
      <Cuboid args={[1, 1, 1]} />
      <MeshStandardMaterial color="#e05a2b" roughness={0.35} metalness={0.2} emissive="#220000" emissiveIntensity={2} />
    </Mesh>
    <Group position={[2, 0, 0]} rotation={[0, 0.5, 0]} scale={[1, 2, 1]}>
      <Mesh name="ball" position={[0, 0.5, 0]}>
        <Sphere args={[0.5]} />
        <MeshPhysicalMaterial color="#3fb950" clearcoat={1} clearcoatRoughness={0.1} roughness={0.2} transmission={0.5} ior={1.4} thickness={0.2} />
      </Mesh>
    </Group>
    <Mesh name="ring" position={[-2, 0.5, 0]}>
      <Torus args={[0.5, 0.2]} />
      <MeshBasicMaterial color="#d2a8ff" opacity={0.5} transparent side="double" />
    </Mesh>
    <Mesh name="tri" position={[0, 0, 2]}>
      <BufferMesh positions={[0, 0, 0, 1, 0, 0, 0, 1, 0]} uvs={[0, 1, 1, 1, 0, 0]} normals={[0, 0, 1, 0, 0, 1, 0, 0, 1]} />
      <MeshStandardMaterial color="#ffffff" />
    </Mesh>
  </>
);

describe("raea → three.js → raea", () => {
  const host = hostOf(SCENE);
  const back = readBack(host);

  test("every light comes back in raea's units, colour and place", () => {
    const want: Record<string, { intensity: number; color: string; position?: number[] }> = {
      AmbientLight: { intensity: 0.6, color: "#c8d4ff" },
      DirectionalLight: { intensity: 2, color: "#fff4e4", position: [3, 6, 2] },
      PointLight: { intensity: 3, color: "#58a6ff", position: [-2, 2, 1] },
      SpotLight: { intensity: 4, color: "#ffffff", position: [0, 5, 0] },
      RectAreaLight: { intensity: 2, color: "#ffffff", position: [0, 2, -3] },
    };
    for (const [type, w] of Object.entries(want)) {
      const [l] = ofType(back, type);
      expect(l, type).toBeDefined();
      expect(near(l.props.intensity as number, w.intensity), `${type} intensity ${l.props.intensity}`).toBe(true);
      expect(l.props.color).toBe(w.color);
      if (w.position) expect(nearAll(l.props.position as number[], w.position)).toBe(true);
    }
    const [spot] = ofType(back, "SpotLight");
    expect(near(spot.props.angle as number, 0.5)).toBe(true);
    expect(near(spot.props.penumbra as number, 0.3)).toBe(true);
    expect(ofType(back, "PointLight")[0].props.distance).toBe(12);
    const [rect] = ofType(back, "RectAreaLight");
    expect(nearAll([rect.props.width as number, rect.props.height as number], [2, 1])).toBe(true);
  });

  test("the camera, tone mapping and fog come back", () => {
    const [cam] = ofType(back, "PerspectiveCamera");
    expect(nearAll(cam.props.position as number[], [4, 3, 6])).toBe(true);
    expect(near(cam.props.fov as number, 40)).toBe(true);
    // its look: one unit from the eye toward [0, 0.5, 0]
    const dir = new THREE.Vector3(...(cam.props.lookAt as [number, number, number])).sub(new THREE.Vector3(4, 3, 6));
    expect(dir.normalize().dot(new THREE.Vector3(-4, -2.5, -6).normalize())).toBeGreaterThan(0.99999);
    const tone = ofType(back, "ToneMapping")[0].props;
    expect(tone.mode).toBe("aces");
    expect(near(tone.exposure as number, 0.5)).toBe(true);
    expect(ofType(back, "Fog")[0].props).toEqual({ color: "#203040", near: 4, far: 30 });
  });

  test("meshes come back where they were, with their surfaces", () => {
    const meshes = ofType(back, "Mesh");
    expect(meshes.length).toBe(4);
    const byName = (n: string) => meshes.find((m) => String(m.props.name).endsWith(`:${n}`))!;
    const box = byName("box");
    expect(nearAll(box.props.position as number[], [0, 0.5, 0])).toBe(true);
    const q = new THREE.Quaternion().setFromEuler(new THREE.Euler(0.1, 0.7, 0, "XYZ"));
    expect(nearAll(box.props.quaternion as number[], q.toArray())).toBe(true);
    const std = box.children![1];
    expect(std.type).toBe("MeshStandardMaterial");
    expect(std.props).toMatchObject({ color: "#e05a2b", roughness: 0.35, metalness: 0.2, emissive: "#220000", emissiveIntensity: 2 });
    const phys = byName("ball").children![1];
    expect(phys.type).toBe("MeshPhysicalMaterial");
    expect(phys.props).toMatchObject({ color: "#3fb950", clearcoat: 1, clearcoatRoughness: 0.1, transmission: 0.5, ior: 1.4, thickness: 0.2 });
    const basic = byName("ring").children![1];
    expect(basic.props).toMatchObject({ color: "#d2a8ff", opacity: 0.5, transparent: true, side: "double" });
    // the triangle's arrays, its uvs as raea counts them (v from the top)
    const tri = byName("tri").children![0];
    expect(tri.props.positions).toEqual([0, 0, 0, 1, 0, 0, 0, 1, 0]);
    expect(nearAll(tri.props.uvs as number[], [0, 1, 1, 1, 0, 0])).toBe(true);
  });

  test("a group's transform stays a group's (its child's world place is three's)", () => {
    const groups = ofType(back, "Group");
    const g = groups.find((x) => nearAll(x.props.position as number[], [2, 0, 0]))!;
    expect(g).toBeDefined();
    expect(g.props.scale).toEqual([1, 2, 1]);
    const ball = host.scene.getObjectByName("ball")!;
    expect(nearAll(ball.getWorldPosition(new THREE.Vector3()).toArray(), [2, 1, 0])).toBe(true);
  });

  test("drawn again from what came back, the scene is the same", () => {
    const again = hostOf(<>{elementsOf(back)}</>);
    const twice = readBack(again);
    const strip = (ps: Part[]) => JSON.stringify(flat(ps).map((p) => ({ type: p.type, props: Object.fromEntries(Object.entries(p.props).filter(([k]) => k !== "name").map(([k, v]) => [k, typeof v === "number" ? Number(v.toPrecision(5)) : Array.isArray(v) && typeof v[0] === "number" ? (v as number[]).map((x) => Number(x.toPrecision(5))) : v])) })));
    expect(strip(twice)).toBe(strip(back));
  });
});

describe("R3F → raea → three.js", () => {
  test("an R3F scene drawn by the host is R3F's own scene", () => {
    const s = stepTo("rt", 0, 24, 640, 360, { camera: { position: [3, 2, 5], fov: 45 }, shadows: true }, () => (
      <>
        <ambientLight intensity={0.5} />
        <directionalLight position={[3, 5, 2]} intensity={2.5} castShadow />
        <pointLight position={[-1, 2, 0]} intensity={8} distance={10} color="#ff8800" />
        <group position={[1, 0, 0]} rotation={[0, 0.3, 0]}>
          <mesh name="k" position={[0, 1, 0]} scale={[1, 0.5, 2]} castShadow>
            <boxGeometry args={[1, 1, 1]} />
            <meshStandardMaterial color="#e05a2b" roughness={0.4} metalness={0.1} />
          </mesh>
        </group>
      </>
    ));
    const st = s.state as unknown as { scene: THREE.Scene; camera: THREE.Camera; gl: THREE.WebGLRenderer };
    const parts = convert(st.scene, st.camera, st.gl, 360).parts;
    const host = hostOf(<>{elementsOf(parts)}</>);
    // lights: the same three.js intensity and place
    const lights = (s: THREE.Object3D) => { const out: THREE.Light[] = []; s.traverse((o) => { if ((o as THREE.Light).isLight) out.push(o as THREE.Light); }); return out; };
    const a = lights(st.scene), b = lights(host.scene);
    for (const l of a) {
      const m = b.find((x) => x.type === l.type)!;
      expect(m, l.type).toBeDefined();
      expect(near(m.intensity, l.intensity), `${l.type} ${m.intensity} vs ${l.intensity}`).toBe(true);
      expect(m.color.getHex()).toBe(l.color.getHex());
      expect(nearAll(m.getWorldPosition(new THREE.Vector3()).toArray(), l.getWorldPosition(new THREE.Vector3()).toArray())).toBe(true);
    }
    // the box: the same world matrix and surface, its triangles the same
    const ours = st.scene.getObjectByName("k") as THREE.Mesh;
    let theirs: THREE.Mesh | undefined;
    host.scene.traverse((o) => { if ((o as THREE.Mesh).isMesh && o.name.endsWith(":k")) theirs = o as THREE.Mesh; });
    expect(theirs).toBeDefined();
    st.scene.updateMatrixWorld();
    host.scene.updateMatrixWorld();
    expect(nearAll(theirs!.matrixWorld.toArray(), ours.matrixWorld.toArray())).toBe(true);
    const mt = theirs!.material as THREE.MeshStandardMaterial, mo = ours.material as THREE.MeshStandardMaterial;
    expect([mt.color.getHex(), mt.roughness, mt.metalness]).toEqual([mo.color.getHex(), mo.roughness, mo.metalness]);
    expect(theirs!.geometry.attributes.position.count).toBe(ours.geometry.index ? ours.geometry.attributes.position.count : ours.geometry.attributes.position.count);
    // the camera
    expect(nearAll(host.camera.position.toArray(), st.camera.position.toArray())).toBe(true);
    expect(nearAll(host.camera.quaternion.toArray(), st.camera.quaternion.toArray(), 1e-3)).toBe(true);
    end("rt");
  });
});
