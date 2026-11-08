// The parity cases (docs/r3f.feature R3F8): each drawn by raea's engine (native.tsx, `raea render`) and by three.js in
// Chrome (the harness page), and the two compared by SSIM (tools/parity.sh). `in-*` are R3F scenes, raea's input;
// `out-*` are raea compositions, drawn in React Three Fiber by <Raea> (outputs.tsx).
import { useFrame } from "@react-three/fiber";
import * as PP from "@react-three/postprocessing";
import { ContactShadows, Environment, Float, Text, useGLTF } from "@react-three/drei";
import { useMemo, useRef, type ReactNode } from "react";
import * as THREE from "three";
import { MeshBasicNodeMaterial } from "three/webgpu";
import * as TSLTyped from "three/tsl";
import { composeEffects, configureFontTexture, createText, parseFont, scramble, wipe } from "lettra/three";
import inter from "./fonts/Inter-400.json";

export { OUTPUTS } from "./outputs";

export const SIZE = { width: 640, height: 360, fps: 24 };

// ---- R3F scenes ----
function Spinner() {
  const ref = useRef<THREE.Mesh>(null!);
  useFrame((_s, dt) => {
    ref.current.rotation.y += dt;
    ref.current.rotation.x += dt * 0.5;
  });
  return (
    <mesh ref={ref} position={[0, 0.8, 0]} castShadow receiveShadow>
      <boxGeometry args={[1, 1, 1]} />
      <meshStandardMaterial color="#e05a2b" roughness={0.4} />
    </mesh>
  );
}

const ground = (
  <mesh rotation={[-Math.PI / 2, 0, 0]} receiveShadow>
    <planeGeometry args={[12, 12]} />
    <meshStandardMaterial color="#cfd6e0" roughness={0.9} />
  </mesh>
);

const instanced = (() => {
  const m = new THREE.InstancedMesh(new THREE.SphereGeometry(0.18, 24, 12), new THREE.MeshStandardMaterial({ color: "#f2cc60", roughness: 0.3 }), 9);
  const o = new THREE.Object3D();
  for (let i = 0; i < 9; i++) {
    o.position.set(-2 + (i % 3) * 0.5, 0.2, 1 + Math.floor(i / 3) * 0.5);
    o.updateMatrix();
    m.setMatrixAt(i, o.matrix);
  }
  m.castShadow = true;
  return m;
})();

export interface InputCase {
  canvas: Record<string, unknown>;
  scene: () => ReactNode;
  frame: number;
}

// the approximate kinds' case: a matcap made in code (lit from the upper left), three's normal shading, Phong with a
// coloured highlight, a dashed line, a knot clipped by a material plane, a hemisphere light and a light probe
const matcap = (() => {
  const n = 64, data = new Uint8Array(n * n * 4);
  for (let y = 0; y < n; y++) for (let x = 0; x < n; x++) {
    const u = (x + 0.5) / n * 2 - 1, v = (y + 0.5) / n * 2 - 1, r = Math.min(1, Math.hypot(u, v));
    const nz = Math.sqrt(1 - r * r), lit = Math.max(0, -0.5 * u + 0.5 * v + 0.7 * nz);
    const i = (y * n + x) * 4;
    data.set([40 + 200 * lit, 60 + 150 * lit, 120 + 120 * lit, 255].map(Math.round), i);
  }
  const t = new THREE.DataTexture(data, n, n);
  t.colorSpace = THREE.SRGBColorSpace;
  t.needsUpdate = true;
  return t;
})();
const dashedLine = (() => {
  const g = new THREE.BufferGeometry().setFromPoints([new THREE.Vector3(-3, -1.25, 0), new THREE.Vector3(-1, -0.9, 0), new THREE.Vector3(1, -1.25, 0), new THREE.Vector3(3, -0.9, 0)]);
  const l = new THREE.Line(g, new THREE.LineDashedMaterial({ color: "#f2cc60", dashSize: 0.18, gapSize: 0.1 }));
  l.computeLineDistances();
  return l;
})();
const probe = (() => {
  const sh = new THREE.SphericalHarmonics3();
  sh.coefficients[0].set(0.5, 0.45, 0.6);
  sh.coefficients[1].set(0.25, 0.2, 0.3);
  sh.coefficients[3].set(-0.15, 0.05, 0.1);
  return new THREE.LightProbe(sh, 1);
})();
const clipPlane = new THREE.Plane(new THREE.Vector3(-0.3, -1, 0.2).normalize(), 0.95);

