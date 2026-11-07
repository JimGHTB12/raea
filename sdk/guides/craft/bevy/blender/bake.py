# bake.py: a model given real UVs and baked maps: ambient occlusion (where light can't reach: under a lip, in a seam,
# where a part meets another), and a normal map from a high-poly source onto this low one.
#   raea blender run blender/bake.py IN.glb OUT.glb [--res 1024] [--samples 32] [--high HIGH.glb] [--into-colour]
#       [--texture IMG.jpg --tile METRES [--turn DEG]]
# The UVs are a fresh smart projection on a second map ("bake") when the model's textures use its first, else they
# replace the first (a solid's box UVs). The AO goes in the material's occlusion slot (glTF's; Bevy darkens ambient light
# with it); --into-colour also multiplies it into the base colour (as a texture), so it shows under direct light too:
# use it for a solid with flat colours. --texture puts a photo texture on it first (wood, metal, paint), projected from
# the box's three sides at --tile metres a repeat (a real scale, the same on every face, no stretched islands), and
# bakes that times the AO into the colour; --turn turns it about the vertical so the grain runs along the piece. --high bakes HIGH's surface detail as a normal map (both models in the same
# space: decimate.py's input and output). Needs Cycles; a 1024 map of a prop takes seconds to a minute.
import bpy, sys, os, json

NAME = "bake.py"


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


def meshes():
    return [o for o in bpy.context.scene.objects if o.type == "MESH"]


def textured(o):
    return any(n.type == "TEX_IMAGE" for s in o.material_slots if s.material and s.material.node_tree for n in s.material.node_tree.nodes)


def gltf_output(nt):
    # the glTF exporter reads occlusion from a node group of this name
    g = bpy.data.node_groups.get("glTF Material Output")
    if g is None:
        g = bpy.data.node_groups.new("glTF Material Output", "ShaderNodeTree")
        g.interface.new_socket("Occlusion", in_out="INPUT", socket_type="NodeSocketFloat")
    n = nt.nodes.new("ShaderNodeGroup")
    n.node_tree = g
    return n


