@spec
Feature: Blender as a measured workshop step: an asset passes through it on its way into media/
  Shane, 2026-11-07: "how do we correctly utilize blender AND how do we correctly prompt in the skill how and when to
  use blender versus our manifold/3d-mesh building in typescript". The 11-06 research (REPORT.md sections 1, 2, 4, 8):
  Claude models badly in bpy (CADClamp: 0.889 in code-CAD, 0.499 in bpy) and well in code-shaped work on assets that
  are already good (rigs, bakes, LODs, batch fixes); a live Blender session is a bad source of truth (five agents on a
  shared MCP stacked their props at the origin; headless won the edit round). So TSX is the scene; solids, sdf and grow
  make what numbers or rules define; raea gen and CC0 libraries make what looks define; and Blender is the workshop
  between an asset's source and media/. It never places, lights, frames, materials or animates a scene.

  Background:
    Given a Blender step is a saved script in the project's blender/ folder, run by `raea blender run`, re-run when
      its source changes; the library ships in sdk/guides/craft/bevy/blender/ and `raea blender run blender/X.py`
      copies a library script there the first time
    And every script takes its arguments after `--`, writes one .glb with the source's licence sidecar beside it
      (MODEL.glb.json), and exits non-zero with the reason when it can't do its job
    And a step is triggered by a measured finding, never by taste: each finding below carries its fix as text
      ("fix: raea blender run blender/X.py ...") and in its JSON (`fix.blender`, `fix.run`)

  Rule: Getting a solid out

    @done
    Scenario: BL1. raea solid export writes a solid as one .glb
      Given a solid file and `raea solid export FILE.ts -o OUT.glb [--set k=v,...] [--part NAME]`
      Then the solid (or the one part) is written as a .glb, metres, y up
      And an empty or non-watertight solid is refused, with solid info's findings printed
      And the .glb read back has the same volume, pieces and box as the solid, within 0.5%

  Rule: The script library

    @done
    Scenario: BL2. convert.py: FBX, OBJ or .blend to one .glb
    @done
    Scenario: BL3. decimate.py: to a triangle budget (a ratio or a count), with LOD copies for far placements
    @done
    Scenario: BL4. repair.py: merged doubles, loose debris dropped, holes filled, normals out, or a voxel remesh
    @done
    Scenario: BL5. split.py: one blob into named parts by boxes, each its own node (a door that opens)
    @done
    Scenario: BL6. bake.py: unwrapped UVs, ambient occlusion baked, and a normal map from a high-poly source
    @done
    Scenario: BL7. rig.py: a bone chain and rigid or automatic weights for a model with no skeleton, and named pose clips
    @done
    Scenario: BL8. cloth.py: a cloth simulated and baked to a morph-target clip
    @done
    Scenario: BL9. Each script has run once on a real asset and its output passes raea gen check

  Rule: Measured triggers

    @done
    Scenario: BL10. Triangles over the budget for the viewing distance name decimate.py
      Given gen check's --seen M (or scene lint's camera distance to the model)
      Then the budget is 150 000 triangles at 1 m or closer, falling with the square of the distance, at least 2 000
    @done
    Scenario: BL11. A wrong piece count names repair.py (generator debris) or split.py (fewer parts than --parts)
    @done
    Scenario: BL12. Open or over-shared edges on a model meant to be solid name repair.py
    @done
    Scenario: BL13. A blob the brief says moves in parts (--parts door,body) names split.py
    @done
    Scenario: BL14. Textures wanted on a solid (its file's `needs.textured`) name solid export then bake.py
    @done
    Scenario: BL15. A model the brief says moves (--rig) with no skin names rig.py
    @done
    Scenario: BL16. A format raea can't load (gen check FILE.fbx, a Model src .obj) names convert.py

  Rule: The skill

    @done
    Scenario: BL17. scenes.md routes by what defines an object and says when it then goes through Blender
    @done
    Scenario: BL18. blender.md is written around the triggers: headless, saved scripts, live MCP to inspect only
    @done
    Scenario: BL19. prompts.md's builder prompt routes and uses Blender the same way

  Rule: Proven

    @done
    Scenario: BL20. A workbench corner uses a solid baked in Blender, a CC0 model converted and decimated, a Tripo
      model split into parts and a model rigged in Blender and posed in raea; every step re-runs from scratch
    @done
    Scenario: BL21. Its review sheets are judged by Kimi and clef every round until Kimi passes or 8 rounds stall (11-07: 8 rounds, stalled; best Kimi round 4)