export const INPUTS: Record<string, InputCase> = {
  basic: {
    canvas: { camera: { position: [3, 2.5, 4], fov: 45 }, shadows: true },
    frame: 24,
    scene: () => (
      <>
        <color attach="background" args={["#1d2330"]} />
        <ambientLight intensity={0.4} />
        <directionalLight position={[3, 5, 2]} intensity={2.5} castShadow />
        <pointLight position={[-2, 1.5, 1]} intensity={6} color="#58a6ff" />
        <Spinner />
        {ground}
        <mesh position={[-1.6, 0.5, -0.5]} castShadow>
          <sphereGeometry args={[0.5, 48, 24]} />
          <meshPhysicalMaterial color="#3fb950" clearcoat={1} roughness={0.2} />
        </mesh>
        <mesh position={[1.6, 0.6, -0.4]} castShadow>
          <torusKnotGeometry args={[0.35, 0.12, 128, 16]} />
          <meshStandardMaterial color="#d2a8ff" metalness={0.7} roughness={0.25} />
        </mesh>
      </>
    ),
  },
  lights: {
    canvas: { camera: { position: [0, 3, 6], fov: 50 }, shadows: true },
    frame: 0,
    scene: () => (
      <>
        <color attach="background" args={["#0b0e14"]} />
        <fog attach="fog" args={["#0b0e14", 6, 14]} />
        <spotLight position={[0, 4, 1]} angle={0.6} penumbra={0.4} intensity={40} castShadow />
        <hemisphereLight args={["#bcd3ff", "#3a2a1a", 0.6]} />
        <rectAreaLight position={[-2, 1, 2]} width={1.5} height={1} intensity={6} color="#ff6040" onUpdate={(l) => l.lookAt(0, 0.5, 0)} />
        {ground}
        <mesh position={[0, 0.6, 0]} castShadow>
          <cylinderGeometry args={[0.4, 0.6, 1.2, 48]} />
          <meshLambertMaterial color="#e6e6e6" />
        </mesh>
        <mesh position={[1.5, 0.5, 0.5]} castShadow>
          <icosahedronGeometry args={[0.5, 1]} />
          <meshPhongMaterial color="#4ea1ff" shininess={60} flatShading />
        </mesh>
      </>
    ),
  },
  things: {
    canvas: { camera: { position: [0, 2.2, 5], fov: 45 }, shadows: true },
    frame: 0,
    scene: () => (
      <>
        <color attach="background" args={["#202830"]} />
        <ambientLight intensity={0.5} />
        <directionalLight position={[2, 4, 3]} intensity={2} castShadow />
        {ground}
        <primitive object={instanced} />
        <mesh position={[0.6, 0.5, 0]} castShadow>
          <boxGeometry args={[0.8, 0.8, 0.8]} />
          {["#e05a2b", "#3fb950", "#58a6ff", "#d2a8ff", "#f2cc60", "#ffffff"].map((c, i) => <meshStandardMaterial key={i} attach={`material-${i}`} color={c} />)}
        </mesh>
        <lineSegments position={[2, 0.01, 0]}>
          <edgesGeometry args={[new THREE.BoxGeometry(1, 1, 1)]} />
          <lineBasicMaterial color="#ffffff" />
        </lineSegments>
        <sprite position={[-0.8, 1.6, 0]} scale={[0.6, 0.6, 1]}>
          <spriteMaterial color="#ff4fa3" />
        </sprite>
      </>
    ),
  },
  approx: {
    canvas: { camera: { position: [0, 0.6, 6.5], fov: 45 }, onCreated: ({ gl }: { gl: THREE.WebGLRenderer }) => void (gl.localClippingEnabled = true) },
    frame: 0,
    scene: () => (
      <>
        <color attach="background" args={["#1d2330"]} />
        <hemisphereLight args={["#bcd7ff", "#5a3a20", 0.8]} />
        <primitive object={probe} />
        <directionalLight position={[3, 4, 5]} intensity={1.6} />
        <mesh position={[-2.4, 0.3, 0]}>
          <sphereGeometry args={[0.65, 48, 32]} />
          <meshMatcapMaterial matcap={matcap} />
        </mesh>
        <mesh position={[-0.8, 0.3, 0]} rotation={[0.6, 0.4, 0]}>
          <torusGeometry args={[0.45, 0.2, 24, 64]} />
          <meshNormalMaterial />
        </mesh>
        <mesh position={[0.8, 0.3, 0]}>
          <sphereGeometry args={[0.65, 48, 32]} />
          <meshPhongMaterial color="#3a6ea5" specular="#ff9a3c" shininess={60} />
        </mesh>
        <mesh position={[2.4, 0.3, 0]}>
          <torusKnotGeometry args={[0.42, 0.14, 128, 16]} />
          <meshStandardMaterial color="#d2a8ff" roughness={0.5} clippingPlanes={[clipPlane]} side={THREE.DoubleSide} />
        </mesh>
        <primitive object={dashedLine} />
      </>
    ),
  },
  // a ShadowMaterial floor: only the box's shadow is drawn, over the page's colour
  catch: {
    canvas: { camera: { position: [0, 2.5, 4.5], fov: 45 }, shadows: true },
    frame: 0,
    scene: () => (
      <>
        <color attach="background" args={["#d8c9a8"]} />
        <ambientLight intensity={0.5} />
        <directionalLight position={[2, 5, 1]} intensity={2.5} castShadow />
        <mesh rotation={[-Math.PI / 2, 0, 0]} receiveShadow>
          <planeGeometry args={[12, 12]} />
          <shadowMaterial color="#1a1030" opacity={0.6} />
        </mesh>
        <mesh position={[0, 0.6, 0]} rotation={[0, 0.5, 0]} castShadow>
          <boxGeometry args={[1, 1.2, 1]} />
          <meshStandardMaterial color="#e05a2b" roughness={0.5} />
        </mesh>
      </>
    ),
  },
  // clear glass over a glow: a transmissive, clearcoated knot seen through to a thin emissive one inside it, at
  // emissiveIntensity 1 (a glow raea once drew 998 times too bright, so the glass read as opaque white)
  glass: {
    canvas: { camera: { position: [0, 0, 5], fov: 40 } },
    frame: 0,
    scene: () => (
      <>
        <color attach="background" args={["#0e1116"]} />
        <hemisphereLight args={["#dfe8ff", "#30261c", 1.2]} />
        <directionalLight position={[3, 4, 5]} intensity={2} />
        <group rotation={[0.4, 0.6, 0]}>
          <mesh>
            <torusKnotGeometry args={[1, 0.34, 320, 56]} />
            <meshPhysicalMaterial color="#e6f3ff" transmission={1} thickness={1.4} roughness={0.04} ior={1.45} clearcoat={1} />
          </mesh>
          <mesh>
            <torusKnotGeometry args={[1, 0.07, 320, 24]} />
            <meshStandardMaterial color="#000" emissive="#ff5fa2" emissiveIntensity={1} />
          </mesh>
        </group>
      </>
    ),
  },
  // lettra's effects: a wipe whose front R3F's clock moves (a uniform set in useFrame), reading the text's own
  // attributes (layoutX, glyphIndex, cellUv), and a scramble through a pool of glyphs (a uniformArray)
  lettrafx: {
    canvas: { camera: { position: [0, 0, 5], fov: 45 }, gl: webgpu },
    frame: 12,
    scene: () => (
      <>
        <color attach="background" args={["#1d2330"]} />
        <LettraFx />
      </>
    ),
  },
  // @react-three/postprocessing (src/input/post.ts): an <EffectComposer>'s effects as raea's Grade and ToneMapping
  // (its Bloom is read too, but raea's own bloom draws no glow in this build: out of this case until it does)
  post: {
    canvas: { camera: { position: [3, 2.5, 4], fov: 45 } },
    frame: 0,
    scene: () => (
      <>
        <color attach="background" args={["#1d2330"]} />
        <ambientLight intensity={0.5} />
        <directionalLight position={[2, 5, 1]} intensity={2.5} />
        <mesh position={[0, 0.6, 0]}>
          <sphereGeometry args={[0.6, 32, 16]} />
          <meshStandardMaterial color="#e05a2b" roughness={0.4} />
        </mesh>
        <mesh position={[0, 0, 0]} rotation={[-Math.PI / 2, 0, 0]}>
          <planeGeometry args={[8, 8]} />
          <meshStandardMaterial color="#cfd6e0" roughness={0.9} />
        </mesh>
        <PP.EffectComposer>
          <PP.HueSaturation saturation={-0.3} />
          <PP.Vignette offset={0.3} darkness={0.7} />
          <PP.ToneMapping mode={6} />
        </PP.EffectComposer>
      </>
    ),
  },
  // a typical drei scene: a model by useGLTF floating (<Float>), troika's <Text>, <ContactShadows> under it and an
  // <Environment preset> lighting it
  drei: {
    canvas: { camera: { position: [0, 1, 4.2], fov: 45 } },
    frame: 12,
    scene: () => (
      <>
        <color attach="background" args={["#1d2330"]} />
        <Environment preset="city" />
        <Float speed={0} rotationIntensity={0.6} floatIntensity={0.8}>
          <Knot />
        </Float>
        <Text position={[0, 1.55, 0]} fontSize={0.32} color="#f2cc60" font="fonts/Inter-400.ttf" anchorX="center" anchorY="middle">
          raea drei
        </Text>
        <ContactShadows position={[0, -0.85, 0]} opacity={0.7} blur={2.2} scale={5} far={2} />
      </>
    ),
  },
  // three.js ShaderMaterials (src/input/shader.ts): GLSL translated by naga and run by raea's engine with three's
  // built-ins. A sphere rippled by R3F's clock (a uniform set in useFrame) and points sized by an attribute of their own
  shader: {
    canvas: { camera: { position: [0, 0, 4.5], fov: 45 } },
    frame: 12,
    scene: () => (
      <>
        <color attach="background" args={["#1d2330"]} />
        <Waves />
        <points position={[0, 0, 0.5]}>
          <primitive object={dots} attach="geometry" />
          <shaderMaterial vertexShader={DOT_VERTEX} fragmentShader={DOT_FRAGMENT} transparent depthWrite={false} />
        </points>
      </>
    ),
  },
  // TSL node materials (three/webgpu), compiled to WGSL by three and run by raea's engine (src/input/tsl.ts): lettra's
  // MSDF text (an OFL face, Inter, baked by `lettra bake`: test/parity/fonts) and a gradient pulsing with TSL's time
  lettra: {
    canvas: { camera: { position: [0, 0, 5], fov: 45 }, gl: webgpu },
    frame: 12,
    scene: () => (
      <>
        <color attach="background" args={["#1d2330"]} />
        <Lettra />
        <mesh position={[0, -1.3, 0]} rotation={[0.4, 0.6, 0]}>
          <boxGeometry args={[0.9, 0.9, 0.9]} />
          <primitive object={pulse} attach="material" />
        </mesh>
      </>
    ),
  },
};

