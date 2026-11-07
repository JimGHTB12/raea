# cloth.py: a cloth simulated over the things it falls on and baked into a clip raea plays: a rag over a bench edge, a
# tarp over a crate, a towel on a rail.
#   raea blender run blender/cloth.py IN.glb OUT.glb (--sheet W,D,X,Y,Z | --cloth NAME) [--pin X0,Y0,Z0,X1,Y1,Z1]
#       [--frames 60] [--every 3] [--cuts 24] [--mass 0.15] [--stiff 5] [--bend 0.05] [--crumple 0] [--no-floor]
# IN holds what the cloth lands on (every mesh in it collides; a solid from `raea solid export` is fine). The cloth is
# a W x D metre sheet made level at X,Y,Z (raea's frame, metres, y up), or the mesh NAME in IN. --pin holds the
# vertices inside that box still (a cloth hung from a nail). A floor at y 0 collides too, unless --no-floor. OUT holds the cloth alone: its flat start, a morph target
# every --every frames and the clip "Drape" keyed through them (<Model animation="Drape" time={t} loop={false}>; past
# its end it holds the settled cloth), and OUT.settled.glb holds the last frame as a plain mesh.
import bpy, bmesh, sys, os, json
from mathutils import Vector

NAME = "cloth.py"


def fail(why):
    print(f"FAILED  {NAME}: {why}", file=sys.stderr)
    sys.exit(1)


def argv():
    return sys.argv[sys.argv.index("--") + 1:] if "--" in sys.argv else []


def opt(a, k, d=None):
    return a[a.index(k) + 1] if k in a and a.index(k) + 1 < len(a) else d


def sidecar(src):
    for c in (src + ".json", os.path.splitext(src)[0] + ".json"):
        if os.path.isfile(c):
            return json.load(open(c))
    fail(f"no licence sidecar for {src}: write {src}.json as {{\"source\": URL, \"license\": \"CC0\", \"credit\": \"\"}}")


def nums(s, n, what):
    v = [float(x) for x in s.split(",")]
    if len(v) != n:
        fail(f"{what} takes {n} numbers, got {s!r}")
    return v


def b(p):
    return Vector((p[0], -p[2], p[1]))


def export(path, objs, side, step, morph):
    bpy.ops.object.select_all(action="DESELECT")
    for o in objs:
        o.select_set(True)
    bpy.ops.export_scene.gltf(filepath=path, export_format="GLB", export_yup=True, use_selection=True,
                              export_morph=morph, export_animations=morph, export_morph_animation=morph)
    json.dump({**side, "steps": list(side.get("steps", [])) + [step]}, open(path + ".json", "w"), indent=2)


