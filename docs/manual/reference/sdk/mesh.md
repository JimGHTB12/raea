---
title: "raea/mesh"
summary: "Shapes and solids: Mesh3d and Mesh2d, Bevy's primitives, buffer meshes, instancing and exact solids."
order: 66
sources: ["sdk/raea/src/mesh.ts"]
---

`<Mesh3d>` holds a shape and a material, as Bevy's Mesh3d and MeshMaterial3d do. The shapes are Bevy's primitives
(`Cuboid`, `Sphere`, `Cylinder`, `Capsule3d`, `Cone`, `Torus`, `Plane3d` and the rest) with Bevy's fields.
`<BufferMesh>` takes your own vertices; `<Instances>` and `scatter` draw many copies at once; `Solid` builds exact
solids with booleans, for parts that must fit together.

```tsx
import { Cuboid, Mesh3d } from "raea/mesh";
import { StandardMaterial } from "raea/pbr";
import { Transform } from "raea/transform";

const Box = () => (
  <Mesh3d transform={Transform.fromXyz(0, 0.5, 0)}>
    <Cuboid half_size={[0.5, 0.5, 0.5]} />
    <StandardMaterial base_color="#4a7ae8" />
  </Mesh3d>
);
```

<!-- generated:begin by tools/manual-ref/sdk.ts from the SDK's exports and doc comments; edit outside this block -->

```ts
import { ATTRIBUTE_COLOR, ATTRIBUTE_NORMAL, ATTRIBUTE_POSITION, ATTRIBUTE_TANGENT } from "raea/mesh"
```

`raea/mesh` exports 14 components, 1 function, 2 classes, 10 values, 24 types. Bevy's counterpart: `bevy_mesh, bevy_shape`. Source: [sdk/raea/src/mesh.ts](https://github.com/shinyobjectz/raea/blob/main/sdk/raea/src/mesh.ts).

## Components

### `BufferMesh`