const WAVE_VERTEX = /* glsl */ `
uniform float uTime;
varying vec3 vNormal;
varying vec2 vUv;
void main() {
  vec3 p = position + normal * sin(position.y * 8.0 + uTime * 3.0) * 0.08;
  vNormal = normalize(normalMatrix * normal);
  vUv = uv;
  gl_Position = projectionMatrix * modelViewMatrix * vec4(p, 1.0);
}`;
const WAVE_FRAGMENT = /* glsl */ `
uniform vec3 uColor;
varying vec3 vNormal;
varying vec2 vUv;
void main() {
  float light = 0.25 + 0.75 * max(dot(normalize(vNormal), normalize(vec3(0.4, 0.8, 0.6))), 0.0);
  float stripe = step(0.5, fract(vUv.x * 8.0));
  gl_FragColor = vec4(mix(uColor, vec3(1.0), stripe * 0.35) * light, 1.0);
}`;

function Waves() {
  const m = useMemo(() => new THREE.ShaderMaterial({ vertexShader: WAVE_VERTEX, fragmentShader: WAVE_FRAGMENT, uniforms: { uTime: { value: 0 }, uColor: { value: new THREE.Color("#e05a2b") } } }), []);
  useFrame(({ clock }) => void (m.uniforms.uTime.value = clock.elapsedTime));
  return (
    <mesh rotation={[0.3, 0.4, 0]} material={m}>
      <sphereGeometry args={[1, 48, 24]} />
    </mesh>
  );
}

