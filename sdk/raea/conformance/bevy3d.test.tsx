// Bevy's names for 3D (docs/sdk.feature S2a): Transform and Quat as Bevy's, and Mesh3d, Camera3d, StandardMaterial and
// the shape primitives drawing what the three.js-shaped names draw for the same scene.
import { describe, expect, test } from "bun:test";
import * as three from "@raea/render/three";
import { Capsule, Cuboid, Capsule3d, Cylinder, DepthOfField, Entity, Model, Plane3d, Screen, Sphere, Torus } from "../src/3d";
import { Group, Mesh, MeshBasicMaterial, MeshStandardMaterial, MeshToonMaterial, PerspectiveCamera } from "@raea/render/three";
// the retired three.js shapes, compared against (M5: no longer exported from raea)
import { Box, Plane } from "@raea/render/3d/shapes";
import { Affine2, AmbientLight, Camera3d, DirectionalLight, ExtendedMaterial, GridExtension, LinearRgba, PaintExtension, ToonExtension, WearExtension, Mesh3d, PointLight, Quat, RectLight, SpotLight, Srgba, StandardMaterial, Transform, eulerOf, light_consts } from "@raea/render/bevy3d";
import { Circle, ColorMaterial, Ellipse, Mesh2d, Rectangle, RegularPolygon, Shape2D } from "raea/sprite";
import { ToonMaterial } from "raea/pbr";
import { ConicalFrustum } from "raea/mesh";
import { renderFrame } from "../src/test";
import { track } from "raea";

const close = (a: number[], b: number[], eps = 1e-9) => a.every((x, i) => Math.abs(x - b[i]) < eps);
const html = (el: React.ReactElement) => renderFrame(el, { frame: 0, width: 320, height: 240, fps: 30, durationInFrames: 30 });
/** a Bevy StandardMaterial's emissive strength as written, which the old material takes as emissive_nits (core/scene emissive_power) */
const nits = (k: number) => ({ emissive_nits: k }) as object;

describe("Quat and Transform", () => {
  test("eulerOf inverts a composition's rotation, Quat::from_euler(EulerRot::XYZ, x, y, z) (scene3d.rs local)", () => {
    for (const [x, y, z] of [[0.3, -1.1, 0.7], [Math.PI / 2 - 0.01, 0, 0], [2.5, -0.5, -2.9], [1.2, 0, 0], [Math.PI / 2, 0, 0.9]]) {
      const q = Quat.mul(Quat.mul(Quat.fromRotationX(x), Quat.fromRotationY(y)), Quat.fromRotationZ(z));
      expect(close(eulerOf(q), [x, y, z], 1e-9)).toBe(true);
      expect(close(Quat.fromEuler("XYZ", x, y, z), q, 1e-12)).toBe(true);
    }
  });

  test("eulerOf at gimbal lock (y = ±π/2) gives the same rotation, its turn about x and z as x", () => {
    for (const [x, y, z] of [[0, Math.PI / 2, 0.9], [0.4, -Math.PI / 2, -1.3], [2.0, Math.PI / 2, 0.5]]) {
      const q = Quat.fromEuler("XYZ", x, y, z);
      const e = eulerOf(q);
      const back = Quat.fromEuler("XYZ", e[0], e[1], e[2]);
      // the same rotation: q and back agree up to sign
      const dot = q.reduce((s, v, i) => s + v * back[i], 0);
      expect(Math.abs(dot)).toBeCloseTo(1, 9);
      expect(e[2]).toBe(0);
    }
  });

  test("fromEuler composes intrinsic turns as glam's Quat::from_euler", () => {
    const q = Quat.fromEuler("XYZ", 0.4, -0.2, 1.3);
    const want = Quat.mul(Quat.mul(Quat.fromRotationX(0.4), Quat.fromRotationY(-0.2)), Quat.fromRotationZ(1.3));
    expect(close(q, want, 1e-12)).toBe(true);
    expect(close(Quat.fromEuler("ZYX", 0, 0, 0), Quat.IDENTITY, 1e-12)).toBe(true);
    expect(() => Quat.fromEuler("XXZ" as "XYZ", 0, 0, 0)).toThrow(/order is one of/);
  });

  test("lookingAt turns -Z toward the target and keeps +Y up, as Bevy's Transform::looking_at", () => {
    const t = Transform.fromXyz(0, 2, 7).lookingAt([0, 0.5, 0]);
    const f = Quat.mulVec3(t.rotation, [0, 0, -1]);
    const want = [0, -1.5, -7].map((v) => v / Math.hypot(1.5, 7));
    expect(close(f, want, 1e-9)).toBe(true);
    expect(Quat.mulVec3(t.rotation, [0, 1, 0])[1]).toBeGreaterThan(0.9);
    expect(Math.hypot(...t.rotation)).toBeCloseTo(1, 12);
    // Bevy's fallback, not a throw: a target at the eye looks down -Z (the identity)
    expect(Transform.fromXyz(0, 0, 0).lookingAt([0, 0, 0]).rotation.map((v) => v + 0)).toEqual([0, 0, 0, 1]);
  });

  test("Transform is also the component's handle, so a program can `with(Transform, value)`", () => {
    expect(Transform.path).toBe("bevy_transform::components::transform::Transform");
    expect(JSON.parse(JSON.stringify(Transform.fromXyz(1, 2, 3).withScale(2)))).toEqual({ translation: [1, 2, 3], rotation: [0, 0, 0, 1], scale: [2, 2, 2] });
  });
});