A mesh from its own vertex buffers, as the geometry of a <Mesh> (Bevy's Mesh built from attributes): a terrain from
a height field, a procedural shape, a ribbon, an imported point list. Drawn as flat triangles, or with indices and
normals as Bevy's indexed Mesh.

Bevy: `bevy_mesh::mesh::Mesh`.

Takes [`BufferMeshProps`](3d.md#buffermeshprops)'s props.

Used in: [3d/vertex_colors](https://github.com/shinyobjectz/raea/blob/main/sdk/raea/bench/bevy-parity/3d/vertex_colors.tsx), [shader_advanced/compute_mesh](https://github.com/shinyobjectz/raea/blob/main/sdk/raea/bench/bevy-parity/shader_advanced/compute_mesh.tsx).

### `Capsule`

A capsule (Bevy's Capsule3d, three.js's CapsuleGeometry): [radius, length of the straight part], standing up
(along y); its whole height is length + 2 x radius. A character's stand-in, a pill, a collider's look.

Also takes the props of [`CapsuleProps`](3d.md#capsuleprops) (`args`).

### `Capsule3d`

*From Bevy's `bevy_shape::Capsule3d`:* A 3D capsule primitive centered on the origin A three-dimensional capsule is defined as a surface at a distance (radius) from a line

Also takes the props of [`Capsule3dProps`](3d.md#capsule3dprops) (`radius`, `half_length`, `args`).

### `Cone`

A cone: [radius, height], its point up (along y).

Bevy: `bevy_shape::Cone`.

Also takes the props of [`ConeProps`](3d.md#coneprops) (`args`, `radius`, `height`).

### `ConicalFrustum`

*From Bevy's `bevy_shape::ConicalFrustum`:* A conical frustum primitive. A conical frustum can be created by slicing off a section of a cone.

Also takes the props of [`ConicalFrustumProps`](#conicalfrustumprops) (`radius_top`, `radius_bottom`, `height`, `args`).

### `Cuboid`

Bevy's Cuboid: `<Cuboid half_size={[1, 0.5, 1]} />` is a Box of args [2, 1, 2].

Bevy: `bevy_shape::Cuboid`.

| Prop | Type | What it does |
| --- | --- | --- |
| `args` | `[number, number, number]` | – |

Also takes the props of [`CuboidProps`](3d.md#cuboidprops) (`half_size`).

Used in: [ui/render_ui_to_texture](https://github.com/shinyobjectz/raea/blob/main/sdk/raea/bench/bevy-parity/ui/render_ui_to_texture.tsx), [ui/widgets/viewport_node](https://github.com/shinyobjectz/raea/blob/main/sdk/raea/bench/bevy-parity/ui/widgets/viewport_node.tsx), [3d/fog](https://github.com/shinyobjectz/raea/blob/main/sdk/raea/bench/bevy-parity/3d/fog.tsx), [3d/transparency_3d](https://github.com/shinyobjectz/raea/blob/main/sdk/raea/bench/bevy-parity/3d/transparency_3d.tsx), [3d/animated_material](https://github.com/shinyobjectz/raea/blob/main/sdk/raea/bench/bevy-parity/3d/animated_material.tsx), [3d/two_passes](https://github.com/shinyobjectz/raea/blob/main/sdk/raea/bench/bevy-parity/3d/two_passes.tsx), [3d/spotlight](https://github.com/shinyobjectz/raea/blob/main/sdk/raea/bench/bevy-parity/3d/spotlight.tsx), [3d/orthographic](https://github.com/shinyobjectz/raea/blob/main/sdk/raea/bench/bevy-parity/3d/orthographic.tsx), and 34 more.

### `Cylinder`

[radiusTop, radiusBottom, height], standing up (along y) as three.js has it.

Bevy: `bevy_shape::Cylinder`.

Also takes the props of [`CylinderProps`](3d.md#cylinderprops) (`args`, `radius`, `half_height`).

Used in: [shader/shader_material_bindless](https://github.com/shinyobjectz/raea/blob/main/sdk/raea/bench/bevy-parity/shader/shader_material_bindless.tsx).

### `Instances`

One glTF (or one mesh) drawn at many placements, loaded once and drawn by the GPU as instances: a meadow's grass
and flowers, a forest, a suburb's houses. `<Instances src={staticFile("daisy.glb")} at={scatter({...})} />`. The
copies share one surface and can't play a clip.

Bevy: `bevy_pbr::render::mesh::MeshPipeline (automatic batching of entities sharing a mesh and material)`.

Takes [`InstancesProps`](3d.md#instancesprops)'s props.

### `Mesh2d`

Bevy's Mesh2d with its Transform, a primitive and a ColorMaterial: <Shape2D> in Bevy's 2D world.

Takes [`Mesh2dProps`](#mesh2dprops)'s props.

### `Mesh3d`

Bevy's Mesh3d on an entity with its Transform: a shape (Cuboid, Sphere...) and a material inside, and any page
element inside as a page layer in the world at it, or on it with layer="surface" (`meshLayers`).

Bevy: `bevy_mesh::components::Mesh3d`, `bevy_pbr::mesh_material::MeshMaterial3d<bevy_pbr::pbr_material::StandardMaterial>
castShadow={false} and receiveShadow={false} put Bevy's markers on it:`, `bevy_light::NotShadowCaster`, `bevy_light::NotShadowReceiver
`layers` is its RenderLayers (a light's and a camera's too):`, `bevy_camera::visibility::render_layers::RenderLayers
shadow="only" draws it with the shadow-only material (engine/render shadow_only.rs):`, `bevy_pbr::mesh_material::MeshMaterial3d<raea_render::engine::shadow_only::ShadowOnlyMaterial>`.

| Prop | Type | What it does |
| --- | --- | --- |
| `transform` | `TransformValue` | – |
| `visibility` | `"Visible" \| "Hidden" \| "Inherited"` | Bevy's Visibility: "Hidden" draws nothing; "Visible" and "Inherited" draw |
| `children` | `ReactNode` | – |

Also takes the props of `MeshProps` (`animate`, `name`, `pass`, `quaternion`, `allowOverlap`, `posed`, `shape`, `driven`, `shadow`, `material`, `castShadow`, `surface`, `receiveShadow`, `attributes`, `compressed`, `layers`); `PointerHandlers3d` (`onClick`, `onDrag`, `onDragEnd`, `onDragEnter`, `onDragLeave`, `onDragOver`, `onDragStart`, `onDrop`, `onPointerDown`, `onPointerMove`, `onPointerUp`, `onPointerEnter`, `onPointerLeave`, `onPointerOver`, `onPointerOut`, `onWheel`).

Used in: [ui/render_ui_to_texture](https://github.com/shinyobjectz/raea/blob/main/sdk/raea/bench/bevy-parity/ui/render_ui_to_texture.tsx), [ui/widgets/viewport_node](https://github.com/shinyobjectz/raea/blob/main/sdk/raea/bench/bevy-parity/ui/widgets/viewport_node.tsx), [3d/fog](https://github.com/shinyobjectz/raea/blob/main/sdk/raea/bench/bevy-parity/3d/fog.tsx), [3d/vertex_colors](https://github.com/shinyobjectz/raea/blob/main/sdk/raea/bench/bevy-parity/3d/vertex_colors.tsx), [3d/rotate_environment_map](https://github.com/shinyobjectz/raea/blob/main/sdk/raea/bench/bevy-parity/3d/rotate_environment_map.tsx), [3d/transparency_3d](https://github.com/shinyobjectz/raea/blob/main/sdk/raea/bench/bevy-parity/3d/transparency_3d.tsx), [3d/spherical_area_lights](https://github.com/shinyobjectz/raea/blob/main/sdk/raea/bench/bevy-parity/3d/spherical_area_lights.tsx), [3d/animated_material](https://github.com/shinyobjectz/raea/blob/main/sdk/raea/bench/bevy-parity/3d/animated_material.tsx), and 54 more.

### `MeshSequence`

A baked simulation, one mesh a frame.

Takes [`MeshSequenceProps`](3d.md#meshsequenceprops)'s props.

### `Plane3d`

*From Bevy's `bevy_shape::Plane3d`:* A bounded plane in 3D space. It forms a surface starting from the origin with a defined height and width.

Takes [`Plane3dProps`](3d.md#plane3dprops)'s props.

Used in: [3d/vertex_colors](https://github.com/shinyobjectz/raea/blob/main/sdk/raea/bench/bevy-parity/3d/vertex_colors.tsx), [3d/transparency_3d](https://github.com/shinyobjectz/raea/blob/main/sdk/raea/bench/bevy-parity/3d/transparency_3d.tsx), [3d/spherical_area_lights](https://github.com/shinyobjectz/raea/blob/main/sdk/raea/bench/bevy-parity/3d/spherical_area_lights.tsx), [3d/texture](https://github.com/shinyobjectz/raea/blob/main/sdk/raea/bench/bevy-parity/3d/texture.tsx), [3d/shadow_caster_receiver](https://github.com/shinyobjectz/raea/blob/main/sdk/raea/bench/bevy-parity/3d/shadow_caster_receiver.tsx), [3d/split_screen](https://github.com/shinyobjectz/raea/blob/main/sdk/raea/bench/bevy-parity/3d/split_screen.tsx), [3d/two_passes](https://github.com/shinyobjectz/raea/blob/main/sdk/raea/bench/bevy-parity/3d/two_passes.tsx), [3d/spotlight](https://github.com/shinyobjectz/raea/blob/main/sdk/raea/bench/bevy-parity/3d/spotlight.tsx), and 16 more.

### `Sphere`

*From Bevy's `bevy_shape::Sphere`:* A sphere primitive, representing the set of all points some distance from the origin

Also takes the props of [`SphereProps`](3d.md#sphereprops) (`args`, `radius`, `ico`, `uv`).

Used in: [3d/fog](https://github.com/shinyobjectz/raea/blob/main/sdk/raea/bench/bevy-parity/3d/fog.tsx), [3d/rotate_environment_map](https://github.com/shinyobjectz/raea/blob/main/sdk/raea/bench/bevy-parity/3d/rotate_environment_map.tsx), [3d/transparency_3d](https://github.com/shinyobjectz/raea/blob/main/sdk/raea/bench/bevy-parity/3d/transparency_3d.tsx), [3d/spherical_area_lights](https://github.com/shinyobjectz/raea/blob/main/sdk/raea/bench/bevy-parity/3d/spherical_area_lights.tsx), [3d/shadow_caster_receiver](https://github.com/shinyobjectz/raea/blob/main/sdk/raea/bench/bevy-parity/3d/shadow_caster_receiver.tsx), [3d/pbr](https://github.com/shinyobjectz/raea/blob/main/sdk/raea/bench/bevy-parity/3d/pbr.tsx), [3d/spotlight](https://github.com/shinyobjectz/raea/blob/main/sdk/raea/bench/bevy-parity/3d/spotlight.tsx), [3d/rect_light](https://github.com/shinyobjectz/raea/blob/main/sdk/raea/bench/bevy-parity/3d/rect_light.tsx), and 7 more.

### `Torus`

*From Bevy's `bevy_shape::Torus`:* A torus primitive, often representing a ring or donut shape The set of points some distance from a circle centered at the origin

Also takes the props of [`TorusProps`](3d.md#torusprops) (`args`, `major_radius`, `minor_radius`).

## Functions

### `scatter`

Points scattered over an area for an <Instances>: seeded (the same layout every frame and every run), at least
`spacing` apart, clear of `avoid` and `clearOf`, on the ground `y` gives, kept by `density`. A spatial hash keeps
thousands quick. Call it once at module level, not per frame.

```ts
function scatter(o: ScatterOptions): Instance[]
```

## Classes

### `CrossSection`

A 2D shape (Manifold's CrossSection), held by value: its contours (counter-clockwise solid, clockwise holes),
area and bounds. Offset, combine and move it, then extrude or revolve it into a Solid.

| Member | What it does |
| --- | --- |
| `new CrossSection(a: SectionAnswer)` | – |
| `CrossSection.from(a: SectionAnswer): CrossSection` | From an engine answer (a slice, a projection, an operation). |
| `CrossSection.of(polygons: Polygons): CrossSection` | Contours, overlaps merged and self-crossings resolved (Manifold's positive fill rule). |
| `.polygons: Polygons` | – |
| `.area: number` | – |
| `.contours: number` | – |
| `.bounds: { min: [number, number]; max: [number, number]; }` | – |
| `.union(...others: CrossSection[]): CrossSection` | This and the others together. |
| `.minus(...others: CrossSection[]): CrossSection` | This with the others cut away. |
| `.intersect(...others: CrossSection[]): CrossSection` | Only where this and every other overlap. |
| `.hull(...others: CrossSection[]): CrossSection` | The convex hull of this and the others. |
| `.offset(delta: number, o?: { join?: JoinType \| undefined; miterLimit?: number \| undefined; segments?: number \| undefined; }): CrossSection` | Grown outward by `delta` (shrunk when negative), corners joined round (default), square, miter or bevel. |
| `.minkowski(other: CrossSection): CrossSection` | Swept by another shape (the 2D Minkowski sum). |
| `.simplify(epsilon?: number): CrossSection` | Fewer points: those within `epsilon` of a straight line dropped. |
| `.move(x: number, y: number): CrossSection` | Moved by (x, y). |
| `.rotate(degrees: number): CrossSection` | Turned by `degrees` about the origin. |
| `.scale(x: number, y?: number): CrossSection` | Scaled uniformly, or per axis. |
| `.mirror(nx: number, ny: number): CrossSection` | Mirrored across the line through the origin with this normal. |
| `.warp(fn: (x: number, y: number) => [number, number]): CrossSection` | Every point moved by `fn`, then re-unioned (crossings resolved). |
| `.decompose(): CrossSection[]` | Its separate pieces (each outline with its holes). |
| `.isEmpty(): boolean` | Whether nothing is left. |
| `.extrude(height: number, o?: { divisions?: number \| undefined; twist?: number \| undefined; scaleTop?: [number, number] \| undefined; }): Solid` | Extruded along z into a Solid (see manifold.extrude). |
| `.revolve(o?: { degrees?: number \| undefined; segments?: number \| undefined; }): Solid` | Revolved about the z axis into a Solid (see manifold.revolve). |

### `Solid`

A handle to a native solid. Free it when done with many (or let the run end).

| Member | What it does |
| --- | --- |
| `new Solid(id: number, expr: string)` | – |
| `.id: number` | – |
| `.expr: string` | the expression that rebuilds this solid (what a mesh row draws) |
| `.union(...others: Solid[]): Solid` | This and the others together. |
| `.minus(...others: Solid[]): Solid` | This with the others cut away. |
| `.intersect(...others: Solid[]): Solid` | Only where this and every other overlap. |
| `.hull(...others: Solid[]): Solid` | The convex hull of this and the others. |
| `.move(x: number, y: number, z: number): Solid` | Moved by (x, y, z). |
| `.rotate(x: number, y: number, z: number): Solid` | Turned by x, y and z degrees. |
| `.scale(s: number, y?: number \| undefined, z?: number \| undefined): Solid` | Scaled uniformly, or per axis. |
| `.volume(): number` | Its volume. |
| `.bounds(): Bounds` | Its bounding box. |
| `.triangles(): number` | How many triangles its mesh has. |
| `.isEmpty(): boolean` | Whether nothing is left (a boolean that removed everything). |
| `.slice(z: number): Section` | Its cross-section at height z (in a space-time mask, z is the frame). |
| `.mirror(nx: number, ny: number, nz: number): Solid` | Mirrored across the plane through the origin with this normal. |
| `.transform(m: number[]): Solid` | Transformed by a 3x4 matrix given column by column (x axis, y axis, z axis, translation). |
| `.trim(nx: number, ny: number, nz: number, offset?: number): Solid` | What's on the normal's side of the plane n. p = offset. |
| `.splitByPlane(nx: number, ny: number, nz: number, offset?: number): [Solid, Solid]` | Cut in two by the plane n. p = offset: [the normal's side, the other side]. |
| `.split(cutter: Solid): [Solid, Solid]` | Cut in two by another solid: [inside it, outside it]. |
| `.refine(n: number): Solid` | Each triangle cut into n x n. |
| `.smooth(o?: { angle?: number \| undefined; smoothness?: number \| undefined; refine?: number \| undefined; }): Solid` | Edges sharper than `angle` degrees kept, the rest rounded (smoothness 0..1 for the sharp ones too), then refined. |
| `.minkowski(other: Solid): Solid` | This swept by another solid (the Minkowski sum): rounded by a sphere, thickened by a box. |
| `.minkowskiDiff(other: Solid): Solid` | This eroded by another solid (the Minkowski difference). |
| `.mesh(): { verts: Float32Array<ArrayBufferLike>; tris: Uint32Array<ArrayBufferLike>; }` | Its triangles: positions (x y z) and indices by threes. |
| `.warp(fn: (x: number, y: number, z: number) => [number, number, number]): Solid` | Every vertex moved by `fn` (the topology kept): a bend, a twist, a wave. |
| `.simplify(tolerance: number): Solid` | The mesh coarsened: vertices closer than `tolerance` merged where the shape allows. |
| `.refineToLength(length: number): Solid` | Refined until no edge is longer than `length`. |
| `.refineToTolerance(tolerance: number): Solid` | Refined until its smoothed surface is within `tolerance` of the true curve. |
| `.crossSection(z: number): CrossSection` | Its cross-section at height z as a CrossSection (to offset, combine or extrude again). |
| `.project(): CrossSection` | Its silhouette seen from above (+z), on the x-y plane. |
| `.decompose(): Solid[]` | Its disconnected pieces, each a Solid. |
| `.minGap(other: Solid, searchLength: number): number` | The smallest gap between this and another, looking no further than `searchLength` (which it answers when nothing is closer). |
| `.rayCast(from: Vec3, to: Vec3): { distance: number; position: Vec3; normal: Vec3; }[]` | Where the segment from `from` to `to` crosses the surface, nearest first (distance 0..1 along it). |
| `.area(): number` | Its surface area. |
| `.genus(): number` | Its genus (how many holes go through it: a torus has 1). |
| `.export(path: string, format?: "obj" \| "glb" \| undefined): number` | Written as OBJ or glTF binary (by the extension, or `format`); the file's size. |
| `.free(): void` | Let the engine drop it. |

## Values

### `ATTRIBUTE_COLOR`

Bevy's Mesh::ATTRIBUTE_COLOR: a vertex's linear RGBA colour (id 5, Float32x4)

### `ATTRIBUTE_NORMAL`

Bevy's Mesh::ATTRIBUTE_NORMAL: a vertex's normal (id 1, Float32x3)

### `ATTRIBUTE_POSITION`

Bevy's Mesh::ATTRIBUTE_* (ids 0 to 5), as a Material's vertex buffer names them.

Used in: [shader_advanced/custom_vertex_attribute](https://github.com/shinyobjectz/raea/blob/main/sdk/raea/bench/bevy-parity/shader_advanced/custom_vertex_attribute.tsx).

### `ATTRIBUTE_TANGENT`

Bevy's Mesh::ATTRIBUTE_TANGENT: a vertex's tangent, its w the bitangent's sign (id 4, Float32x4)

### `ATTRIBUTE_UV_0`

Bevy's Mesh::ATTRIBUTE_UV_0: a vertex's first texture coordinates (id 2, Float32x2)

### `ATTRIBUTE_UV_1`

Bevy's Mesh::ATTRIBUTE_UV_1: a vertex's second texture coordinates, as a lightmap's (id 3, Float32x2)

### `crossSection`

2D shapes as CrossSections, made natively.

### `manifold`

### `MeshVertexAttribute`

A vertex attribute: its name, its unique id (a high random one for your own, as Bevy asks, so it sorts after the
built-in ones) and its format. MeshVertexAttribute.new("BlendColor", 988540917, "Float32x4") is Bevy's
MeshVertexAttribute::new; Mesh's own are ATTRIBUTE_POSITION and the rest.

Bevy: `bevy_mesh::MeshVertexAttribute`.

Used in: [shader_advanced/custom_vertex_attribute](https://github.com/shinyobjectz/raea/blob/main/sdk/raea/bench/bevy-parity/shader_advanced/custom_vertex_attribute.tsx).

### `outline`

2D outlines for extrude and revolve.

## Types

### `Animate3d`

A 3D node's declared animation: tracks by property, or a move.

```ts
type Animate3d = Record<string, TrackSpec | TrackSpec[] | Keys> | Move
```

### `Bounds`

| Field | Type | What it is |
| --- | --- | --- |
| `min` (required) | `Vec3` | – |
| `max` (required) | `Vec3` | – |

### `BufferMeshProps`

| Field | Type | What it is |
| --- | --- | --- |
| `positions` (required) | `number[]` | x, y, z of each vertex |
| `indices` | `number[]` | three vertex indices a triangle (counter-clockwise seen from its front); without them each three positions are one. With normals too it is Bevy's indexed Mesh, its index buffer kept, as a compute that writes a mesh expects |
| `normals` | `number[]` | x, y, z of each vertex's normal (without them each face is flat, facing its front) |
| `uvs` | `number[]` | u, v of each vertex (without them a picture map is projected onto the mesh's bounds) |
| `colors` | `number[] \| string[]` | each vertex's colour: r, g, b, a (linear, 0 to 1) per vertex, or one "#rrggbb" per vertex; the material's colour is multiplied by it (Bevy's ATTRIBUTE_COLOR), so leave the material white to see them as given |
| `attributes` | `Record<string, { size: number; values: number[] }>` | a mesh's own attributes beyond those, for a three.js shader that reads them (<ThreeShaderMaterial>): each one's size (1 to 4 numbers a vertex) and values, per vertex |

### `Capsule3dProps`

Bevy's Capsule3d: radius and half_length, the straight part's half (0.5 each by Bevy's default), standing up.

| Field | Type | What it is |
| --- | --- | --- |
| `radius` | `number` | – |
| `half_length` | `number` | – |
| `args` | `[number, number]` | – |

### `CapsuleProps`

| Field | Type | What it is |
| --- | --- | --- |
| `args` | `[number, number]` | – |

### `ConeProps`

| Field | Type | What it is |
| --- | --- | --- |
| `args` | `[number, number]` | – |
| `radius` | `number` | – |
| `height` | `number` | – |

### `ConicalFrustumProps`

Bevy's ConicalFrustum: a cylinder whose ends differ, radius_top and radius_bottom (Bevy's 0.25 and 0.5) and its
height (0.5), standing up: a tapered Cylinder of args [radius_top, radius_bottom, height].

| Field | Type | What it is |
| --- | --- | --- |
| `radius_top` | `number` | – |
| `radius_bottom` | `number` | – |
| `height` | `number` | – |
| `args` | `[number, number, number]` | – |

### `CuboidProps`

Bevy's Cuboid: half its size along each axis ([0.5, 0.5, 0.5], a 1 m cube, as Bevy's default).

| Field | Type | What it is |
| --- | --- | --- |
| `half_size` | `[number, number, number]` | – |

### `CylinderProps`

`args`, or Bevy's Cylinder fields: radius and half_height (0.5 each by Bevy's default).

| Field | Type | What it is |
| --- | --- | --- |
| `args` | `[number, number, number]` | – |
| `radius` | `number` | – |
| `half_height` | `number` | – |

### `Instance`

One copy's placement in an <Instances>, in the Instances' own space (its position, rotation and scale move every
copy together). Rotation in radians, XYZ order, as everywhere.

| Field | Type | What it is |
| --- | --- | --- |
| `position` (required) | `Vec3` | – |
| `rotation` | `Vec3` | – |
| `scale` | `number \| Vec3` | – |

### `InstancesProps`

| Field | Type | What it is |
| --- | --- | --- |
| `src` | `string` | a glTF file drawn at every placement: staticFile("models/house.glb"). Without it, a geometry child (or `shape`) and a material child draw a mesh at every placement |
| `at` (required) | `Instance[]` | where each copy stands. Fixed for the shot: the first frame's list is drawn on every frame (move, turn, scale, hide or recolour the whole set by the Instances' own props) |
| `name` | `string` | – |
| `position` | `Vec3` | – |
| `rotation` | `Vec3` | radians |
| `scale` | `number \| Vec3` | – |
| `scene` | `number` | of a glTF: which of its scenes (0 by default) |
| `shape` | `string \| Solid` | a Manifold solid or its expression for the mesh form; wins over the geometry child |
| `shadow` | `boolean` | false: the copies cast no shadow (true by default). Grass and flowers by the thousand usually want false |
| `receiveShadow` | `boolean` | false: no shadow falls on the copies |
| `pass` | `string` | – |
| `children` | `ReactNode` | the mesh form: a geometry and a material (MeshStandardMaterial, MeshPhysicalMaterial or MeshBasicMaterial). The glTF form: MeshStandardMaterial children re-covering its materials, as on a Model |

### `Mesh2dProps`

| Field | Type | What it is |
| --- | --- | --- |
| `transform` | `TransformValue` | – |
| `visibility` | `"Inherited" \| "Visible" \| "Hidden"` | – |
| `children` | `ReactNode` | one of bevy_math's 2D primitives (<Circle>, <Capsule2d>,...) and a <ColorMaterial>, or your own <Material> |
| `name` | `string` | – |
| `onClick` | `(e: Event3d) => void` | – |
| `onDrag` | `(e: Event3d) => void` | – |
| `onDragEnd` | `(e: Event3d) => void` | – |
| `onDragEnter` | `(e: Event3d) => void` | – |
| `onDragLeave` | `(e: Event3d) => void` | – |
| `onDragOver` | `(e: Event3d) => void` | – |
| `onDragStart` | `(e: Event3d) => void` | – |
| `onDrop` | `(e: Event3d) => void` | – |
| `onPointerDown` | `(e: Event3d) => void` | – |
| `onPointerMove` | `(e: Event3d) => void` | – |
| `onPointerUp` | `(e: Event3d) => void` | – |
| `onPointerEnter` | `(e: Event3d) => void` | – |
| `onPointerLeave` | `(e: Event3d) => void` | – |
| `onPointerOver` | `(e: Event3d) => void` | – |
| `onPointerOut` | `(e: Event3d) => void` | – |
| `onWheel` | `(e: Event3d) => void` | – |
| `quat` | `Quat` | Bevy's rotation as it is, a quaternion [x, y, z, w] in Bevy's axes (y up, z toward the viewer), in place of `rotation`: a turn out of the frame's plane too, as a Sprite's Transform takes it (camera/2d_on_ui) |
| `opacity` | `number` | 0 to 1 (1 by default) |

### `Mesh3dProps`

The props of [`Mesh3d`](#mesh3d), tabled there.

### `MeshAttribute`

One attribute's values on a mesh: one a vertex, each a number or a list of the format's size.

```ts
type MeshAttribute = [MeshVertexAttribute, (number | number[])[]]
```

### `MeshSequenceProps`

| Field | Type | What it is |
| --- | --- | --- |
| `name` | `string` | – |
| `src` (required) | `string` | a.rseq (tools/goo writes them; raea_render::meshseq) |
| `frame` (required) | `number` | which of its frames (clamped) |
| `position` | `Vec3` | – |
| `rotation` | `Vec3` | – |
| `scale` | `number` | – |
| `color` | `string` | multiplies the vertex colours |
| `roughness` | `number` | – |
| `metalness` | `number` | – |
| `clearcoat` | `number` | – |
| `clearcoatRoughness` | `number` | – |
| `reflectance` | `number` | 0..1 specular strength at normal incidence (0.5: 4%, a dielectric's) |
| `subsurface` | `number` | 0..1 light let through (diffuse transmission): a jelly's glow |
| `thickness` | `number` | – |
| `transmission` | `number` | 0..1 light let straight through and bent (a clear liquid: water, tea, juice), with its index of refraction (1.33 water) and, for a tinted one, the colour it takes after `attenuationDistance` metres through it. Screen space: it refracts what's on screen behind it, and one clear layer doesn't show another through it |
| `ior` | `number` | – |
| `attenuationColor` | `string` | – |
| `attenuationDistance` | `number` | – |
| `opacity` | `number` | – |
| `map` | `string` | a picture over its texture coordinates |
| `doubleSided` | `boolean` | – |

### `PlanBox`

A box on the plan (x, z, metres): the room SDK's Footprint, or any { min, max }.

| Field | Type | What it is |
| --- | --- | --- |
| `min` (required) | `[number, number]` | – |
| `max` (required) | `[number, number]` | – |

### `Plane3dProps`

Bevy's Plane3d: a flat rectangle facing `normal` (Bevy's default +Y, a floor), `half_size` its half extent along
its two axes ([0.5, 0.5]), laid out as Bevy's PlaneMeshBuilder does: facing +Z it is a Plane of args
[2 x half_size]. The normal is along an axis; a tilted plane is a rotated Mesh3d.

| Field | Type | What it is |
| --- | --- | --- |
| `normal` | `Vec3` | – |
| `half_size` | `[number, number]` | – |

### `Polygons`

A 2D outline: contours of [x, y] points (counter-clockwise for solid, clockwise for a hole).

```ts
type Polygons = [number, number][][]
```

### `ScatterOptions`

| Field | Type | What it is |
| --- | --- | --- |
| `area` (required) | `PlanBox` | where the points fall: a box on the plan (x, z) |
| `count` (required) | `number` | how many (fewer when the area fills up: spacing and avoid leave no room) |
| `spacing` | `number` | the least distance between two points, metres (0: none) |
| `avoid` | `PlanBox[]` | boxes on the plan to keep clear of, grown by `margin` |
| `margin` | `number` | how far to stay off each avoided box (spacing / 2 by default) |
| `clearOf` | `Instance[]` | points kept off already-placed ones too: a second scatter (flowers) clear of the first (stones) by `spacing` |
| `seed` | `number` | 1 by default; the same seed gives the same points, every frame, render and play |
| `yaw` | `boolean \| number \| [number, number]` | each point's turn about y (radians): true (the default) any, false none, a number that one, [lo, hi] between |
| `tilt` | `number` | a lean off upright, radians: each point tipped by up to this much in a random direction (0 by default) |
| `scale` | `number \| [number, number]` | a scale, or [lo, hi] between (uniform) |
| `y` | `number \| ((x: number, z: number) => number)` | the ground's height: a number (0), or the terrain's height at (x, z) |
| `density` | `(x: number, z: number) => number` | how likely a point is kept at (x, z), 0 to 1: a density map (a path worn bare, a patch thicker by the chair) |

### `Section`

A cross-section of a solid at a height: its area, how many contours, and the contours themselves.

| Field | Type | What it is |
| --- | --- | --- |
| `area` (required) | `number` | – |
| `contours` (required) | `number` | – |
| `polygons` (required) | `[number, number][][]` | – |

### `SphereProps`

`args`, or Bevy's Sphere field: radius (0.5 by Bevy's default). Its mesh is Bevy's Sphere::mesh(): an icosphere of 5
subdivisions, or `ico` of them (Bevy's `.ico(n)`, 0 to 80), or a UV sphere of `uv` = [sectors, stacks] (`.uv(s, t)`).

| Field | Type | What it is |
| --- | --- | --- |
| `args` | `[number]` | – |
| `radius` | `number` | – |
| `ico` | `number` | – |
| `uv` | `[number, number]` | – |

### `TorusProps`

`args`, or Bevy's Torus fields: major_radius (0.75) and minor_radius (0.25), the tube's.

| Field | Type | What it is |
| --- | --- | --- |
| `args` | `[number, number]` | – |
| `major_radius` | `number` | – |
| `minor_radius` | `number` | – |

### `Vec3`

```ts
type Vec3 = [number, number, number]
```

### `VertexFormat`

A vertex attribute's format, as Bevy's VertexFormat names it.

```ts
type VertexFormat = "Float32" | "Float32x2" | "Float32x3" | "Float32x4" | "Uint32" | "Uint32x2" | "Uint32x3" | "Uint32x4" | "Sint32" | "Sint32x4"
```

<!-- generated:end -->
