"""A character for raea/character: an Anny body (NAVER LABS' parametric human, Apache 2.0; MakeHuman's CC0
assets) on Anny's cmu_mb rig, the 31 joints of CMU's motion capture skeleton, so CMU's clips play on it as they are,
and after them 30 of its own: each finger's and thumb's three phalanges, so a grasp can pose each.

    uv venv -p 3.12 ~/.cache/raea-anny-venv && uv pip install -p ~/.cache/raea-anny-venv/bin/python anny trimesh scipy roma pillow
    ~/.cache/raea-anny-venv/bin/python sdk/raea/tools/anny/character.py NAME [gender=0.5 age=0.3 ...] --out DIR

writes DIR/NAME/:
  body.glb       Anny's whole mesh skinned with her default rig's weights (its shoulder helpers and twist bones) to 68
                 joints that follow the 61, the playtest's graph paper baked into its texture in the
                 coordinates each face had at rest (so the grid rides the body as it bends, and a paint look shades it)
  body.hull.glb  its ink outline (the mesh pushed out along its normals and turned inside out), skinned the same; looks
                 that draw their own outlines leave it out
  joint.glb      one tiny triangle: the hidden model each joint follows (<Model bones>)
  skeleton.json  the joints (names, parents, rest heads), the body's sliders, its height and landmarks, and samples of
                 the skin's surface (each with its skin weights, so posed they are the skinned mesh's own points)

The skin is bound in CMU's T-pose (MPFB2's, shipped with Anny), and every joint's rest frame is the world's (no bone
roll), so a pose is each joint's turn from the T-pose in world axes as a BVH gives it. Coordinates: y up, the body facing +z, x its left, feet on y = 0, metres (Anny's (x, y, z) is
(x, -z, y) here)."""
import io, json, os, re, struct, sys
import numpy as np, torch, trimesh, anny

CLAY = (236, 235, 230)
INK = (42, 42, 40)
OUTLINE = 0.007  # the outline's thickness, metres
GRID = 0.05  # the graph paper's cell, metres; every fourth line a major one
GRID_COLOUR = (169, 169, 164)
# Anny's default rig's finger1..5 (each three phalanges, finger1-1 the thumb's metacarpal)
FING = ["Thumb", "Index", "Middle", "Ring", "Pinky"]
# the surface samples a skin joint gives (`surface`), by the capture joint that carries it: the hands densest
SAMPLES = {"Hand": 40, "finger": 20, "Hips": 120, "LowerBack": 120, "Spine": 120, "Spine1": 120, "Head": 80}
# the sliders' 0..1 (Anny extrapolates past them); the rest at 0.5
PRESETS = {
    "adult": {},
    "kid": {"gender": 0.5, "age": 0.3, "weight": 0.45, "height": 0.4},
    # the parkour course's man and woman (examples/character): about 1.77 m and 1.65 m
    "man": {"gender": 1.0, "age": 0.5, "muscle": 0.65, "weight": 0.5, "height": 0.68},
    "woman": {"gender": 0.0, "age": 0.5, "muscle": 0.45, "weight": 0.45, "height": 0.45},
}


def to_game(p):
    p = np.asarray(p)
    return np.stack([p[..., 0], p[..., 2], -p[..., 1]], axis=-1)


