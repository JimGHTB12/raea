# Blender: fixing an asset before it goes in a scene

## When Blender, when raea

- **raea** makes anything that renders or plays: the scene, its camera, lights, motion, materials, people and games.
  Build those in TSX (the other `bevy/` topics), never in Blender.
- **Blender** fixes an asset on its way in: a model at the wrong scale or with its origin off to one side, a mesh too
  heavy to draw, a format raea can't load (FBX, OBJ, .blend), a rig to retarget, a texture or light to bake into a
  mesh. One asset at a time, out to a `.glb` under the project's `media/`, then into a scene with `<Model>`
  (`bevy/assets`).
- Never build a whole scene in Blender and export it; the scene belongs in TSX, where it's frame-pure and checked
  (`raea scene lint`).

## Setting it up — HEADLESS

- `raea setup blender` finds a Blender (`git config --global raea.blender PATH`, `/Applications/Blender.app`, then
  `blender` on PATH) or downloads the current LTS from blender.org into `~/.raea/tools/blender`, checked against
  blender.org's sha256 list. raea never bundles Blender: it's GPL, so it runs as its own program.
- The same command installs the MCP for Blender add-on into that Blender (it needs uv's `uvx`).
  `raea setup blender --check` only reports.

## Headless: `raea blender run` — HEADLESS

`raea blender run SCRIPT.py [ARGS...]` runs the script in a windowless Blender (`--background --factory-startup`);
the script reads its own arguments after `--` in `sys.argv`, and an uncaught Python error makes the command fail.
Use it for every fix an agent can describe in code: convert, clean, decimate, set scale and pivot, re-rig, bake.

## Live: the `blender` MCP server — BLENDER

`raea skills install --agent claude|codex|kimi|gemini` registers a `blender` MCP server for the agent
(`uvx mcp-for-blender`). Its tools (look, scene info, run Python, search and import Poly Haven, Sketchfab and
Poly Pizza assets, generate a model) need a Blender running with its window open: tag such a step **BLENDER**, and
give the headless `raea blender run` way beside it. `generate_3d` can cost the person a credit; ask first.
In the raea app, a workspace gives it to the feed's agents with a controller's servers field holding
`uvx mcp-for-blender` (`raea controller set PATH VALUE`).

## Licences

Only permissive assets (CC0, CC BY with its credit, MIT). Write the source, licence and credit beside the `.glb`
(`chair.glb.json`: `{"source": URL, "license": "CC0", "credit": ""}`), and keep them through every fix.

## Common mistakes

- Building the scene in Blender: export one asset, build the scene in TSX.
- Judging scale by eye: read the model's bounds in the script and scale to a size in metres (a chair seat is
  0.45 m high), with the origin at the bottom centre so it stands on the floor at `position={[x, 0, z]}`.
- Hardcoding shader node names: they're localized; find nodes by `type` (`BSDF_PRINCIPLED`).
- Exporting `.gltf` with loose files: export `.glb`, one file.
- Moving a rigged model's mesh: the example is for a static model. A rigged one moves with its armature, so scale
  and place the armature (the mesh's parent) and apply its transform, or the skin pulls the mesh back.

## Example: scale and pivot fixed, out as one .glb

`raea blender run fix.py media/chair.fbx media/chair.glb 0.9` imports the file, makes it 0.9 m tall, puts its origin
at the bottom centre and exports a `.glb`:

```python
import bpy, sys
src, out, height = sys.argv[sys.argv.index("--") + 1:][:3]
bpy.ops.wm.read_factory_settings(use_empty=True)
ext = src.lower().rsplit(".", 1)[-1]
load = {"fbx": bpy.ops.import_scene.fbx, "obj": bpy.ops.wm.obj_import, "glb": bpy.ops.import_scene.gltf,
        "gltf": bpy.ops.import_scene.gltf}.get(ext)
load(filepath=src)
objs = [o for o in bpy.context.scene.objects if o.type == "MESH"]
bpy.ops.object.select_all(action="DESELECT")
for o in objs:
    o.select_set(True)
bpy.context.view_layer.objects.active = objs[0]
bpy.ops.object.join()
o = bpy.context.active_object
bpy.ops.object.transform_apply(location=True, rotation=True, scale=True)
s = float(height) / o.dimensions.z
o.scale = (s, s, s)
bpy.ops.object.transform_apply(scale=True)
xs, ys, zs = zip(*[o.matrix_world @ v.co for v in o.data.vertices])
bpy.context.scene.cursor.location = ((min(xs) + max(xs)) / 2, (min(ys) + max(ys)) / 2, min(zs))
bpy.ops.object.origin_set(type="ORIGIN_CURSOR")
o.location = (0, 0, 0)
bpy.ops.export_scene.gltf(filepath=out, use_selection=True, export_format="GLB")
print("wrote", out, [round(d, 3) for d in o.dimensions])
```

Blender's Z is up and glTF's Y is up; the exporter converts, so the model stands upright in raea.
