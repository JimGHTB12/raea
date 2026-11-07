# split.py: one blob cut into named parts, so a part can move on its own (a vise's jaw, a cabinet's door).
#   raea blender run blender/split.py IN.glb OUT.glb NAME=X0,Y0,Z0,X1,Y1,Z1 ... [--rest body] [--each]
# Each box is in raea's frame (metres, y up, the model's own space, as gen check's box reports it); the faces whose
# centres fall inside it become the part NAME, and the rest is --rest. OUT holds every part as its own node. --each
# also writes OUT.NAME.glb per part with its origin at the part's bottom centre, to place and move in TSX
# (<Model src=".../vise.jaw.glb" position={...}>): raea moves whole files, not a file's nodes.
# The cut follows the triangles, so a box that misses a part's edge leaves a ragged edge: read the part sizes it prints.
import bpy, bmesh, sys, os, json
from mathutils import Vector

NAME = "split.py"


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


def load(src):
    bpy.ops.wm.read_factory_settings(use_empty=True)
    if not src.lower().endswith((".glb", ".gltf")):
        fail(f"{src}: give a .glb (convert.py makes one)")
    bpy.ops.import_scene.gltf(filepath=src)


def meshes():
    return [o for o in bpy.context.scene.objects if o.type == "MESH"]


def save(out, side, step, only=None):
    bpy.ops.object.select_all(action="DESELECT")
    for o in (only or meshes()):
        o.select_set(True)
    bpy.ops.export_scene.gltf(filepath=out, export_format="GLB", export_yup=True, use_selection=True)
    json.dump({**side, "steps": list(side.get("steps", [])) + [step]}, open(out + ".json", "w"), indent=2)


def blender_box(raea):
    # raea (glTF): x right, y up, z toward the viewer; Blender: x right, y away, z up
    x0, y0, z0, x1, y1, z1 = raea
    lo = Vector((min(x0, x1), min(-z0, -z1), min(y0, y1)))
    hi = Vector((max(x0, x1), max(-z0, -z1), max(y0, y1)))
    return lo, hi


def size_of(o):
    ws = [o.matrix_world @ v.co for v in o.data.vertices]
    lo = Vector([min(w[k] for w in ws) for k in range(3)])
    hi = Vector([max(w[k] for w in ws) for k in range(3)])
    return lo, hi


def main():
    a = argv()
    boxes = [x for x in a[2:] if "=" in x and not x.startswith("--")]
    if len(a) < 3 or not boxes:
        fail("usage: split.py IN.glb OUT.glb NAME=X0,Y0,Z0,X1,Y1,Z1 ... [--rest body] [--each]")
    src, out = os.path.abspath(a[0]), os.path.abspath(a[1])
    side = sidecar(src)
    load(src)
    ms = meshes()
    if not ms:
        fail(f"{src} holds no mesh")
    bpy.ops.object.select_all(action="DESELECT")
    for o in ms:
        o.select_set(True)
    bpy.context.view_layer.objects.active = ms[0]
    # boxes are in the model's world: a fitted model's meshes sit under a parent node (gen check's raea-fixed)
    bpy.ops.object.parent_clear(type="CLEAR_KEEP_TRANSFORM")
    if len(ms) > 1:
        bpy.ops.object.join()
    body = bpy.context.view_layer.objects.active
    bpy.ops.object.transform_apply(location=True, rotation=True, scale=True)
    body.name = opt(a, "--rest", "body")
    parts = []
    for spec in boxes:
        name, nums = spec.split("=", 1)
        v = [float(x) for x in nums.split(",")]
        if len(v) != 6:
            fail(f"{spec}: a box is NAME=X0,Y0,Z0,X1,Y1,Z1")
        lo, hi = blender_box(v)
        inside = [all(lo[k] <= p.center[k] <= hi[k] for k in range(3)) for p in body.data.polygons]
        if not any(inside):
            fail(f"{name}: no faces inside its box; read the model's box with raea gen check and try again")
        part = body.copy()
        part.data = body.data.copy()
        bpy.context.scene.collection.objects.link(part)
        for o, keep in ((part, True), (body, False)):
            bm = bmesh.new()
            bm.from_mesh(o.data)
            bm.faces.ensure_lookup_table()
            gone = [f for f in bm.faces if inside[f.index] != keep]
            bmesh.ops.delete(bm, geom=gone, context="FACES")
            bmesh.ops.delete(bm, geom=[v for v in bm.verts if not v.link_faces], context="VERTS")
            bm.to_mesh(o.data)
            bm.free()
        part.name = part.data.name = name
        parts.append(part)
    if not body.data.polygons:
        fail("the boxes took every face: nothing is left for the rest")
    report = {}
    for o in [body] + parts:
        lo, hi = size_of(o)
        report[o.name] = {"triangles": sum(len(p.vertices) - 2 for p in o.data.polygons), "size": [round(hi.x - lo.x, 4), round(hi.z - lo.z, 4), round(hi.y - lo.y, 4)]}
    save(out, side, {"script": NAME, "from": os.path.basename(src), "parts": report})
    print(f"APPLIED  {NAME}: {out}: " + ", ".join(f"{k} {v['triangles']} triangles {v['size']} m (w h d)" for k, v in report.items()))
    if "--each" in a:
        for o in [body] + parts:
            lo, hi = size_of(o)
            pivot = Vector(((lo.x + hi.x) / 2, (lo.y + hi.y) / 2, lo.z))
            bpy.context.scene.cursor.location = pivot
            bpy.ops.object.select_all(action="DESELECT")
            o.select_set(True)
            bpy.context.view_layer.objects.active = o
            bpy.ops.object.origin_set(type="ORIGIN_CURSOR")
            at = o.location.copy()
            o.location = (0, 0, 0)
            each = out[:-4] + f".{o.name}.glb"
            # where the part sat in the model, in raea's frame: place it there to put the model back together
            save(each, side, {"script": NAME, "from": os.path.basename(src), "part": o.name, "at": [round(at.x, 4), round(at.z, 4), round(-at.y, 4)]}, only=[o])
            print(f"APPLIED  {NAME}: {each} (origin at its bottom centre; it sat at {[round(at.x, 4), round(at.z, 4), round(-at.y, 4)]} in the model)")
            o.location = at


main()