def main():
    a = argv()
    if len(a) < 2:
        fail("usage: bake.py IN.glb OUT.glb [--res 1024] [--samples 32] [--high HIGH.glb] [--into-colour [--texture IMG --tile M]]")
    src, out = os.path.abspath(a[0]), os.path.abspath(a[1])
    side = sidecar(src)
    bpy.ops.wm.read_factory_settings(use_empty=True)
    bpy.ops.import_scene.gltf(filepath=src)
    ms = meshes()
    if not ms:
        fail(f"{src} holds no mesh")
    bpy.ops.object.select_all(action="DESELECT")
    for o in ms:
        o.select_set(True)
    bpy.context.view_layer.objects.active = ms[0]
    bpy.ops.object.parent_clear(type="CLEAR_KEEP_TRANSFORM")
    if len(ms) > 1:
        bpy.ops.object.join()
    low = bpy.context.view_layer.objects.active
    low.name = "baked"
    bpy.ops.object.transform_apply(location=True, rotation=True, scale=True)
    keep_uv = textured(low) and len(low.data.uv_layers) > 0
    if not keep_uv:
        while low.data.uv_layers:
            low.data.uv_layers.remove(low.data.uv_layers[0])
    uv = low.data.uv_layers.new(name="bake")
    low.data.uv_layers.active = uv
    bpy.ops.object.mode_set(mode="EDIT")
    bpy.ops.mesh.select_all(action="SELECT")
    bpy.ops.uv.smart_project(angle_limit=1.15, island_margin=0.004)
    bpy.ops.object.mode_set(mode="OBJECT")
    res = int(opt(a, "--res", "1024"))
    sc = bpy.context.scene
    sc.render.engine = "CYCLES"
    sc.cycles.device = "CPU"
    sc.cycles.samples = int(opt(a, "--samples", "32"))
    sc.render.bake.margin = 8
    if not low.material_slots:
        m = bpy.data.materials.new("baked")
        m.use_nodes = True
        low.data.materials.append(m)
    mats = {s.material for s in low.material_slots if s.material}
    for m in mats:
        m.use_nodes = True

    def target(kind, colour):
        img = bpy.data.images.new(f"{kind}", res, res, alpha=False, float_buffer=False)
        img.colorspace_settings.name = "sRGB" if colour else "Non-Color"
        for m in mats:
            nt = m.node_tree
            for n in nt.nodes:
                n.select = False
            t = nt.nodes.new("ShaderNodeTexImage")
            t.image, t.name, t.select = img, f"raea-{kind}", True
            nt.nodes.active = t
            u = nt.nodes.new("ShaderNodeUVMap")
            u.uv_map = "bake"
            nt.links.new(u.outputs["UV"], t.inputs["Vector"])
        return img

    did = []
    ao = target("ao", False)
    bpy.ops.object.select_all(action="DESELECT")
    low.select_set(True)
    bpy.context.view_layer.objects.active = low
    bpy.ops.object.bake(type="AO")
    did.append("ao")
    high = opt(a, "--high")
    nmap = None
    if high:
        nmap = target("normal", False)
        before = set(bpy.context.scene.objects)
        bpy.ops.import_scene.gltf(filepath=os.path.abspath(high))
        hs = [o for o in bpy.context.scene.objects if o not in before and o.type == "MESH"]
        if not hs:
            fail(f"{high} holds no mesh")
        bpy.ops.object.select_all(action="DESELECT")
        for h in hs:
            h.select_set(True)
        low.select_set(True)
        bpy.context.view_layer.objects.active = low
        size = max(low.dimensions)
        bpy.ops.object.bake(type="NORMAL", use_selected_to_active=True, cage_extrusion=size * 0.02, max_ray_distance=size * 0.05)
        for h in hs:
            bpy.data.objects.remove(h)
        did.append("normal")
    # wire the maps in: occlusion through the glTF output group, the normal map into the shader, AO into colour
    for m in mats:
        nt = m.node_tree
        bsdf = next((n for n in nt.nodes if n.type == "BSDF_PRINCIPLED"), None)
        if bsdf is None:
            continue
        aon = nt.nodes["raea-ao"]
        nt.links.new(aon.outputs["Color"], gltf_output(nt).inputs["Occlusion"])
        if nmap is not None:
            nm = nt.nodes.new("ShaderNodeNormalMap")
            nm.uv_map = "bake"
            nt.links.new(nt.nodes["raea-normal"].outputs["Color"], nm.inputs["Color"])
            nt.links.new(nm.outputs["Normal"], bsdf.inputs["Normal"])
        if "--into-colour" in a and opt(a, "--texture"):
            # the photo projected from the box's sides at a real scale, baked through the fresh UVs, times the AO
            base = bsdf.inputs["Base Color"]
            if base.is_linked:
                fail("--texture is for flat colours: this material's base colour is a texture already")
            tile = float(opt(a, "--tile", "0.5"))
            src_img = bpy.data.images.load(os.path.abspath(opt(a, "--texture")))
            tc = nt.nodes.new("ShaderNodeTexCoord")
            mp = nt.nodes.new("ShaderNodeMapping")
            mp.inputs["Scale"].default_value = (1 / tile, 1 / tile, 1 / tile)
            mp.inputs["Rotation"].default_value = (0, 0, float(opt(a, "--turn", "0")) * 3.14159265 / 180)
            nt.links.new(tc.outputs["Object"], mp.inputs["Vector"])
            tx = nt.nodes.new("ShaderNodeTexImage")
            tx.image, tx.projection, tx.projection_blend = src_img, "BOX", 0.2
            nt.links.new(mp.outputs["Vector"], tx.inputs["Vector"])
            nt.links.new(tx.outputs["Color"], base)
            img = bpy.data.images.new(f"colour-{m.name}", res, res, alpha=False)
            for n in nt.nodes:
                n.select = False
            t = nt.nodes.new("ShaderNodeTexImage")
            t.image, t.select = img, True
            nt.nodes.active = t
            u = nt.nodes.new("ShaderNodeUVMap")
            u.uv_map = "bake"
            nt.links.new(u.outputs["UV"], t.inputs["Vector"])
            bpy.ops.object.select_all(action="DESELECT")
            low.select_set(True)
            bpy.context.view_layer.objects.active = low
            bpy.ops.object.bake(type="DIFFUSE", pass_filter={"COLOR"})
            px, occ = [0.0] * (res * res * 4), [0.0] * (res * res * 4)
            img.pixels.foreach_get(px)
            ao.pixels.foreach_get(occ)
            # the colour bake is linear in a float view of an sRGB image: multiply in linear, store as display values
            lin = lambda c: c / 12.92 if c <= 0.04045 else ((c + 0.055) / 1.055) ** 2.4
            srgb = lambda c: 12.92 * c if c <= 0.0031308 else 1.055 * c ** (1 / 2.4) - 0.055
            for i in range(0, len(px), 4):
                k = occ[i]
                px[i], px[i + 1], px[i + 2], px[i + 3] = srgb(lin(px[i]) * k), srgb(lin(px[i + 1]) * k), srgb(lin(px[i + 2]) * k), 1.0
            img.pixels.foreach_set(px)
            for n in (tc, mp, tx):
                nt.nodes.remove(n)
            nt.links.new(t.outputs["Color"], base)
            did.append(f"texture {os.path.basename(opt(a, '--texture'))} at {tile} m")
        elif "--into-colour" in a:
            base = bsdf.inputs["Base Color"]
            if base.is_linked:
                fail("--into-colour is for flat colours: this material's base colour is a texture already (its occlusion slot has the AO)")
            # a texture of the flat colour times the AO, baked with the same UVs
            col = list(base.default_value)
            img = bpy.data.images.new(f"colour-{m.name}", res, res, alpha=False)
            px = [0.0] * (res * res * 4)
            ao.pixels.foreach_get(px)
            # the base colour is linear; an 8-bit sRGB image holds display values
            srgb = lambda c: 12.92 * c if c <= 0.0031308 else 1.055 * c ** (1 / 2.4) - 0.055
            for i in range(0, len(px), 4):
                k = px[i]
                px[i], px[i + 1], px[i + 2], px[i + 3] = srgb(col[0] * k), srgb(col[1] * k), srgb(col[2] * k), 1.0
            img.pixels.foreach_set(px)
            t = nt.nodes.new("ShaderNodeTexImage")
            t.image = img
            u = nt.nodes.new("ShaderNodeUVMap")
            u.uv_map = "bake"
            nt.links.new(u.outputs["UV"], t.inputs["Vector"])
            nt.links.new(t.outputs["Color"], base)
            did.append("colour")
    for img in bpy.data.images:
        if img.name.startswith(("ao", "normal", "colour-")):
            img.pack()
    bpy.ops.export_scene.gltf(filepath=out, export_format="GLB", export_yup=True)
    json.dump({**side, "steps": list(side.get("steps", [])) + [{"script": NAME, "from": os.path.basename(src), "res": res, "maps": did, "uvKept": keep_uv}]},
              open(out + ".json", "w"), indent=2)
    print(f"APPLIED  {NAME}: {out}: {', '.join(did)} at {res}px; UVs {'added beside its own' if keep_uv else 'made fresh'}")


main()