describe("Bevy's shapes draw the same args", () => {
  test("Cuboid's half_size, Sphere's radius, Cylinder's half_height, Torus's radii, Capsule3d's half_length", () => {
    expect(html(<Mesh><Cuboid half_size={[1, 0.5, 2]} /></Mesh>)).toBe(html(<Mesh><Box args={[2, 1, 4]} /></Mesh>));
    expect(html(<Mesh><Sphere radius={0.5} /></Mesh>)).toBe(html(<Mesh><Sphere args={[0.5]} /></Mesh>));
    expect(html(<Mesh><Cylinder radius={0.3} half_height={1} /></Mesh>)).toBe(html(<Mesh><Cylinder args={[0.3, 0.3, 2]} /></Mesh>));
    expect(html(<Mesh><Torus major_radius={0.75} minor_radius={0.25} /></Mesh>)).toBe(html(<Mesh><Torus args={[0.75, 0.25]} /></Mesh>));
    expect(html(<Mesh><Capsule3d radius={0.4} half_length={0.6} /></Mesh>)).toContain(html(<Mesh><Capsule args={[0.4, 1.2]} /></Mesh>).match(/data-shape="[^"]*"/)![0]);
  });

  test("Cuboid.fromSize is Bevy's Cuboid::from_size: half of each side", () => {
    expect(Cuboid.fromSize([2, 1, 4])).toEqual({ half_size: [1, 0.5, 2] });
    expect(html(<Mesh><Cuboid {...Cuboid.fromSize([2, 1, 4])} /></Mesh>)).toBe(html(<Mesh><Box args={[2, 1, 4]} /></Mesh>));
    expect(() => Cuboid.fromSize([1, 2] as never)).toThrow();
  });

  test("ConicalFrustum is the tapered Cylinder, Bevy's defaults 0.25, 0.5 and 0.5", () => {
    expect(html(<Mesh><ConicalFrustum radius_top={0.2} radius_bottom={0.6} height={1.5} /></Mesh>)).toBe(html(<Mesh><Cylinder args={[0.2, 0.6, 1.5]} /></Mesh>));
    expect(html(<Mesh><ConicalFrustum /></Mesh>)).toBe(html(<Mesh><Cylinder args={[0.25, 0.5, 0.5]} /></Mesh>));
  });

  test("Plane3d is Bevy's one-faced plane facing its normal, its sign kept (plane(nx, ny, nz, w, h)); tilted is refused", () => {
    const shape = (node: React.ReactElement) => /data-shape="([^"]*)"/.exec(html(node))?.[1];
    // Bevy's default faces +Y, half_size.y along z
    expect(shape(<Mesh><Plane3d half_size={[2, 3]} /></Mesh>)).toBe("plane(0, 1, 0, 4, 6)");
    expect(shape(<Mesh><Plane3d /></Mesh>)).toBe("plane(0, 1, 0, 1, 1)");
    // facing down is its own plane, not the one facing up (a box of no depth had both faces)
    expect(shape(<Mesh><Plane3d normal={[0, -1, 0]} half_size={[2, 3]} /></Mesh>)).toBe("plane(0, -1, 0, 4, 6)");
    expect(shape(<Mesh><Plane3d normal={[-1, 0, 0]} half_size={[2, 0.75]} /></Mesh>)).toBe("plane(-1, 0, 0, 4, 1.5)");
    expect(() => html(<Mesh><Plane3d normal={[0, 1, 1]} /></Mesh>)).toThrow(/along an axis/);
  });

  test("a missing Bevy field takes Bevy's default; args and fields together are refused", () => {
    expect(html(<Mesh><Cuboid half_size={undefined} /></Mesh>)).toBe(html(<Mesh><Cuboid /></Mesh>));
    expect(html(<Mesh><Sphere radius={0.5} /></Mesh>)).toBe(html(<Mesh><Sphere radius={undefined as unknown as number} args={[0.5]} /></Mesh>));
    expect(() => html(<Mesh><Sphere radius={1} args={[1]} /></Mesh>)).toThrow(/args or Bevy's fields/);
  });
});

