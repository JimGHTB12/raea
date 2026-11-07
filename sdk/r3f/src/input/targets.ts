// What an app draws into a render target itself, done on the CPU where it's a pass raea knows, so the picture a
// material shows from that target is there in raea's frame (docs/r3f.feature R3F13). raea's engine draws the scene;
// three never does, so a target an app fills in useFrame (gl.setRenderTarget(t); gl.render(...)) would otherwise
// be empty. The passes done here are the ones drei's <ContactShadows> makes, which are the common ones:
//
//   a depth pass: the scene drawn with a MeshDepthMaterial as scene.overrideMaterial from an orthographic camera, as
//     three draws it (no depth test, no blending: the last triangle drawn wins, opaque objects front to back), its
//     colour plain depth or drei's tint (ucolor * z * 2, alpha 1 - z, read from the material's onBeforeCompile)
//   a blur pass: three's HorizontalBlurShader or VerticalBlurShader (9 taps, h or v apart in uv) over a target
//
// Anything else drawn into a target is left out, with one warning: the target stays empty.
import * as THREE from "three";

/** a target's pixels: RGBA floats, row 0 at v = 0 (as three's targets hold them) */
interface Pixels {
  w: number;
  h: number;
  data: Float32Array;
}

const drawn = new WeakMap<THREE.Texture, Pixels>();
const warned = new Set<string>();
function warnOnce(k: string, why: string) {
  if (warned.has(k)) return;
  warned.add(k);
  console.warn(`raea/r3f: ${k} ${why}`);
}

/** raea's working size for a target (the passes are in uv, so a smaller one blurs the same) */
const MAX = 256;

function pixelsFor(rt: THREE.RenderTarget): Pixels {
  const w = Math.max(1, Math.min(MAX, rt.width)), h = Math.max(1, Math.min(MAX, rt.height));
  let p = drawn.get(rt.texture);
  if (!p || p.w !== w || p.h !== h) {
    p = { w, h, data: new Float32Array(w * h * 4) };
    drawn.set(rt.texture, p);
  }
  return p;
}

/** a target's picture as raea's frame shows it, or undefined when nothing raea knows was drawn into it */
export function targetPixels(t: THREE.Texture): Pixels | undefined {
  return drawn.get(t);
}

const BLUR = [0.051, 0.0918, 0.12245, 0.1531, 0.1633, 0.1531, 0.12245, 0.0918, 0.051];

/** `gl.render(scene, camera)` into `rt` (the stand-in renderer's), done here when it's a pass raea knows */
export function renderInto(rt: THREE.RenderTarget, scene: THREE.Object3D, camera: THREE.Camera, clear: [number, number, number, number]) {
  const out = pixelsFor(rt);
  const mat = (scene as THREE.Mesh).material as THREE.ShaderMaterial | undefined;
  const u = mat?.uniforms;
  if ((scene as THREE.Mesh).isMesh && u?.tDiffuse && ("h" in u || "v" in u)) {
    const src = u.tDiffuse.value as THREE.Texture | null;
    const from = src ? drawn.get(src) : undefined;
    if (from) blur(from, out, "h" in u ? Number(u.h.value) : 0, "v" in u ? Number(u.v.value) : 0);
    return;
  }
  const over = (scene as THREE.Scene).overrideMaterial;
  if ((scene as THREE.Scene).isScene && (over as THREE.MeshDepthMaterial | null)?.isMeshDepthMaterial && (camera as THREE.OrthographicCamera).isOrthographicCamera) {
    for (let i = 0; i < out.data.length; i += 4) out.data.set(clear, i);
    depth(scene as THREE.Scene, camera as THREE.OrthographicCamera, over as THREE.MeshDepthMaterial, out);
    return;
  }
  warnOnce(`a render into a target (${(scene as THREE.Mesh).type}${mat ? " with " + mat.type : ""})`, "isn't one raea draws (a depth pass or a blur): the target stays empty");
}

/** three's 9-tap blur, `h` and `v` apart in uv, sampled linearly with edges clamped */
function blur(from: Pixels, out: Pixels, h: number, v: number) {
  const src = from.data.slice();
  const { w, h: hh } = from;
  const res = new Float32Array(w * hh * 4);
  const dx = h * w, dy = v * hh;
  for (let y = 0; y < hh; y++) {
    for (let x = 0; x < w; x++) {
      const o = (y * w + x) * 4;
      for (let k = 0; k < 9; k++) {
        const wt = BLUR[k], s = k - 4;
        // the texel centre's sample position, in texels
        const fx = Math.min(w - 1, Math.max(0, x + s * dx)), fy = Math.min(hh - 1, Math.max(0, y + s * dy));
        const x0 = Math.floor(fx), y0 = Math.floor(fy), x1 = Math.min(w - 1, x0 + 1), y1 = Math.min(hh - 1, y0 + 1);
        const tx = fx - x0, ty = fy - y0;
        const a = (y0 * w + x0) * 4, b = (y0 * w + x1) * 4, c = (y1 * w + x0) * 4, d = (y1 * w + x1) * 4;
        for (let ch = 0; ch < 4; ch++) {
          const top = src[a + ch] * (1 - tx) + src[b + ch] * tx, bot = src[c + ch] * (1 - tx) + src[d + ch] * tx;
          res[o + ch] += wt * (top * (1 - ty) + bot * ty);
        }
      }
    }
  }
  if (out.w === w && out.h === hh) out.data.set(res);
}

