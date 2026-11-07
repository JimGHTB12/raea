# rig.py: a model with no skeleton given bones, weights and named pose clips, so raea can pose it (<Model
# animation="Reach">, the clip at the Sequence's time). Blender rigs; raea decides when a pose plays.
#   raea blender run blender/rig.py IN.glb OUT.glb --bone NAME=HX,HY,HZ:TX,TY,TZ[:PARENT] ... \
#       [--pose CLIP=BONE:RX,RY,RZ/BONE:RX,RY,RZ ...] [--weights rigid|auto] [--frames 24]
# Points are in raea's frame (metres, y up, the model's own space: gen check's box), head to tail. A pose turns bones
# by degrees about their own axes (X bends along the bone's plane, Y twists, Z bends sideways) and the clip eases from
# rest to it over --frames; every model also gets "Rest". --weights rigid gives each vertex wholly to its nearest bone
# (a lamp, a machine, a robot: parts that don't bend); auto is Blender's heat weighting (a creature, cloth-like parts),
# falling back to envelopes when heat weighting fails on a broken mesh (repair.py first is better).
import bpy, sys, os, json
from mathutils import Vector, Euler

NAME = "rig.py"


def fail(why):
    print(f"FAILED  {NAME}: {why}", file=sys.stderr)
    sys.exit(1)


def argv():
    return sys.argv[sys.argv.index("--") + 1:] if "--" in sys.argv else []


def opt(a, k, d=None):
    return a[a.index(k) + 1] if k in a and a.index(k) + 1 < len(a) else d


def many(a, k):
    return [a[i + 1] for i, x in enumerate(a) if x == k and i + 1 < len(a)]


def sidecar(src):
    for c in (src + ".json", os.path.splitext(src)[0] + ".json"):
        if os.path.isfile(c):
            return json.load(open(c))
    fail(f"no licence sidecar for {src}: write {src}.json as {{\"source\": URL, \"license\": \"CC0\", \"credit\": \"\"}}")


def load(src):
    bpy.ops.wm.read_factory_settings(use_empty=True)
    if not src.lower().endswith((".glb", ".gltf")):
        fail(f"{src}: give a .glb (convert.py makes one)")
    bpy.ops.import_scene.gltf(filepath=src)


def b(p):
    # raea (x, y up, z toward the viewer) to Blender (x, y away, z up)
    return Vector((p[0], -p[2], p[1]))


def point(s):
    v = [float(x) for x in s.split(",")]
    if len(v) != 3:
        fail(f"{s}: a point is X,Y,Z")
    return b(v)


def nearest(p, bones):
    best, d = None, float("inf")
    for name, h, t in bones:
        ab = t - h
        k = max(0.0, min(1.0, (p - h).dot(ab) / max(ab.length_squared, 1e-12)))
        e = (h + ab * k - p).length
        if e < d:
            best, d = name, e
    return best