describe("Mesh3d, Camera3d and StandardMaterial", () => {
  test("Bevy's DepthOfField on a Camera3d: focal_distance is the camera's focus, its fields the old lens's", () => {
    const cam = <PerspectiveCamera position={[0, 0, 0]} lookAt={[0, 0, -1]} fov={45} focus={7} />;
    expect(html(<Camera3d><DepthOfField focal_distance={7} aperture_f_stops={2.8} mode="Gaussian" max_circle_of_confusion_diameter={32} /></Camera3d>)).toBe(
      html(<>{cam}<DepthOfField mode="gaussian" aperture={2.8} maxBlur={32} /></>),
    );
    // Bevy's defaults: focal_distance 10, f/1
    expect(html(<Camera3d><DepthOfField /></Camera3d>)).toBe(html(<><PerspectiveCamera position={[0, 0, 0]} lookAt={[0, 0, -1]} fov={45} focus={10} /><DepthOfField aperture={1} /></>));
    expect(html(<Camera3d><DepthOfField aperture_f_stops={2} /></Camera3d>)).toContain(html(<DepthOfField aperture={2} />));
    expect(() => html(<DepthOfField focal_distance={3} />)).toThrow(/inside the Camera3d/);
    expect(() => html(<DepthOfField aperture={2} aperture_f_stops={2} />)).toThrow(/not both/);
  });

  test("an Entity with Bevy's Transform is a parent, the old Group: its children placed in it", () => {
    const q = Quat.fromEuler("XYZ", 0.3, -0.2, 0.1);
    expect(html(<Entity transform={Transform.fromXyz(1, 2, 3).withRotation(q).withScale([2, 2, 2])} pass="fg"><Mesh><Box /></Mesh></Entity>)).toBe(
      html(<Group position={[1, 2, 3]} rotation={eulerOf(q)} scale={[2, 2, 2]} pass="fg"><Mesh><Box /></Mesh></Group>),
    );
    expect(html(<Entity transform={Transform.IDENTITY}><Mesh><Box /></Mesh></Entity>)).toBe(html(<Group scale={[1, 1, 1]}><Mesh><Box /></Mesh></Group>));
    expect(() => html(<Entity transform={Transform.IDENTITY} position={[0, 1, 0]} />)).toThrow(/not both/);
  });

  test("Mesh3d's transform is the Mesh's place; Hidden draws nothing", () => {
    const q = Quat.fromRotationY(0.5);
    // the rotation stays Bevy's quaternion (docs/control.feature FC1d); one an `animate` turns keeps the Euler angles
    // its tracks patch
    expect(html(<Mesh3d transform={Transform.fromXyz(1, 2, 3).withRotation(q)}><Sphere radius={0.5} /></Mesh3d>)).toBe(
      html(<Mesh position={[1, 2, 3]} quaternion={q} scale={[1, 1, 1]}><Sphere args={[0.5]} /></Mesh>),
    );
    const spin = { rotateY: track({ 0: 0, 10: 1 }) };
    expect(html(<Mesh3d transform={Transform.fromXyz(1, 2, 3).withRotation(q)} animate={spin}><Sphere radius={0.5} /></Mesh3d>)).toBe(
      html(<Mesh position={[1, 2, 3]} rotation={eulerOf(q)} scale={[1, 1, 1]} animate={spin}><Sphere args={[0.5]} /></Mesh>),
    );
    expect(html(<Mesh3d visibility="Hidden"><Sphere /></Mesh3d>)).toBe("");
  });

  test("Camera3d looks along its -Z: the same camera as a PerspectiveCamera looking at the point ahead, fov in radians", () => {
    const cam = html(<Camera3d transform={Transform.fromXyz(0, 2, 7).lookingAt([0, 0.5, 0])} projection={{ Perspective: { fov: Math.PI / 4 } }} />);
    const props = JSON.parse(cam.match(/data-props="([^"]*)"/)![1].replace(/&quot;/g, '"'));
    expect(props.fov).toBeCloseTo(45, 12);
    const dir = [props.lookAt[0] - 0, props.lookAt[1] - 2, props.lookAt[2] - 7];
    expect(close(dir.map((v) => v / Math.hypot(...dir)), [0, -1.5, -7].map((v) => v / Math.hypot(1.5, 7)), 1e-9)).toBe(true);
    expect(html(<Camera3d />)).toBe(html(<PerspectiveCamera position={[0, 0, 0]} lookAt={[0, 0, -1]} fov={45} />));
    // no `up` for a camera that doesn't roll (one sends the row down the engine's roll path); a rolled one keeps it
    expect(props.up).toBeUndefined();
    const t = Transform.fromXyz(0, 2, 7).lookingAt([0, 0.5, 0]);
    const rolled = html(<Camera3d transform={t.withRotation(Quat.mul(t.rotation, Quat.fromRotationZ(0.3)))} />);
    const up = JSON.parse(rolled.match(/data-props="([^"]*)"/)![1].replace(/&quot;/g, '"')).up;
    expect(up[0]).toBeCloseTo(-Math.sin(0.3), 9);
  });

  test("StandardMaterial takes Bevy's fields and defaults; unlit is MeshBasicMaterial", () => {
    expect(html(<StandardMaterial base_color="#cc8844" perceptual_roughness={0.4} metallic={1} />)).toBe(html(<MeshStandardMaterial color="#cc8844" roughness={0.4} metalness={1} />));
    expect(html(<StandardMaterial />)).toBe(html(<MeshStandardMaterial color="#ffffff" roughness={0.5} metalness={0} />));
    expect(html(<StandardMaterial base_color="#ff0000" unlit />)).toBe(html(<MeshBasicMaterial color="#ff0000" />));
    expect(html(<StandardMaterial alpha_mode={{ Mask: 0.3 }} cull_mode={null} />)).toBe(html(<MeshStandardMaterial color="#ffffff" roughness={0.5} metalness={0} alphaMode="mask" alphaCutoff={0.3} opacity={1} side="double" />));
  });

  test("WearExtension is the old material's wear, its colours camelCase there", () => {
    expect(html(<ExtendedMaterial base={<StandardMaterial base_color="#d9c69c" perceptual_roughness={0.55} />} extension={<WearExtension amount={0.6} chip_color="#333333" seed={2} />} />)).toBe(html(<MeshStandardMaterial color="#d9c69c" roughness={0.55} metalness={0} wear={{ amount: 0.6, seed: 2, chipColor: "#333333" }} />));
    expect(() => html(<ExtendedMaterial base={<StandardMaterial unlit />} extension={<WearExtension amount={0.5} />} />)).toThrow(/unlit/);
  });

  test("inside a Model a StandardMaterial re-covers with only the fields given, as the old material did", () => {
    const m = (el: React.ReactElement) => html(<Model src="chair.glb">{el}</Model>);
    expect(m(<StandardMaterial material="Fabric" base_color="#7a3b2e" />)).toBe(m(<MeshStandardMaterial color="#7a3b2e" material="Fabric" />));
    expect(m(<StandardMaterial perceptual_roughness={0.3} />)).toBe(m(<MeshStandardMaterial roughness={0.3} />));
    expect(m(<StandardMaterial base_color="#c03" base_color_texture={null} />)).toBe(m(<MeshStandardMaterial color="#c03" map={null} />));
    expect(m(<StandardMaterial base_color="#000000" emissive={LinearRgba.from(Srgba.hex("#ffaa55")).mul(700)} />)).toBe(m(<MeshStandardMaterial color="#000000" emissive="#ffaa55" {...nits(700)} />));
    expect(html(<StandardMaterial base_color="#7a3b2e" />)).toBe(html(<MeshStandardMaterial color="#7a3b2e" roughness={0.5} metalness={0} />));
  });

  test("a texture field with its sampler ({ path, sampler }) is the map with that map's sampler", () => {
    const near = { filter: "nearest" } as const;
    expect(html(<StandardMaterial base_color_texture={{ path: "a.png", sampler: near }} unlit />)).toBe(html(<MeshBasicMaterial color="#ffffff" map="a.png" sampler={near} />));
    expect(html(<StandardMaterial base_color_texture={{ path: "a.png", sampler: near }} normal_map_texture="n.png" />)).toBe(html(<MeshStandardMaterial color="#ffffff" roughness={0.5} metalness={0} map="a.png" normalMap="n.png" samplers={{ map: near }} />));
    expect(() => html(<StandardMaterial base_color_texture={{ path: "a.png", sampler: { filter: "soft" as "nearest" } }} />)).toThrow(/sampler.filter/);
  });

  test("metallic_roughness_texture's { roughness, metalness } are the separate maps the engine packs as they load", () => {
    expect(html(<StandardMaterial metallic_roughness_texture={{ roughness: "r.jpg" }} perceptual_roughness={1} />)).toBe(html(<MeshStandardMaterial color="#ffffff" roughness={1} metalness={0} roughnessMap="r.jpg" />));
    expect(html(<StandardMaterial metallic_roughness_texture={{ roughness: "r.jpg", metalness: "m.jpg" }} metallic={1} />)).toBe(html(<MeshStandardMaterial color="#ffffff" roughness={0.5} metalness={1} roughnessMap="r.jpg" metalnessMap="m.jpg" />));
    expect(() => html(<StandardMaterial metallic_roughness_texture={{}} />)).toThrow(/roughness or a metalness/);
  });

  test("Bevy's further fields (fog_enabled, specular_tint, anisotropy, attenuation, parallax, depth_bias, ...) are the old material's", () => {
    expect(html(<StandardMaterial fog_enabled={false} specular_tint="#00ff00" anisotropy_strength={0.5} anisotropy_rotation={1} attenuation_distance={2} attenuation_color="#ffeecc" depth_map="d.png" parallax_depth_scale={0.05} parallax_mapping_method={{ Relief: { max_steps: 8 } }} max_parallax_layer_count={12} depth_bias={1} opaque_render_method="Deferred" diffuse_transmission={0.2} />)).toBe(
      html(<MeshStandardMaterial color="#ffffff" roughness={0.5} metalness={0} depthMap="d.png" fog={false} specularTint="#00ff00" anisotropy={{ strength: 0.5, rotation: 1 }} attenuation={{ distance: 2, color: "#ffeecc" }} parallax={{ scale: 0.05, method: "relief", steps: 8, layers: 12 }} depthBias={1} renderMethod="deferred" diffuseTransmission={0.2} />),
    );
    expect(() => html(<StandardMaterial parallax_mapping_method={"Relief" as "Occlusion"} />)).toThrow(/parallax_mapping_method/);
  });

  test("animate: Bevy's fields are the old material's keys; an emissive's LinearRgba keys its colour and intensity", () => {
    const blue = track({ 0: "#3060ff", 30: "#ff4070" }, "in-out-cubic");
    const E = LinearRgba.from(Srgba.hex("#ff4070"));
    expect(html(<StandardMaterial base_color="#3060ff" animate={{ base_color: blue, perceptual_roughness: track({ 0: 0.2, 30: 0.8 }), metallic: track({ 0: 0, 30: 1 }) }} />)).toBe(
      html(<MeshStandardMaterial color="#3060ff" roughness={0.5} metalness={0} animate={{ color: blue, roughness: track({ 0: 0.2, 30: 0.8 }), metalness: track({ 0: 0, 30: 1 }) }} />),
    );
    expect(html(<StandardMaterial emissive="#ff4070" animate={{ emissive: track({ 0: E.mul(0), 30: E.mul(2.5) }, "out-cubic") }} />)).toBe(
      html(<MeshStandardMaterial color="#ffffff" roughness={0.5} metalness={0} emissive="#ff4070" {...nits(1)} animate={{ emissive_nits: track({ 0: 0, 30: 2.5 }, "out-cubic") } as never} />),
    );
    const F = LinearRgba.from(Srgba.hex("#00ff00"));
    // a hex string, not an Srgba, once made NaN channels and a glow that drew nothing: it names the form that works
    expect(() => LinearRgba.from("#ff3fa4" as never)).toThrow(/LinearRgba.from\(Srgba.hex\("#ff3fa4"\)\)/);
    expect(() => html(<StandardMaterial animate={{ emissive: track({ 0: E, 30: F }) }} />)).toThrow(/one colour/);
    expect(() => html(<StandardMaterial animate={{ metallic: track({ 0: "#fff", 30: "#000" }) as never }} />)).toThrow(/numbers/);
    expect(() => html(<StandardMaterial animate={{ roughness: track({ 0: 0, 30: 1 }) } as never} />)).toThrow(/not roughness/);
    expect(html(<ExtendedMaterial base={<StandardMaterial base_color="#7bd389" animate={{ base_color: blue }} />} extension={<ToonExtension steps={2} animate={{ outline_width: track({ 0: 2, 59: 4 }, "linear") }} />} />)).toBe(
      html(<MeshToonMaterial color="#7bd389" steps={2} animate={{ color: blue, outlineWidth: track({ 0: 2, 59: 4 }, "linear") }} />),
    );
    expect(() => html(<ExtendedMaterial base={<StandardMaterial animate={{ metallic: track({ 0: 0, 30: 1 }) }} />} extension={<ToonExtension />} />)).toThrow(/takes base_color, emissive/);
  });

  test("animate: any other field that interpolates goes by its Bevy name, set on the part's own material (FC1c)", () => {
    const coat = track({ 0: 0, 30: 1 });
    const tint = track({ 0: "#ffffff", 30: "#ff0000" });
    const h = html(<StandardMaterial animate={{ clearcoat: coat, specular_tint: tint, metallic: coat }} />);
    const attr = (name: string) => JSON.parse(new RegExp(`${name}="([^"]*)"`).exec(h)![1].replace(/&quot;/g, '"'));
    // the engine samples them (runtime/react animate.rs) onto the props, which start at frame 0's value
    expect(attr("data-raea-anim").tracks.map((t: { prop: string }) => t.prop).sort()).toEqual(["field:clearcoat", "field:specular_tint", "metalness"]);
    expect(attr("data-props")["field:clearcoat"]).toBe(0);
    expect(() => html(<StandardMaterial animate={{ clearcoat: tint as never }} />)).toThrow(/clearcoat keys are numbers/);
    expect(() => html(<StandardMaterial animate={{ attenuation_color: coat as never }} />)).toThrow(/attenuation_color keys are colours/);
    expect(() => html(<StandardMaterial animate={{ unlit: coat } as never} />)).toThrow(/not unlit/);
    expect(() => html(<ExtendedMaterial base={<StandardMaterial animate={{ clearcoat: coat }} />} extension={<GridExtension cell={1} />} />)).toThrow(/an ExtendedMaterial takes/);
  });

  test("uv_transform: a scale is the old repeat, a turn or a move its uvTransform too", () => {
    expect(html(<StandardMaterial base_color_texture="a.jpg" uv_transform={Affine2.fromScale([2, 3])} />)).toBe(html(<MeshStandardMaterial color="#ffffff" roughness={0.5} metalness={0} map="a.jpg" repeat={[2, 3]} />));
    expect(html(<StandardMaterial uv_transform={Affine2.IDENTITY} />)).toBe(html(<StandardMaterial />));
    expect(html(<StandardMaterial uv_transform={Affine2.fromScaleAngleTranslation([2, 2], 0.5, [0.1, 0])} />)).toBe(html(<MeshStandardMaterial color="#ffffff" roughness={0.5} metalness={0} repeat={[2, 2]} uvTransform={{ offset: [0.1, 0], rotation: 0.5, scale: [2, 2] }} />));
    expect(() => html(<StandardMaterial uv_transform={Affine2.fromScale([0, 1])} />)).toThrow(/positive/);
  });

  test("see-through: alpha_mode Blend is the old transparent, the base colour's alpha its opacity", () => {
    const old = (o: Record<string, unknown>) => html(<MeshStandardMaterial color="#88ccff" roughness={0.5} metalness={0} {...o} />);
    expect(html(<StandardMaterial base_color={Srgba.hex("#88ccff").with_alpha(0.4)} alpha_mode="Blend" />)).toBe(old({ transparent: true, opacity: 0.4 }));
    expect(html(<StandardMaterial base_color="#88ccff" alpha_mode="Blend" />)).toBe(old({ transparent: true, opacity: 1 }));
    // Opaque ignores the alpha, as Bevy's does
    expect(html(<StandardMaterial base_color={Srgba.hex("#88ccff").with_alpha(0.4)} />)).toBe(old({}));
    expect(html(<StandardMaterial base_color={Srgba.hex("#88ccff").with_alpha(0.4)} alpha_mode="Blend" unlit />)).toBe(html(<MeshBasicMaterial color="#88ccff" transparent opacity={0.4} />));
    expect(html(<StandardMaterial base_color={new Srgba(1, 0.5, 0, 0.25)} alpha_mode="Blend" />)).toBe(html(<MeshStandardMaterial color="#ff8000" roughness={0.5} metalness={0} transparent opacity={0.25} />));
    // a Mask cuts by the alpha and AlphaToCoverage dithers by it, as Bevy's read the base colour's alpha too
    expect(html(<StandardMaterial base_color={Srgba.hex("#88ccff").with_alpha(0.4)} alpha_mode={{ Mask: 0.5 }} />)).toBe(old({ alphaMode: "mask", alphaCutoff: 0.5, opacity: 0.4 }));
    expect(html(<StandardMaterial base_color={Srgba.hex("#88ccff").with_alpha(0.4)} alpha_mode="AlphaToCoverage" />)).toBe(old({ alphaMode: "coverage", opacity: 0.4 }));
    expect(html(<StandardMaterial base_color={Srgba.hex("#88ccff").with_alpha(0.4)} alpha_mode="Add" />)).toBe(old({ alphaMode: "add", opacity: 0.4 }));
  });

  test("emissive brighter than white: LinearRgba, made from a colour, goes as that colour and its intensity exactly", () => {
    const glow = (e: string, k: number) => html(<MeshStandardMaterial color="#ffffff" roughness={0.5} metalness={0} emissive={e} {...nits(k)} />);
    expect(html(<StandardMaterial emissive={LinearRgba.from(Srgba.hex("#884422")).mul(3)} />)).toBe(glow("#884422", 3));
    expect(html(<StandardMaterial emissive={LinearRgba.from(Srgba.hex("#884422")).mul(2).mul(1.5)} />)).toBe(glow("#884422", 3));
    expect(html(<StandardMaterial emissive={LinearRgba.from(Srgba.hex("#ff8800"))} />)).toBe(html(<MeshStandardMaterial color="#ffffff" roughness={0.5} metalness={0} emissive="#ff8800" {...nits(1)} />));
    // raw floats: the hue at full strength times the brightest channel
    expect(html(<StandardMaterial emissive={LinearRgba.rgb(4, 2, 0)} />)).toBe(glow("#ffbc00", 4));
    expect(html(<StandardMaterial emissive={LinearRgba.rgb(0, 0, 0)} />)).toBe(html(<StandardMaterial />));
    const l = LinearRgba.from(Srgba.hex("#ff8800")).mul(2);
    expect([l.red, l.green, l.blue]).toEqual([2, expect.closeTo(2 * 0.2462, 4), 0]);
  });

  test("ExtendedMaterial: raea's GridExtension and PaintExtension on a StandardMaterial are the old grid and paint", () => {
    const grid = { cell: 0.5, major: 2, color: "#a9a49a", edge: "" };
    const paint = { breakup: 0.5, scale: 0.6, color: "#4d7230", amount: 0.5 };
    expect(html(<ExtendedMaterial base={<StandardMaterial base_color="#cfc8bb" perceptual_roughness={0.7} />} extension={<GridExtension {...grid} />} />)).toBe(
      html(<MeshStandardMaterial color="#cfc8bb" roughness={0.7} metalness={0} grid={grid} />),
    );
    expect(html(<ExtendedMaterial base={<StandardMaterial base_color="#5c8638" perceptual_roughness={1} />} extension={<PaintExtension {...paint} />} />)).toBe(
      html(<MeshStandardMaterial color="#5c8638" roughness={1} metalness={0} paint={paint} />),
    );
    // a second extension: an ExtendedMaterial as the base
    const both = <ExtendedMaterial base={<ExtendedMaterial base={<StandardMaterial base_color="#5c8638" />} extension={<GridExtension cell={1} />} />} extension={<PaintExtension scale={2} />} />;
    expect(html(both)).toBe(html(<MeshStandardMaterial color="#5c8638" roughness={0.5} metalness={0} paint={{ scale: 2 }} grid={{ cell: 1 }} />));
    expect(() => html(<ExtendedMaterial base={<StandardMaterial />} extension={<StandardMaterial />} />)).toThrow(/<GridExtension>, <PaintExtension>, <WearExtension> or <ShaderExtension>/);
    expect(() => html(<ExtendedMaterial base={<StandardMaterial unlit />} extension={<GridExtension cell={1} />} />)).toThrow(/unlit/);
    expect(() => html(<GridExtension cell={1} />)).toThrow(/goes in an ExtendedMaterial/);
  });

  test("ExtendedMaterial with raea's ToonExtension is the old MeshToonMaterial", () => {
    const toon = (b: Record<string, unknown>, t: Record<string, unknown>) => html(<ExtendedMaterial base={<StandardMaterial {...b} />} extension={<ToonExtension {...t} />} />);
    expect(toon({ base_color: "#ff8844" }, { steps: 3, rim: true, outline: { width: 4 } })).toBe(html(<MeshToonMaterial color="#ff8844" steps={3} rim outline={{ width: 4 }} />));
    expect(toon({ base_color: "#ff8844" }, { shadow_color: "#332244", outline_width: 3, rim_intensity: 0.5 })).toBe(
      html(<MeshToonMaterial color="#ff8844" shadowColor="#332244" rimIntensity={0.5} outlineWidth={3} />),
    );
    // no base_color: on a Model, each glTF material keeps its own
    expect(toon({ material: "Body" }, { steps: 2 })).toBe(html(<MeshToonMaterial steps={2} material="Body" />));
    expect(toon({ base_color: Srgba.hex("#88ccff").with_alpha(0.5), alpha_mode: "Blend", emissive: LinearRgba.from(Srgba.hex("#ff0000")).mul(2) }, {})).toBe(
      html(<MeshToonMaterial color="#88ccff" emissive="#ff0000" {...nits(2)} transparent opacity={0.5} />),
    );
    expect(() => toon({ perceptual_roughness: 0.3 }, {})).toThrow(/not perceptual_roughness/);
    expect(() => html(<ExtendedMaterial base={<ExtendedMaterial base={<StandardMaterial />} extension={<GridExtension cell={1} />} />} extension={<ToonExtension />} />)).toThrow(/takes no other extension/);
  });
});

