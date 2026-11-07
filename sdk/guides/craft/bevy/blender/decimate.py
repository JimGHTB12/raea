# decimate.py: a model over its triangle budget brought under it, its UVs and materials kept.
#   raea blender run blender/decimate.py IN.glb OUT.glb TARGET [--lods 0.5,0.25]
# TARGET is a triangle count (20000) or a ratio of what it has (0.25); gen check's and scene lint's `fix` give the
# count for its viewing distance. --lods also writes OUT.lod1.glb, OUT.lod2.glb at those fractions of TARGET, for far
# placements of the same thing (raea draws the file a Model names; use the small one where it's seen far).
# A model with shape keys is refused (a collapse can't keep them): decimate its source before it's rigged.
import bpy, sys, os, json

NAME = "decimate.py"


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


def tris():
    return sum(sum(len(p.vertices) - 2 for p in o.data.polygons) for o in meshes())


def save(out, side, step):
    bpy.ops.export_scene.gltf(filepath=out, export_format="GLB", export_yup=True)
    json.dump({**side, "steps": list(side.get("steps", [])) + [step]}, open(out + ".json", "w"), indent=2)


def collapse(ratio):
    for o in meshes():
        m = o.modifiers.new("raea-decimate", "DECIMATE")
        m.ratio = max(0.0005, min(1.0, ratio))
        m.use_collapse_triangulate = True
        bpy.context.view_layer.objects.active = o
        bpy.ops.object.modifier_apply(modifier=m.name)


def main():
    a = argv()
    if len(a) < 3:
        fail("usage: decimate.py IN.glb OUT.glb TARGET [--lods 0.5,0.25]")
    src, out = os.path.abspath(a[0]), os.path.abspath(a[1])
    side = sidecar(src)
    load(src)
    if not meshes():
        fail(f"{src} holds no mesh")
    if any(o.data.shape_keys for o in meshes()):
        fail("it has shape keys (morph targets), which a collapse can't keep: decimate the model before it's rigged or keyed")
    before = tris()
    t = float(a[2])
    target = int(t * before) if t <= 1.0 else int(t)
    if target >= before:
        print(f"APPLIED  {NAME}: {before} triangles, already within {target}: written unchanged")
    else:
        collapse(target / before)
    got = tris()
    if got > target * 1.15:
        fail(f"got {got} triangles, over the {target} asked (thin parts the collapse protects): repair.py --voxel first, or a lower target")
    save(out, side, {"script": NAME, "from": os.path.basename(src), "before": before, "after": got})
    print(f"APPLIED  {NAME}: {out}: {before} -> {got} triangles (target {target})")
    lods = [float(x) for x in opt(a, "--lods", "").split(",") if x.strip()]
    for i, f in enumerate(lods, 1):
        collapse(f * target / tris())
        lod = out[:-4] + f".lod{i}.glb"
        save(lod, side, {"script": NAME, "from": os.path.basename(src), "before": before, "after": tris(), "lod": i})
        print(f"APPLIED  {NAME}: {lod}: {tris()} triangles")


main()