def main():
    a = argv()
    if len(a) < 2 or not (opt(a, "--sheet") or opt(a, "--cloth")):
        fail("usage: cloth.py IN.glb OUT.glb (--sheet W,D,X,Y,Z | --cloth NAME) [--pin box] [--frames 60] [--every 3]")
    src, out = os.path.abspath(a[0]), os.path.abspath(a[1])
    side = sidecar(src)
    bpy.ops.wm.read_factory_settings(use_empty=True)
    bpy.ops.import_scene.gltf(filepath=src)
    ms = [o for o in bpy.context.scene.objects if o.type == "MESH"]
    bpy.ops.object.select_all(action="DESELECT")
    for o in ms:
        o.select_set(True)
    if ms:
        bpy.context.view_layer.objects.active = ms[0]
        bpy.ops.object.parent_clear(type="CLEAR_KEEP_TRANSFORM")
    cuts = int(opt(a, "--cuts", "24"))
    if opt(a, "--sheet"):
        w, d, x, y, z = nums(opt(a, "--sheet"), 5, "--sheet")
        bpy.ops.mesh.primitive_grid_add(x_subdivisions=cuts, y_subdivisions=cuts, size=1, location=b([x, y, z]))
        cloth = bpy.context.active_object
        cloth.scale = (w, d, 1)
        bpy.ops.object.transform_apply(location=False, rotation=False, scale=True)
        bpy.ops.object.shade_smooth()  # cloth is soft: the grid's flat faces read as folded card
        cloth.name = "cloth"
        # a sheet laid perfectly flat lands perfectly flat: --crumple lifts it in a few soft seeded waves (up to that
        # many metres, folds about 12 cm apart), so it lands in folds like a rag dropped from the hand
        crumple = float(opt(a, "--crumple", "0"))
        if crumple > 0:
            import math, random
            rnd = random.Random(7)
            waves = [(rnd.uniform(-1, 1), rnd.uniform(-1, 1), rnd.uniform(0, 6.3)) for _ in range(4)]
            k = 2 * math.pi / 0.12
            for v in cloth.data.vertices:
                h = sum(math.sin(k * (dx * v.co.x + dy * v.co.y) + ph) for dx, dy, ph in waves) / len(waves)
                v.co.z += crumple * (0.5 + 0.5 * h)
    else:
        cloth = bpy.data.objects.get(opt(a, "--cloth"))
        if cloth is None or cloth.type != "MESH":
            fail(f"no mesh {opt(a, '--cloth')} in {src}: {', '.join(o.name for o in ms)}")
    colliders = [o for o in ms if o != cloth]
    if "--no-floor" not in a:
        # the floor at y 0, so a cloth that slides off lands instead of falling forever (it isn't exported)
        bpy.ops.mesh.primitive_plane_add(size=20, location=(0, 0, 0))
        colliders.append(bpy.context.active_object)
    for o in colliders:
        o.modifiers.new("collide", "COLLISION")
        # thick enough that a fall of a few centimetres a frame can't pass through a big flat face, with friction so
        # the cloth stops where it lands instead of skating off (or being thrown, when it ends up inside)
        o.collision.thickness_outer = 0.01
        o.collision.cloth_friction = 40
        o.collision.damping = 0.5
    if opt(a, "--pin"):
        x0, y0, z0, x1, y1, z1 = nums(opt(a, "--pin"), 6, "--pin")
        lo, hi = b([min(x0, x1), min(y0, y1), max(z0, z1)]), b([max(x0, x1), max(y0, y1), min(z0, z1)])
        g = cloth.vertex_groups.new(name="pin")
        mw = cloth.matrix_world
        idx = [v.index for v in cloth.data.vertices if all(min(lo[k], hi[k]) <= (mw @ v.co)[k] <= max(lo[k], hi[k]) for k in range(3))]
        if not idx:
            fail("--pin holds no vertex of the cloth")
        g.add(idx, 1.0, "REPLACE")
    m = cloth.modifiers.new("cloth", "CLOTH")
    m.settings.mass = float(opt(a, "--mass", "0.15"))
    m.settings.tension_stiffness = m.settings.compression_stiffness = float(opt(a, "--stiff", "5"))
    m.settings.bending_stiffness = float(opt(a, "--bend", "0.05"))
    m.settings.quality = 12
    m.settings.air_damping = 2
    m.collision_settings.collision_quality = 4
    m.collision_settings.use_self_collision = True
    m.collision_settings.distance_min = 0.004
    m.collision_settings.self_distance_min = 0.004
    if opt(a, "--pin"):
        m.settings.vertex_group_mass = "pin"
    frames, every = int(opt(a, "--frames", "60")), max(1, int(opt(a, "--every", "3")))
    sc = bpy.context.scene
    sc.frame_start, sc.frame_end = 1, frames
    m.point_cache.frame_start, m.point_cache.frame_end = 1, frames
    # step the simulation, keeping the cloth's shape at every sampled frame
    shapes = []
    for f in range(1, frames + 1):
        sc.frame_set(f)
        if f == 1 or f % every == 0 or f == frames:
            ev = cloth.evaluated_get(bpy.context.evaluated_depsgraph_get())
            shapes.append((f, [v.co.copy() for v in ev.data.vertices]))
    cloth.modifiers.remove(m)
    if not cloth.data.shape_keys:
        cloth.shape_key_add(name="flat")
    keys = []
    for f, co in shapes[1:]:
        k = cloth.shape_key_add(name=f"f{f}")
        for i, c in enumerate(co):
            k.data[i].co = c
        keys.append((f, k))
    # one key on at a time, eased between neighbours
    for i, (f, k) in enumerate(keys):
        prev = shapes[i][0]
        nxt = keys[i + 1][0] if i + 1 < len(keys) else None
        for at, val in ((prev, 0.0), (f, 1.0)) + (((nxt, 0.0),) if nxt else ()):
            k.value = val
            k.keyframe_insert("value", frame=at)
    act = cloth.data.shape_keys.animation_data.action
    act.name = "Drape"
    for o in colliders:
        o.hide_set(True)
    settled_co = shapes[-1][1]
    moved = max((c - cloth.data.vertices[i].co).length for i, c in enumerate(settled_co))
    if moved < 1e-4:
        fail("the cloth never moved: is it inside what it should fall onto, or pinned everywhere?")
    export(out, [cloth], side, {"script": NAME, "from": os.path.basename(src), "frames": frames, "keys": len(keys)}, True)
    # the settled cloth as a plain mesh
    cloth.animation_data_clear() if cloth.animation_data else None
    cloth.data.shape_keys.animation_data_clear()
    bpy.ops.object.select_all(action="DESELECT")
    cloth.select_set(True)
    bpy.context.view_layer.objects.active = cloth
    for k in list(cloth.data.shape_keys.key_blocks)[1:]:
        k.value = 0.0
    cloth.data.shape_keys.key_blocks[-1].value = 1.0
    bpy.ops.object.shape_key_remove(all=True, apply_mix=True)
    settled = out[:-4] + ".settled.glb"
    export(settled, [cloth], side, {"script": NAME, "from": os.path.basename(src), "frames": frames, "settled": True}, False)
    print(f"APPLIED  {NAME}: {out}: clip Drape, {len(keys)} morph targets over {frames} frames (the cloth moved up to {moved:.3f} m); settled mesh {settled}")


main()