/** the depth material's colour for depth z (0 near, 1 far): drei's tint when its onBeforeCompile puts one in */
function depthColour(m: THREE.MeshDepthMaterial): (z: number, px: Float32Array, o: number) => void {
  const shader = { uniforms: {} as Record<string, { value: unknown }>, vertexShader: "", fragmentShader: "void main() {\n\tgl_FragColor = vec4( vec3( 1.0 - fragCoordZ ), opacity );\n}" };
  try {
    m.onBeforeCompile?.(shader as unknown as THREE.WebGLProgramParametersWithUniforms, undefined as unknown as THREE.WebGLRenderer);
  } catch {
    // its patch asked for what the stand-in shader hasn't: plain depth
  }
  const tint = shader.uniforms.ucolor?.value as THREE.Color | undefined;
  if (tint && /ucolor \* fragCoordZ \* 2\.0, \( 1\.0 - fragCoordZ \)/.test(shader.fragmentShader)) {
    const { r, g, b } = tint;
    return (z, px, o) => {
      px[o] = r * z * 2;
      px[o + 1] = g * z * 2;
      px[o + 2] = b * z * 2;
      px[o + 3] = 1 - z;
    };
  }
  const a = m.opacity;
  return (z, px, o) => {
    px[o] = px[o + 1] = px[o + 2] = 1 - z;
    px[o + 3] = a;
  };
}

const v = new THREE.Vector3();
const mvp = new THREE.Matrix4();

/** the scene's meshes drawn as `m` from `cam` into `out`, as three would: front faces only, no depth test */
function depth(scene: THREE.Scene, cam: THREE.OrthographicCamera, m: THREE.MeshDepthMaterial, out: Pixels) {
  scene.updateMatrixWorld();
  cam.updateMatrixWorld();
  cam.updateProjectionMatrix();
  const colour = depthColour(m);
  const meshes: { mesh: THREE.Mesh; z: number }[] = [];
  const visit = (o: THREE.Object3D) => {
    if (!o.visible) return;
    const mesh = o as THREE.Mesh;
    if (mesh.isMesh && !(mesh as unknown as THREE.Points).isPoints && mesh.geometry?.attributes?.position) {
      v.setFromMatrixPosition(mesh.matrixWorld).applyMatrix4(cam.matrixWorldInverse);
      meshes.push({ mesh, z: -v.z });
    }
    for (const c of o.children) visit(c);
  };
  visit(scene);
  // three draws opaque objects front to back (its painter sort, by renderOrder, then distance)
  meshes.sort((a, b) => a.mesh.renderOrder - b.mesh.renderOrder || a.z - b.z);
  const { w, h, data } = out;
  const near = cam.near, far = cam.far;
  const sx = new Float32Array(3), sy = new Float32Array(3), sz = new Float32Array(3);
  const inst = new THREE.Matrix4();
  for (const { mesh } of meshes) {
    const pos = mesh.geometry.attributes.position as THREE.BufferAttribute;
    const index = mesh.geometry.index;
    const n = index ? index.count : pos.count;
    const im = mesh as THREE.InstancedMesh;
    const count = im.isInstancedMesh ? im.count : 1;
    for (let k = 0; k < count; k++) {
      mvp.multiplyMatrices(cam.matrixWorldInverse, mesh.matrixWorld);
      if (im.isInstancedMesh) {
        im.getMatrixAt(k, inst);
        mvp.multiply(inst);
      }
      const flip = mvp.determinant() < 0;
      for (let t = 0; t + 2 < n; t += 3) {
        for (let j = 0; j < 3; j++) {
          const i = index ? index.getX(t + j) : t + j;
          v.fromBufferAttribute(pos, i).applyMatrix4(mvp);
          // view space to the target's pixels (x right, y up from row 0) and depth 0..1 from near to far
          sx[j] = ((v.x - cam.left) / (cam.right - cam.left)) * w;
          sy[j] = ((v.y - cam.bottom) / (cam.top - cam.bottom)) * h;
          sz[j] = (-v.z - near) / (far - near);
        }
        const area = (sx[1] - sx[0]) * (sy[2] - sy[0]) - (sx[2] - sx[0]) * (sy[1] - sy[0]);
        // a front face winds counter-clockwise on screen (and the other way under a mirroring matrix)
        if (area === 0 || (area > 0) === flip) continue;
        const x0 = Math.max(0, Math.floor(Math.min(sx[0], sx[1], sx[2]))), x1 = Math.min(w - 1, Math.ceil(Math.max(sx[0], sx[1], sx[2])));
        const y0 = Math.max(0, Math.floor(Math.min(sy[0], sy[1], sy[2]))), y1 = Math.min(h - 1, Math.ceil(Math.max(sy[0], sy[1], sy[2])));
        for (let y = y0; y <= y1; y++) {
          const py = y + 0.5;
          for (let x = x0; x <= x1; x++) {
            const px = x + 0.5;
            const w0 = ((sx[1] - px) * (sy[2] - py) - (sx[2] - px) * (sy[1] - py)) / area;
            const w1 = ((sx[2] - px) * (sy[0] - py) - (sx[0] - px) * (sy[2] - py)) / area;
            const w2 = 1 - w0 - w1;
            if (w0 < 0 || w1 < 0 || w2 < 0) continue;
            const z = w0 * sz[0] + w1 * sz[1] + w2 * sz[2];
            if (z < 0 || z > 1) continue;
            colour(z, data, (y * w + x) * 4);
          }
        }
      }
    }
  }
}