def paper(px=512):
    """one tile of graph paper (four cells a side, the major line on its edge) on the clay, as PNG bytes"""
    from PIL import Image
    img = np.tile(np.array(CLAY, np.uint8), (px, px, 1))
    cell = px // 4
    for k in range(4):
        w = 5 if k == 0 else 2
        for a in range(-(w // 2), w - w // 2):
            img[(k * cell + a) % px, :] = GRID_COLOUR
            img[:, (k * cell + a) % px] = GRID_COLOUR
    buf = io.BytesIO()
    Image.fromarray(img).save(buf, format="PNG")
    return buf.getvalue()


def glb(path, doc, chunks):
    """a GLB from its JSON and its chunks (bytes, target), laid one after another 4-byte aligned as buffer views"""
    views, blob = [], b""
    for data, target in chunks:
        blob += b"\0" * (-len(blob) % 4)
        v = {"buffer": 0, "byteOffset": len(blob), "byteLength": len(data)}
        if target:
            v["target"] = target
        views.append(v)
        blob += data
    blob += b"\0" * (-len(blob) % 4)
    doc = {"asset": {"version": "2.0", "generator": "raea sdk/raea/tools/anny/character.py"}, **doc,
           "bufferViews": views, "buffers": [{"byteLength": len(blob)}]}
    js = json.dumps(doc, separators=(",", ":")).encode()
    js += b" " * (-len(js) % 4)
    out = struct.pack("<III", 0x46546C67, 2, 12 + 8 + len(js) + 8 + len(blob))
    out += struct.pack("<II", len(js), 0x4E4F534A) + js + struct.pack("<II", len(blob), 0x004E4942) + blob
    with open(path, "wb") as fh:
        fh.write(out)


def t_pose(model, body):
    """MPFB2's T-pose for the cmu_mb rig (shipped with Anny): the turns that stand Anny's A-pose as CMU's skeleton stands
    at rest, collarbones up, arms level, forearms straight, hips splayed. Each is a Blender Euler XYZ in its bone's own
    frame; Anny's pose turns are in the world's axes, so each goes through its bone's rest frame."""
    from scipy.spatial.transform import Rotation
    L = model.bone_labels
    P = torch.eye(4, dtype=torch.float64)[None, None].repeat(1, model.bone_count, 1, 1)
    with torch.no_grad():
        F = model(pose_parameters=P, phenotype_kwargs=body)["rest_bone_poses"][0][:, :3, :3].numpy()
    with open(os.path.join(anny.__path__[0], "data/mpfb2/poses/cmu_mb_fk/t-pose.json")) as fh:
        turns = json.load(fh)["bone_rotations"]
    for b, e in turns.items():
        i = L.index(b)
        P[0, i, :3, :3] = torch.tensor(F[i] @ Rotation.from_euler("xyz", e).as_matrix() @ F[i].T)
    return P


def export(model, name, body, out):
    L, parents = model.bone_labels, list(model.bone_parents)
    # the skin bound in CMU's T-pose, so a clip's turns play on it as they are: bound in Anny's A-pose, every bone's
    # turn had to be corrected to CMU's, the corrections couldn't say a bone's roll, and the shoulders came out wrong
    with torch.no_grad():
        o = model(pose_parameters=t_pose(model, body), phenotype_kwargs=body)
    verts = to_game(o["vertices"][0].numpy())
    heads = to_game(o["bone_poses"][0][:, :3, 3].numpy())
    # each tail carried by its bone's turn from rest
    turn = o["bone_poses"][0][:, :3, :3].numpy() @ np.transpose(o["rest_bone_poses"][0][:, :3, :3].numpy(), (0, 2, 1))
    tails = to_game(o["bone_poses"][0][:, :3, 3].numpy() + np.einsum("bij,bj->bi", turn, (o["rest_bone_tails"][0] - o["rest_bone_heads"][0]).numpy()))
    floor = verts[:, 1].min()
    verts[:, 1] -= floor; heads[:, 1] -= floor; tails[:, 1] -= floor
    faces = model.faces.numpy().astype(np.int64)
    # the skin: Anny's default rig's weights (its shoulder helpers and twist bones; the cmu_mb rig's own weights square
    # the shoulders off as the arms come down, whoever poses them), bound where Anny stands at rest, so each vertex is
    # skinned once. Its joints follow the CMU skeleton's: each takes the turn of the CMU joint it lies along, the
    # helpers a blend, and a pose's turns (from the T-pose) reach them through each CMU joint's turn from rest to the
    # T-pose (`bind`)
    full = anny.Anny(extrapolate_phenotypes=True).to(dtype=torch.float64)
    with torch.no_grad():
        fo = full(pose_parameters=torch.eye(4, dtype=torch.float64)[None, None].repeat(1, full.bone_count, 1, 1), phenotype_kwargs=body)
    skin_verts = to_game(fo["rest_vertices"][0].numpy()); skin_verts[:, 1] -= floor
    fheads = to_game(fo["rest_bone_heads"][0].numpy()); fheads[:, 1] -= floor
    a_heads = to_game(o["rest_bone_heads"][0].numpy()); a_heads[:, 1] -= floor
    G = np.array([[1, 0, 0], [0, 0, 1], [0, -1, 0]], float)  # Anny's axes to the game's
    from scipy.spatial.transform import Rotation
    bind_turns = [Rotation.from_matrix(G @ turn[j] @ G.T).as_quat().tolist() for j in range(len(L))]
    skin, FL, fpar = [], full.bone_labels, list(full.bone_parents)
    def joint_for(n):
        """the skin joint a default-rig bone's weights go to: its own, or a group's, or (face, eyes) its ancestor's"""
        side = n[-1] if n[-2:] in (".L", ".R") else ""
        S = {"L": "Left", "R": "Right"}.get(side, "")
        base = n[:-2] if side else n
        own = {"root": ("Hips",), "spine05": ("LowerBack",), "spine04": ("LowerBack",), "spine03": ("Spine",),
               "spine02": ("Spine1",), "spine01": ("Spine1",), "neck01": ("Neck",), "neck02": ("Neck1",), "neck03": ("Neck1",),
               "head": ("Head",), "pelvis": (f"{side}HipJoint",), "upperleg01": (S + "UpLeg",), "upperleg02": (S + "UpLeg",),
               "lowerleg01": (S + "Leg",), "lowerleg02": (S + "Leg",), "foot": (S + "Foot",), "clavicle": (S + "Shoulder",),
               "shoulder01": (S + "Shoulder", S + "Shoulder", S + "Arm", 0.5), "upperarm01": (S + "Arm", S + "Shoulder", S + "Arm", 0.85),
               "upperarm02": (S + "Arm",), "lowerarm01": (S + "ForeArm", S + "Arm", S + "ForeArm", 0.85), "lowerarm02": (S + "ForeArm",),
               "wrist": (S + "Hand",)}
        if base in own:
            return n, fheads[FL.index(n)], own[base]
        # each phalanx its own skin joint, carried by its own finger joint (FING, after the capture's 31)
        m = re.match(r"^finger([1-5])-([1-3])$", base)
        if m:
            F, ph = FING[int(m[1]) - 1], int(m[2])
            return f"{F.lower()}{ph}.{side}", knuckle.get(n, fheads[FL.index(n)]), (f"{S}Hand{F}{ph}",)
        groups = [(r"^metacarpal", "wrist", None), (r"^toe", "toes", S + "ToeBase")]
        for pat, g, drive in groups:
            if re.search(pat, base):
                if g == "wrist":
                    return f"wrist.{side}", fheads[FL.index(f"wrist.{side}")], (S + "Hand",)
                return f"{g}.{side}", a_heads[L.index(drive)], (drive,)
        return joint_for(FL[fpar[FL.index(n)]])
    # the fingers' own joints, after the capture's 31 (whose indices clips count on): each phalanx down from the hand,
    # and `follow`, the capture's joint the skin rode before (a phalanx 1 the finger base, 2 and 3 the finger, the thumb
    # the thumb). A pose that leaves them unturned (every clip) carries the fingers as before (character.tsx `pose`).
    # Their heads at the T-pose: the default rig's at rest carried as the joint they follow carries them
    Wd, Id = full.vertex_bone_weights.numpy().astype(np.float64), full.vertex_bone_indices.numpy().astype(np.int64)
    Rb = [G @ turn[j] @ G.T for j in range(len(L))]
    tpose = lambda f, x: heads[L.index(f)] + Rb[L.index(f)] @ (x - a_heads[L.index(f)])
    def carried(b):
        """the rest vertices a default-rig bone carries most"""
        w = np.zeros(len(skin_verts)); np.add.at(w, np.repeat(np.arange(len(skin_verts)), Wd.shape[1]), (Id == b).reshape(-1) * Wd.reshape(-1))
        return skin_verts[w >= 0.5]
    def centred(ph_bone, a, b):
        """a phalanx's knuckle `a` moved across its bone to the middle of the finger there. The default rig lays the
        finger bones along the back of the finger (4-7 mm off the middle of a 10-14 mm finger), so a curl about them
        swung the palm side of each phalanx into the one before it and drove the tips through the palm and out the
        back of the hand; a real finger bends about its knuckle's middle. The middle: half way between the skin's
        extremes across the bone (each of the two ways across it), over the phalanx's own skin in its first third"""
        v = carried(ph_bone) - a; L = np.linalg.norm(b - a); d = (b - a) / L
        t = v @ d; s = v[(t >= 0) & (t <= 0.35 * L)]
        if len(s) < 8:
            return a
        p = s - np.outer(s @ d, d)
        # the two ways across: the way the skin lies off the bone (toward the pad), and across that
        n = p.mean(0); n -= d * (n @ d)
        if np.linalg.norm(n) < 1e-6:
            return a
        n /= np.linalg.norm(n); m = np.cross(d, n)
        off = sum(ax * (float((p @ ax).max()) + float((p @ ax).min())) / 2 for ax in (n, m))
        return a + off
    X, knuckle = [], {}
    for side, S in (("L", "Left"), ("R", "Right")):
        for k, F in enumerate(FING):
            hs = [fheads[FL.index(f"finger{k + 1}-{ph}.{side}")] for ph in (1, 2, 3)]
            # (the thumb's first, its metacarpal's base deep in the ball of the thumb, stays where the rig has it)
            ends = hs[1:] + [2 * hs[2] - hs[1]]
            hs = [hs[ph] if (k == 0 and ph == 0) else centred(FL.index(f"finger{k + 1}-{ph + 1}.{side}"), hs[ph], ends[ph]) for ph in range(3)]
            knuckle.update({f"finger{k + 1}-{ph + 1}.{side}": hs[ph] for ph in range(3)})
            # the tip: the farthest of the last phalanx's skin along it
            d = (hs[2] - hs[1]) / np.linalg.norm(hs[2] - hs[1])
            hs.append(hs[2] + d * float(((carried(FL.index(f"finger{k + 1}-3.{side}")) - hs[2]) @ d).max()))
            for ph in (1, 2, 3):
                f = f"{side}Thumb" if k == 0 else S + ("FingerBase" if ph == 1 else "HandFinger1")
                # its thickness: the median of its skin's distance from the bone
                a, b = hs[ph - 1], hs[ph]; v = carried(FL.index(f"finger{k + 1}-{ph}.{side}")) - a; ab = (b - a) / np.linalg.norm(b - a)
                r = float(np.median(np.linalg.norm(v - np.outer(v @ ab, ab), axis=1)))
                X.append({"name": f"{S}Hand{F}{ph}", "parent": S + "Hand" if ph == 1 else f"{S}Hand{F}{ph - 1}", "follow": f,
                          "head": tpose(f, a), "tail": tpose(f, b), "a": a, "r": r})
    LX = L + [x["name"] for x in X]
    bind_turns += [bind_turns[L.index(x["follow"])] for x in X]
    a_heads = np.concatenate([a_heads, np.array([x["a"] for x in X])])
    names, of = [], {}
    for b, n in enumerate(FL):
        sn, head, drive = joint_for(n)
        if sn not in of:
            of[sn] = len(names); names.append(sn)
            d = LX.index(drive[0])
            frm, to, u = (LX.index(drive[1]), LX.index(drive[2]), drive[3]) if len(drive) > 1 else (d, d, 0.0)
            skin.append({"name": sn, "head": [round(float(x), 5) for x in head], "drive": d, "from": frm, "to": to, "u": u})
        of[n] = of[sn]
    # each vertex's four heaviest skin joints (Anny gives seven bones), renormalised
    dense = np.zeros((len(skin_verts), len(skin)))
    to_skin = np.array([of[n] for n in FL])
    np.add.at(dense, (np.repeat(np.arange(len(skin_verts)), Wd.shape[1]), to_skin[Id].reshape(-1)), Wd.reshape(-1))
    # (the four picked as before the phalanges had joints of their own, by the groups the skin had then: a thumb's,
    # the fingers' first phalanges', their seconds' and thirds'. Each group picked splits into its phalanges while slots
    # are left, and a phalanx that misses one goes to its group's heaviest: unturned, a group's phalanges all move as
    # one, so the skin is the old skin to the vertex; split on the merged groups, 8-12 vertices a body lost a
    # different small weight and moved 1-2 mm)
    grp_name = lambda n: re.sub(r"^(index|middle|ring|pinky)([23])\.", r"fingers2.", re.sub(r"^(index|middle|ring|pinky)1\.", "fingers1.", re.sub(r"^thumb[1-3]\.", "thumb.", n)))
    gn = sorted({grp_name(x["name"]) for x in skin}, key=lambda g: min(k for k, x in enumerate(skin) if grp_name(x["name"]) == g))
    grp = np.array([gn.index(grp_name(x["name"])) for x in skin])
    dg = np.zeros((len(skin_verts), len(gn)))
    for k in range(len(skin)):
        dg[:, grp[k]] += dense[:, k]
    tg = np.argsort(-dg, axis=1)[:, :4]
    tj, tw = np.zeros((len(skin_verts), 4), np.int64), np.zeros((len(skin_verts), 4))
    members = [np.nonzero(grp == g)[0] for g in range(len(gn))]
    for v in range(len(skin_verts)):
        picked = [g for g in tg[v] if dg[v, g] > 0]
        # each picked group's heaviest phalanx first, then the next heaviest of any while slots are left
        own = {g: sorted(members[g], key=lambda k: -dense[v, k]) for g in picked}
        slots = [own[g][0] for g in picked]
        rest_ = sorted([k for g in picked for k in own[g][1:] if dense[v, k] > 0], key=lambda k: -dense[v, k])
        slots += rest_[:4 - len(slots)]
        w = {k: dense[v, k] for k in slots}
        for k in rest_[4 - len(picked):]:
            w[own[grp[k]][0]] += dense[v, k]
        for i, k in enumerate(slots):
            tj[v, i], tw[v, i] = k, w[k]
    tw /= np.maximum(tw.sum(1, keepdims=True), 1e-9)
    tj[tw == 0] = 0
    W, I = model.vertex_bone_weights.numpy().astype(np.float64), model.vertex_bone_indices.numpy().astype(np.int64)
    normals = np.asarray(trimesh.Trimesh(skin_verts, faces, process=False).vertex_normals)
    # each face's graph paper on the pair of axes it faces least along at rest: a set of vertices per projection, so
    # no triangle mixes two
    fn = np.abs(np.cross(skin_verts[faces[:, 1]] - skin_verts[faces[:, 0]], skin_verts[faces[:, 2]] - skin_verts[faces[:, 0]]))
    dom = fn.argmax(1)
    src, uv, idx, base = [], [], [], 0
    for ax, (a, b) in enumerate([(2, 1), (0, 2), (0, 1)]):
        g = faces[dom == ax]
        if len(g) == 0:
            continue
        used = np.unique(g)
        remap = -np.ones(len(skin_verts), dtype=np.int64); remap[used] = np.arange(len(used)) + base
        src.append(used); uv.append(skin_verts[used][:, [a, b]]); idx.append(remap[g])
        base += len(used)
    src, uv, idx = np.concatenate(src), np.concatenate(uv) / (4 * GRID), np.concatenate(idx)
    jn, jw = tj[src].astype(np.uint16), tw[src].astype(np.float32)
    J = len(L)
    K = len(skin)
    # the skin joints' rest: at their heads, turned as the world (so the bind matrices only undo the head's offset)
    inv = np.zeros((K, 16), np.float32)
    for j in range(K):
        M = np.eye(4); M[:3, 3] = -np.array(skin[j]["head"])
        inv[j] = M.T.reshape(-1)  # column-major
    folder = os.path.join(out, name)
    os.makedirs(folder, exist_ok=True)
    tex = paper()
    for kind in ("body", "hull"):
        hull = kind == "hull"
        pos = (skin_verts[src] + (normals[src] * OUTLINE if hull else 0)).astype(np.float32)
        nor = (-normals[src] if hull else normals[src]).astype(np.float32)
        tri = (idx[:, ::-1] if hull else idx).astype(np.uint32)
        chunks = [(pos.tobytes(), 34962), (nor.tobytes(), 34962), (uv.astype(np.float32).tobytes(), 34962),
                  (jn.tobytes(), 34962), (jw.tobytes(), 34962), (tri.tobytes(), 34963), (inv.tobytes(), None)]
        n = len(pos)
        accessors = [
            {"bufferView": 0, "componentType": 5126, "count": n, "type": "VEC3", "min": pos.min(0).tolist(), "max": pos.max(0).tolist()},
            {"bufferView": 1, "componentType": 5126, "count": n, "type": "VEC3"},
            {"bufferView": 2, "componentType": 5126, "count": n, "type": "VEC2"},
            {"bufferView": 3, "componentType": 5123, "count": n, "type": "VEC4"},
            {"bufferView": 4, "componentType": 5126, "count": n, "type": "VEC4"},
            {"bufferView": 5, "componentType": 5125, "count": int(tri.size), "type": "SCALAR"},
            {"bufferView": 6, "componentType": 5126, "count": K, "type": "MAT4"},
        ]
        attrs = {"POSITION": 0, "NORMAL": 1, "JOINTS_0": 3, "WEIGHTS_0": 4}
        if hull:
            material = {"pbrMetallicRoughness": {"baseColorFactor": [c / 255 for c in INK] + [1], "metallicFactor": 0.0, "roughnessFactor": 1.0}}
            extra = {}
        else:
            attrs["TEXCOORD_0"] = 2
            chunks.append((tex, None))
            material = {"pbrMetallicRoughness": {"baseColorTexture": {"index": 0}, "metallicFactor": 0.0, "roughnessFactor": 0.75}}
            extra = {"images": [{"bufferView": 7, "mimeType": "image/png"}], "samplers": [{"magFilter": 9729, "minFilter": 9987, "wrapS": 10497, "wrapT": 10497}],
                     "textures": [{"source": 0, "sampler": 0}]}
        joints = list(range(2, 2 + K))
        nodes = [{"name": kind, "mesh": 0, "skin": 0}, {"name": "skeleton", "children": joints}]
        nodes += [{"name": skin[j]["name"], "translation": skin[j]["head"]} for j in range(K)]
        glb(os.path.join(folder, "body.glb" if not hull else "body.hull.glb"), {
            "scene": 0, "scenes": [{"nodes": [0, 1]}], "nodes": nodes, "accessors": accessors, "materials": [material],
            "meshes": [{"primitives": [{"attributes": attrs, "indices": 5, "material": 0}]}],
            "skins": [{"joints": joints, "inverseBindMatrices": 6, "skeleton": 1}], **extra}, chunks)
    # the hidden model a joint follows: one triangle a millimetre across
    t = np.array([[0, 0, 0], [1e-3, 0, 0], [0, 1e-3, 0]], np.float32)
    glb(os.path.join(folder, "joint.glb"), {"scene": 0, "scenes": [{"nodes": [0]}], "nodes": [{"mesh": 0}],
        "accessors": [{"bufferView": 0, "componentType": 5126, "count": 3, "type": "VEC3", "min": t.min(0).tolist(), "max": t.max(0).tolist()}],
        "meshes": [{"primitives": [{"attributes": {"POSITION": 0}}]}]}, [(t.tobytes(), 34962)])
    at = lambda b: [round(float(x), 5) for x in heads[L.index(b)]]
    # landmarks for the IK: between the eyes, from Anny's own rig (whose eye bones the CMU rig hasn't)
    eyes = (fheads[FL.index("eye.L")] + fheads[FL.index("eye.R")]) / 2
    # how far the hip joint sits over a seat: the back of the upper thigh (whose skin goes mostly with the thigh's
    # joint, a hand's width under the hip) behind the joint at rest is under it when the thigh is level
    hip = heads[L.index("LeftUpLeg")]
    thigh = (I[np.arange(len(verts)), W.argmax(1)] == L.index("LeftUpLeg")) & (verts[:, 1] < hip[1] - 0.03) & (verts[:, 1] > hip[1] - 0.12)
    sit_drop = float(hip[2] - verts[thigh, 2].min())
    R5 = lambda v: [round(float(x), 5) for x in v]
    # the skin's surface, sampled: for each skin joint the rest vertices it carries most (half their weight or more),
    # spread over it (farthest point first), each its offset from the joint's head at rest, its normal and its skin
    # weights as body.glb has them, so a pose's samples are the skinned mesh's own (`surface` in character.tsx)
    part = {"Hips": "hips", "LHipJoint": "hips", "RHipJoint": "hips", "LowerBack": "waist", "Spine": "torso", "Spine1": "chest",
            "Neck": "neck", "Neck1": "neck", "Head": "head", "Shoulder": "shoulder", "Arm": "upperArm", "ForeArm": "foreArm",
            "Hand": "palm", "UpLeg": "thigh", "Leg": "shin", "Foot": "foot", "ToeBase": "toes"}
    surface = []
    for k, sk in enumerate(skin):
        dn = LX[sk["drive"]]
        m = re.match(r"^(Left|Right)Hand([A-Z][a-z]+)(\d)$", dn)
        side = "L" if re.match(r"^(Left|L[A-Z])", dn) else "R" if re.match(r"^(Right|R[A-Z])", dn) else ""
        bare = re.sub(r"^(Left|Right)", "", dn)
        pt = m[2].lower() + m[3] if m else part.get(bare, bare.lower())
        want = SAMPLES.get("finger" if m else bare, 30)
        cand = np.nonzero((tj[:, 0] == k) & (tw[:, 0] >= 0.5))[0]
        if not len(cand):
            continue
        P = skin_verts[cand]
        pick = [int(np.argmax(np.linalg.norm(P - P.mean(0), axis=1)))]
        dist = np.linalg.norm(P - P[pick[0]], axis=1)
        while len(pick) < min(want, len(cand)):
            pick.append(int(np.argmax(dist))); dist = np.minimum(dist, np.linalg.norm(P - P[pick[-1]], axis=1))
        for v in cand[pick]:
            js = [int(j) for j, x in zip(tj[v], tw[v]) if x > 0]
            surface.append({"part": pt, "side": side, "j": js, "w": [round(float(x), 4) for x in tw[v][:len(js)]],
                            "o": [round(float(x), 4) for x in skin_verts[v] - np.array(sk["head"])], "n": [round(float(x), 3) for x in normals[v]]})
    skel = {
        "name": name, "body": body, "height": round(float(verts[:, 1].max()), 4),
        "joints": [{"name": L[j], "parent": parents[j], "head": at(L[j]), "tail": [round(float(x), 5) for x in tails[j]]} for j in range(J)]
                  + [{"name": x["name"], "parent": LX.index(x["parent"]), "head": R5(x["head"]), "tail": R5(x["tail"]),
                      "follow": L.index(x["follow"]), "radius": round(x["r"], 5)} for x in X],
        "landmarks": {"eyes": [round(float(x), 5) for x in eyes], "sitDrop": round(sit_drop, 4)},
        "bind": {"turns": [[round(x, 6) for x in q] for q in bind_turns], "heads": [[round(float(x), 5) for x in h] for h in a_heads]},
        "skin": skin,
        "surface": surface,
    }
    # (a surface sample to a line: indented, the samples alone were 450 KB a body)
    body_js = json.dumps({k: v for k, v in skel.items() if k != "surface"}, indent=1)
    lines = ",\n".join("  " + json.dumps(x, separators=(",", ":")) for x in surface)
    with open(os.path.join(folder, "skeleton.json"), "w") as fh:
        fh.write(body_js[:-2] + ',\n "surface": [\n' + lines + "\n ]\n}")
    print(f"{name}: {skel['height']:.3f} m, {len(LX)} joints, {K} skin joints, {len(surface)} surface samples, {len(idx)} faces -> {folder}")


if __name__ == "__main__":
    args = sys.argv[1:]
    out = args[args.index("--out") + 1]
    args = [a for i, a in enumerate(args) if a != "--out" and (i == 0 or args[i - 1] != "--out")]
    name, sliders = args[0], dict(a.split("=") for a in args[1:])
    model = anny.Anny(rig="cmu_mb", extrapolate_phenotypes=True).to(dtype=torch.float64)
    body = {**{k: 0.5 for k in model.phenotype_labels}, **PRESETS.get(name, {}), **{k: float(v) for k, v in sliders.items()}}
    export(model, name, body, out)