const DOT_VERTEX = /* glsl */ `
attribute float aScale;
varying float vScale;
void main() {
  vScale = aScale;
  gl_Position = projectionMatrix * modelViewMatrix * vec4(position, 1.0);
  gl_PointSize = 6.0 + aScale * 18.0;
}`;
const DOT_FRAGMENT = /* glsl */ `
varying float vScale;
void main() {
  float d = length(gl_PointCoord - vec2(0.5));
  if (d > 0.5) discard;
  gl_FragColor = vec4(mix(vec3(0.35, 0.65, 1.0), vec3(0.95, 0.8, 0.38), vScale), 1.0 - smoothstep(0.35, 0.5, d));
}`;

const dots = (() => {
  const g = new THREE.BufferGeometry();
  const pos: number[] = [], scale: number[] = [];
  for (let i = 0; i < 24; i++) {
    const a = (i / 24) * Math.PI * 2;
    pos.push(Math.cos(a) * 1.7, Math.sin(a) * 1.1, 0);
    scale.push((i % 6) / 5);
  }
  g.setAttribute("position", new THREE.Float32BufferAttribute(pos, 3));
  g.setAttribute("aScale", new THREE.Float32BufferAttribute(scale, 1));
  return g;
})();

/** three's WebGPURenderer on its WebGL 2 backend (a TSL node material needs it; headless Chrome may have no WebGPU) */
async function webgpu(props: object) {
  const { WebGPURenderer } = await import("three/webgpu");
  const r = new WebGPURenderer({ ...(props as object), forceWebGL: true, antialias: true });
  await r.init();
  return r;
}