describe("Bevy's lights, in Bevy's units", () => {
  const props = (h: string) => JSON.parse(h.match(/data-props="([^"]*)"/)![1].replace(/&quot;/g, '"'));

  test("the units go to the rows as written (docs/control.feature FC1f): lux, lumens and cd/m², no hidden scale", () => {
    expect(props(html(<DirectionalLight illuminance={light_consts.lux.DIRECT_SUNLIGHT} />)).lux).toBe(100_000);
    expect(props(html(<PointLight intensity={800} />)).lumens).toBe(800);
    expect(props(html(<SpotLight intensity={light_consts.lumens.VERY_LARGE_CINEMA_LIGHT} />)).lumens).toBe(1_000_000);
    expect(props(html(<RectLight intensity={900} />)).lumens).toBe(900);
    expect(props(html(<AmbientLight brightness={80} />)).nits).toBe(80);
    expect(() => html(<three.PointLight lumens={-1} />)).toThrow(/lumens must be 0 or more/);
  });

  test("an AmbientLight on a Camera3d is the camera's own (Bevy 0.20), beside it and marked so the scene's others give way", () => {
    const h = html(<Camera3d><AmbientLight brightness={200} color="#ffeedd" /></Camera3d>);
    const lights = [...h.matchAll(/data-3d="AmbientLight" data-props="([^"]*)"/g)].map((m) => JSON.parse(m[1].replace(/&quot;/g, '"')));
    expect(lights).toHaveLength(1);
    expect(lights[0].camera).toBe(true);
    expect(lights[0].nits).toBe(200);
    expect(props(html(<AmbientLight />)).camera).toBeUndefined();
    expect(() => html(<Camera3d><AmbientLight {...({ intensity: 1 } as object)} /></Camera3d>)).toThrow(/`intensity` is 3d.tsx/);
  });

  test("PointLight sits at its translation, with Bevy's defaults (1e6 lm, range 20, no shadows)", () => {
    expect(html(<PointLight transform={Transform.fromXyz(1, 3, 2)} />)).toBe(html(<three.PointLight position={[1, 3, 2]} lumens={1_000_000} distance={20} castShadow={false} />));
    expect(html(<PointLight shadow_maps_enabled soft_shadows_enabled radius={0.1} color="#ffaa66" />)).toBe(
      html(<three.PointLight position={[0, 0, 0]} lumens={1_000_000} distance={20} castShadow radius={0.1} softShadow={0.1} color="#ffaa66" />),
    );
  });

  test("SpotLight and DirectionalLight shine along their transform's -Z; the spot's angles are Bevy's", () => {
    const t = Transform.fromXyz(0, 4, 0).lookingAt([0, 0, 0], [0, 0, -1]);
    const spot = props(html(<SpotLight transform={t} outer_angle={0.6} inner_angle={0.3} />));
    expect(close(spot.target, [0, 3, 0], 1e-9)).toBe(true);
    expect(spot.angle).toBe(0.6);
    expect(spot.penumbra).toBeCloseTo(0.5, 12);
    const sun = props(html(<DirectionalLight transform={Transform.fromRotation(Quat.fromRotationX(-Math.PI / 2))} shadow_maps_enabled />));
    expect(close(sun.target.map((v: number, i: number) => v - sun.position[i]), [0, -1, 0], 1e-9)).toBe(true);
    expect(sun.lux).toBe(10_000);
    expect(sun.castShadow).toBe(true);
  });

  test("RectLight faces its -Z in lumens; Hidden lights nothing; 3d.tsx's props are refused", () => {
    expect(props(html(<RectLight intensity={9e5} width={2} />))).toMatchObject({ lumens: 9e5, width: 2, height: 1, distance: 20 });
    expect(html(<PointLight visibility="Hidden" />)).toBe("");
    expect(() => html(<PointLight {...({ position: [0, 1, 0] } as object)} />)).toThrow(/`position` is 3d.tsx's/);
    expect(() => html(<DirectionalLight {...({ intensity: 1 } as object)} />)).toThrow(/`intensity` is 3d.tsx's/);
  });
});

describe("one tree (docs/sdk.feature S3a)", () => {
  const panel = <div style={{ width: 320, height: 200, background: "#1d2a44" }}>HELLO</div>;
  test("a page element inside an entity is a page layer in the world: a flat Screen at it, 100 px to the metre", () => {
    const out = html(<Mesh3d name="crate"><Cuboid />{panel}</Mesh3d>);
    expect(html(<Mesh name="crate"><Cuboid /><Screen name="crate:0" crt={false} background="transparent" resolution={[320, 200]} size={[3.2]}>{panel}</Screen></Mesh>)).toBe(out);
    expect(out).toContain("HELLO");
    expect(html(<Entity name="sign">{panel}</Entity>)).toContain('data-raea-screen="sign:0"');
  });
  test("its size is the element's own, in px; a layer is world or, on a mesh, surface", () => {
    expect(() => html(<Mesh3d><div>no size</div></Mesh3d>)).toThrow(/needs its size in px/);
    // layer isn't a DOM attribute: given as page layers read it
    expect(() => html(<Mesh3d><div {...({ layer: "wall" } as object)} style={{ width: 10, height: 10 }} /></Mesh3d>)).toThrow(/layer is "world"/);
    expect(() => html(<Entity><div {...({ layer: "surface" } as object)} style={{ width: 10, height: 10 }} /></Entity>)).toThrow(/a surface needs a mesh/);
  });
  test('layer="surface" in a Mesh3d: the page on the mesh over its UVs, its spec on the mesh and its page part beside it', () => {
    const out = html(<Mesh3d name="crate"><Cuboid /><div {...({ layer: "surface" } as object)} style={{ width: 256, height: 128 }}>HELLO</div></Mesh3d>);
    expect(out).toContain("&quot;surface&quot;:{&quot;name&quot;:&quot;crate:surface&quot;,&quot;w&quot;:256,&quot;h&quot;:128,&quot;crt&quot;:false}");
    expect(out).toContain('data-raea-screen="crate:surface"');
    expect(out).toContain("HELLO");
    expect(() => html(<Mesh3d><div {...({ layer: "surface" } as object)} style={{ width: 8, height: 8 }} /><div {...({ layer: "surface" } as object)} style={{ width: 8, height: 8 }} /></Mesh3d>)).toThrow(/one page element per mesh/);
  });
});

describe("S2a's last Bevy names: Mesh2d with a primitive and a ColorMaterial, and raea's ToonMaterial", () => {
  test("a Mesh2d is a Shape2D placed as Bevy's 2D camera sees it: the frame's centre the origin, +Y up, turns counterclockwise", () => {
    expect(html(<Mesh2d transform={Transform.fromXyz(100, 50, 2)}><Circle radius={40} /><ColorMaterial color="#c84" /></Mesh2d>))
      .toBe(html(<Shape2D shape="circle" size={[80, 80]} color="#c84" x={260} y={70} z={2} rotation={0} scale={1} />));
    const turned = Transform.fromXyz(-10, 0, 0).withRotation(Quat.fromRotationZ(0.5)).withScale([2, 3, 1]);
    expect(html(<Mesh2d transform={turned}><Rectangle half_size={[20, 10]} /><ColorMaterial color="#fff" /></Mesh2d>))
      .toBe(html(<Shape2D shape="rect" size={[40, 20]} color="#fff" x={150} y={120} z={0} rotation={-0.5} scale={[2, 3]} />));
    expect(html(<Mesh2d><RegularPolygon sides={5} circumcircle={{ radius: 30 }} /></Mesh2d>))
      .toBe(html(<Shape2D shape="polygon" size={[60, 60]} sides={5} color="#ffffff" x={160} y={120} z={0} rotation={0} scale={1} />));
    expect(html(<Mesh2d><Ellipse /><ColorMaterial /></Mesh2d>)).toBe(html(<Shape2D shape="ellipse" size={[2, 1]} color="#ffffff" x={160} y={120} z={0} rotation={0} scale={1} />));
    expect(() => html(<Mesh2d><Circle /><Ellipse /></Mesh2d>)).toThrow(/one primitive/);
    expect(() => html(<Mesh2d><ColorMaterial /></Mesh2d>)).toThrow(/give it a primitive/);
    expect(() => html(<Circle />)).toThrow(/goes in a Mesh2d/);
  });
  test("a ColorMaterial's texture and AlphaMode2d reach the Shape2D; an Srgba's alpha is its opacity (mesh2d_alpha_mode)", () => {
    expect(html(<Mesh2d><Rectangle half_size={[128, 128]} /><ColorMaterial color={new Srgba(0, 0, 1, 0.7)} texture="branding/icon.png" alpha_mode="Blend" /></Mesh2d>))
      .toBe(html(<Shape2D shape="rect" size={[256, 256]} color="#0000FF" opacity={0.7} texture="branding/icon.png" alpha_mode="Blend" x={160} y={120} z={0} rotation={0} scale={1} />));
    expect(html(<Mesh2d><Rectangle /><ColorMaterial color={new LinearRgba(0.2, 0, 0, 1)} alpha_mode={{ Mask: 0.5 }} /></Mesh2d>))
      .toBe(html(<Shape2D shape="rect" size={[1, 1]} color="#7C0000" opacity={1} alpha_mode={{ Mask: 0.5 }} x={160} y={120} z={0} rotation={0} scale={1} />));
    expect(html(<Mesh2d opacity={0.5}><Rectangle /><ColorMaterial color={new Srgba(1, 1, 1, 0.5)} /></Mesh2d>)).toContain("&quot;opacity&quot;:0.25");
    expect(() => html(<Shape2D alpha_mode={"Mask" as never} />)).toThrow(/alpha_mode/);
    expect(() => html(<Shape2D texture="" />)).toThrow(/texture/);
  });
  test("a primitive's vertex_colors (LinearRgba or [r, g, b, a]) reach the Shape2D as linear arrays, one per vertex (mesh2d_vertex_color_texture)", () => {
    const four = [new LinearRgba(1, 0, 0, 1), [0, 1, 0, 1] as [number, number, number, number], new LinearRgba(0, 0, 1, 0.5), new LinearRgba(1, 1, 1, 1)];
    expect(html(<Mesh2d><Rectangle vertex_colors={four} /><ColorMaterial /></Mesh2d>))
      .toBe(html(<Shape2D shape="rect" size={[1, 1]} vertex_colors={[[1, 0, 0, 1], [0, 1, 0, 1], [0, 0, 1, 0.5], [1, 1, 1, 1]]} color="#ffffff" x={160} y={120} z={0} rotation={0} scale={1} />));
    expect(html(<Mesh2d><Rectangle /></Mesh2d>)).not.toContain("vertex_colors");
    expect(() => html(<Shape2D vertex_colors={[[1, 0, 0, 1]]} />)).toThrow(/rect's mesh has 4 vertices/);
    expect(() => html(<Shape2D shape="triangle" vertex_colors={four.slice(0, 2).map(() => [1, 1, 1, 1] as [number, number, number, number])} />)).toThrow(/triangle's mesh has 3/);
    expect(html(<Shape2D shape="circle" vertex_colors={[[1, 1, 1, 1]]} />)).toContain("vertex_colors");
    expect(() => html(<Shape2D vertex_colors={[[1, 0, 0]] as never} />)).toThrow(/one linear \[r, g, b, a\]/);
  });
  test("a ColorMaterial's uv_transform and a texture with its own sampler reach the Shape2D (mesh2d_repeated_texture)", () => {
    const tex = { path: "panel.png", sampler: { wrap: "repeat" as const } };
    const uv = { scale: [2, 3] as [number, number], angle: 0, translation: [0, 0] as [number, number] };
    expect(html(<Mesh2d><Rectangle /><ColorMaterial texture={tex} uv_transform={uv} /></Mesh2d>))
      .toBe(html(<Shape2D shape="rect" size={[1, 1]} color="#ffffff" texture={tex} uv_transform={uv} x={160} y={120} z={0} rotation={0} scale={1} />));
    expect(html(<Mesh2d><Rectangle /><ColorMaterial texture="panel.png" /></Mesh2d>)).not.toContain("uv_transform");
    expect(() => html(<Shape2D texture={{ path: "panel.png", sampler: { wrap: "tile" as never } }} />)).toThrow(/sampler.wrap/);
    expect(() => html(<Shape2D texture={{ path: "", sampler: {} }} />)).toThrow(/or \{ path, sampler \}/);
    expect(() => html(<Shape2D uv_transform={{ scale: [2], angle: 0, translation: [0, 0] } as never} />)).toThrow(/uv_transform is an Affine2/);
  });
  test("ToonMaterial is the StandardMaterial with ToonExtension", () => {
    expect(html(<Mesh3d><Cuboid /><ToonMaterial base_color="#c84" steps={3} rim /></Mesh3d>))
      .toBe(html(<Mesh3d><Cuboid /><ExtendedMaterial base={<StandardMaterial base_color="#c84" />} extension={<ToonExtension steps={3} rim />} /></Mesh3d>));
    expect(() => html(<ToonMaterial {...({ metallic: 1 } as object)} />)).toThrow(/not metallic/);
  });
});
