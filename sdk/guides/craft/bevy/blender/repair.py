# repair.py: a model's surface mended: doubled vertices merged, loose debris dropped, holes filled, normals pointed out;
# or, with --voxel, rebuilt as one closed surface.
#   raea blender run blender/repair.py IN.glb OUT.glb [--debris 0.005] [--voxel SIZE_M] [--keep-pieces]
# --debris: a separate piece with fewer than this share of the triangles is dropped (gen check's "loose bits");
# --keep-pieces keeps every piece (a tool modelled in parts). --voxel remeshes to a closed surface at that cell size in
# metres: for a mesh too broken to mend (it loses its UVs and textures, so bake.py after it). Prints what it changed.
import bpy, bmesh, sys, os, json

NAME = "repair.py"


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


def save(out, side, step):
    bpy.ops.export_scene.gltf(filepath=out, export_format="GLB", export_yup=True, export_apply=True)
    json.dump({**side, "steps": list(side.get("steps", [])) + [step]}, open(out + ".json", "w"), indent=2)


def edges(o):
    bm = bmesh.new()
    bm.from_mesh(o.data)
    bm.edges.ensure_lookup_table()
    opened = sum(1 for e in bm.edges if len(e.link_faces) == 1)
    shared = sum(1 for e in bm.edges if len(e.link_faces) > 2)
    bm.free()
    return opened, shared


def pieces(bm):
    seen, out = set(), []
    for f in bm.faces:
        if f.index in seen:
            continue
        stack, part = [f], []
        seen.add(f.index)
        while stack:
            g = stack.pop()
            part.append(g)
            for e in g.edges:
                for h in e.link_faces:
                    if h.index not in seen:
                        seen.add(h.index)
                        stack.append(h)
        out.append(part)
    return out


def mend(o, debris, keep, size):
    bm = bmesh.new()
    bm.from_mesh(o.data)
    n0 = len(bm.verts)
    bmesh.ops.remove_doubles(bm, verts=bm.verts, dist=max(1e-6, size * 1e-5))
    merged = n0 - len(bm.verts)
    bm.faces.ensure_lookup_table()
    dropped = 0
    if not keep:
        parts = pieces(bm)
        total = sum(len(p) for p in parts)
        small = [f for p in parts if len(p) < debris * total for f in p]
        dropped = len([p for p in parts if len(p) < debris * total])
        bmesh.ops.delete(bm, geom=small, context="FACES")
    holes = bmesh.ops.holes_fill(bm, edges=bm.edges, sides=64)
    bmesh.ops.recalc_face_normals(bm, faces=bm.faces)
    bm.to_mesh(o.data)
    bm.free()
    return merged, dropped, len(holes["faces"])


def main():
    a = argv()
    if len(a) < 2:
        fail("usage: repair.py IN.glb OUT.glb [--debris 0.005] [--voxel SIZE_M] [--keep-pieces]")
    src, out = os.path.abspath(a[0]), os.path.abspath(a[1])
    side = sidecar(src)
    load(src)
    ms = meshes()
    if not ms:
        fail(f"{src} holds no mesh")
    for o in ms:
        o.select_set(True)
    bpy.context.view_layer.objects.active = ms[0]
    bpy.ops.object.transform_apply(location=False, rotation=True, scale=True)
    size = max(max(o.dimensions) for o in ms)
    before = [edges(o) for o in ms]
    voxel = opt(a, "--voxel")
    did = {}
    if voxel:
        for o in ms:
            m = o.modifiers.new("raea-remesh", "REMESH")
            m.mode, m.voxel_size = "VOXEL", float(voxel)
            bpy.context.view_layer.objects.active = o
            bpy.ops.object.modifier_apply(modifier=m.name)
        did["voxel"] = float(voxel)
        print(f"NOTE  {NAME}: remeshed at {voxel} m: UVs and textures are gone; bake.py gives it new ones")
    else:
        got = [mend(o, float(opt(a, "--debris", "0.005")), "--keep-pieces" in a, size) for o in ms]
        did = {"merged": sum(g[0] for g in got), "debrisDropped": sum(g[1] for g in got), "holesFilled": sum(g[2] for g in got)}
    after = [edges(o) for o in ms]
    step = {"script": NAME, "from": os.path.basename(src), **did, "openEdges": [sum(b[0] for b in before), sum(x[0] for x in after)],
            "sharedEdges": [sum(b[1] for b in before), sum(x[1] for x in after)]}
    save(out, side, step)
    print(f"APPLIED  {NAME}: {out}: {did}; open edges {step['openEdges'][0]} -> {step['openEdges'][1]}, shared {step['sharedEdges'][0]} -> {step['sharedEdges'][1]}")


main()