const pulse = (() => {
  // untyped: @types/three's chained TSL types make tsc take minutes and gigabytes
  const { color, mix, sin, time, uv } = TSLTyped as unknown as Record<string, (...a: unknown[]) => any> & { time: any };
  const m = new MeshBasicNodeMaterial();
  m.colorNode = mix(color("#58a6ff"), color("#e05a2b"), uv().y).mul(sin(time.mul(2)).mul(0.25).add(0.75));
  return m;
})();

function Knot() {
  const { scene } = useGLTF("models/knot.glb");
  return <primitive object={scene} />;
}

function LettraFx() {
  const text = useMemo(() => {
    const map = new THREE.TextureLoader().load("fonts/Inter-400.png", (t) => void configureFontTexture(t));
    Object.assign(map, { flipY: false, colorSpace: THREE.NoColorSpace, generateMipmaps: false, minFilter: THREE.LinearFilter, magFilter: THREE.LinearFilter });
    const effect = composeEffects(wipe(), scramble({ font: inter as unknown as Parameters<typeof scramble>[0]["font"], chars: "raeR3FlitMSD", rate: 0 }));
    const t = createText({ font: parseFont(inter), map, text: "raea R3F\nlettra MSDF", layout: { align: "center" }, material: { fill: "#58a6ff", effect } });
    (t.uniforms as unknown as { scramble: { value: number } }).scramble.value = 0.6;
    return t;
  }, []);
  useFrame(({ clock }) => void ((text.uniforms as unknown as { wipeIn: { value: number } }).wipeIn.value = Math.min(1, 0.2 + clock.elapsedTime)));
  return <primitive object={text.mesh} />;
}

function Lettra() {
  const text = useMemo(() => {
    // lettra's atlas settings now (raea reads them as it compiles the graph), its upload once the picture is in
    const map = new THREE.TextureLoader().load("fonts/Inter-400.png", (t) => void configureFontTexture(t));
    Object.assign(map, { flipY: false, colorSpace: THREE.NoColorSpace, generateMipmaps: false, minFilter: THREE.LinearFilter, magFilter: THREE.LinearFilter });
    const t = createText({ font: parseFont(inter), map, text: "raea R3F\nlettra MSDF", layout: { align: "center" }, material: { fill: "#f2cc60" } });
    t.mesh.position.y = 0.45;
    return t;
  }, []);
  return <primitive object={text.mesh} />;
}