def main():
    a = argv()
    specs = many(a, "--bone")
    if len(a) < 2 or not specs:
        fail("usage: rig.py IN.glb OUT.glb --bone NAME=HX,HY,HZ:TX,TY,TZ[:PARENT] ... [--pose CLIP=BONE:RX,RY,RZ/...] [--weights rigid|auto]")
    src, out = os.path.abspath(a[0]), os.path.abspath(a[1])
    side = sidecar(src)
    load(src)
    if any(o.type == "ARMATURE" for o in bpy.context.scene.objects):
        fail("it already has a skeleton: pose its own bones (its clips), or strip it in a script of your own first")
    ms = [o for o in bpy.context.scene.objects if o.type == "MESH"]
    if not ms:
        fail(f"{src} holds no mesh")
    bpy.ops.object.select_all(action="DESELECT")
    for o in ms:
        o.select_set(True)
    bpy.context.view_layer.objects.active = ms[0]
    bpy.ops.object.parent_clear(type="CLEAR_KEEP_TRANSFORM")
    if len(ms) > 1:
        bpy.ops.object.join()
    body = bpy.context.view_layer.objects.active
    bpy.ops.object.transform_apply(location=True, rotation=True, scale=True)
    bones = []
    for s in specs:
        name, rest = s.split("=", 1)
        parts = rest.split(":")
        if len(parts) < 2:
            fail(f"{s}: a bone is NAME=HX,HY,HZ:TX,TY,TZ[:PARENT]")
        bones.append((name, point(parts[0]), point(parts[1]), parts[2] if len(parts) > 2 else None))
    names = [x[0] for x in bones]
    for _, _, _, p in bones:
        if p and p not in names:
            fail(f"parent {p} isn't a bone ({', '.join(names)})")
    arm = bpy.data.armatures.new("rig")
    rig = bpy.data.objects.new("rig", arm)
    bpy.context.scene.collection.objects.link(rig)
    bpy.context.view_layer.objects.active = rig
    bpy.ops.object.mode_set(mode="EDIT")
    for name, h, t, _ in bones:
        e = arm.edit_bones.new(name)
        e.head, e.tail = h, t
    for name, _, _, p in bones:
        if p:
            arm.edit_bones[name].parent = arm.edit_bones[p]
    bpy.ops.object.mode_set(mode="OBJECT")
    mode = opt(a, "--weights", "rigid")
    how = mode
    if mode == "rigid":
        groups = {n: body.vertex_groups.new(name=n) for n in names}
        segs = [(n, h, t) for n, h, t, _ in bones]
        by = {n: [] for n in names}
        for v in body.data.vertices:
            by[nearest(v.co, segs)].append(v.index)
        for n, idx in by.items():
            groups[n].add(idx, 1.0, "REPLACE")
        m = body.modifiers.new("rig", "ARMATURE")
        m.object = rig
        body.parent = rig
        counts = {n: len(i) for n, i in by.items()}
    else:
        bpy.ops.object.select_all(action="DESELECT")
        body.select_set(True)
        rig.select_set(True)
        bpy.context.view_layer.objects.active = rig
        bpy.ops.object.parent_set(type="ARMATURE_AUTO")
        counts = {g.name: sum(1 for v in body.data.vertices if any(x.group == g.index and x.weight > 0.01 for x in v.groups)) for g in body.vertex_groups}
        if not any(counts.values()):
            body.parent = None
            body.modifiers.clear()
            body.vertex_groups.clear()
            bpy.ops.object.select_all(action="DESELECT")
            body.select_set(True)
            rig.select_set(True)
            bpy.ops.object.parent_set(type="ARMATURE_ENVELOPE")
            how = "envelope (heat weighting failed: the mesh is broken; repair.py first)"
            counts = {}
    empty = [n for n, c in counts.items() if c == 0]
    if empty:
        fail(f"bones with no vertices: {', '.join(empty)}; move them onto the part they should carry")
    frames = int(opt(a, "--frames", "24"))
    clips = {"Rest": {}}
    for s in many(a, "--pose"):
        clip, rest = s.split("=", 1)
        turns = {}
        for t in rest.split("/"):
            bone, r = t.split(":")
            if bone not in names:
                fail(f"pose {clip}: no bone {bone}")
            turns[bone] = [float(x) for x in r.split(",")]
        clips[clip] = turns
    rig.animation_data_create()
    for pb in rig.pose.bones:
        pb.rotation_mode = "XYZ"
    for clip, turns in clips.items():
        act = bpy.data.actions.new(clip)
        rig.animation_data.action = act
        for f, k in ((1, 0.0), (frames, 1.0)):
            for pb in rig.pose.bones:
                r = turns.get(pb.name, [0, 0, 0])
                pb.rotation_euler = Euler([x * 3.14159265 / 180 * k for x in r], "XYZ")
                pb.keyframe_insert("rotation_euler", frame=f)
        act.use_fake_user = True
        tr = rig.animation_data.nla_tracks.new()
        tr.name = clip
        tr.strips.new(clip, 1, act)
        rig.animation_data.action = None
    bpy.context.scene.frame_end = frames
    bpy.ops.export_scene.gltf(filepath=out, export_format="GLB", export_yup=True, export_animation_mode="NLA_TRACKS", export_skins=True)
    json.dump({**side, "steps": list(side.get("steps", [])) + [{"script": NAME, "from": os.path.basename(src), "bones": names, "weights": how, "clips": list(clips)}]},
              open(out + ".json", "w"), indent=2)
    print(f"APPLIED  {NAME}: {out}: bones {', '.join(f'{n} ({counts.get(n, '?')} vertices)' for n in names)}; weights {how}; clips {', '.join(clips)} (frame {frames} is the pose)")


main()
