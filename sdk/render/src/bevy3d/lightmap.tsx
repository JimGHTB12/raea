// Bevy's Lightmap on a <Model>'s meshes, by their glTF mesh's name.
import { useContext } from "react";
import { InModel, node3d } from "../3d/all";

export interface LightmapProps {
  /** the glTF mesh it lies on, by name (Bevy's GltfMeshName): every primitive of it */
  mesh: string;
  /** the bake: a .ktx2 (or other picture) file, staticFile("lightmaps/room.ktx2"), sampled by the mesh's second UVs */
  image: string;
  /** the rect in the image those UVs are relative to, for meshes packed in one atlas ({ min: [0, 0], max: [1, 1] } by
   *  default) */
  uv_rect?: { min: [number, number]; max: [number, number] };
  /** bicubic sampling: smoother, slower, may leak light (false by default) */
  bicubic_sampling?: boolean;
}

/** A light bake on the meshes of a <Model> its `mesh` names, lighting them in place of the scene's lights, scaled by
 *  their material's lightmap_exposure (a <StandardMaterial lightmap_exposure> in the same Model, by material). Goes in
 *  a <Model>. @bevy bevy_pbr::lightmap::Lightmap */
export function Lightmap({ mesh, image, uv_rect, bicubic_sampling }: LightmapProps) {
  if (!useContext(InModel)) throw new TypeError("Lightmap: goes in a <Model>, on the glTF meshes it names (mesh)");
  if (typeof mesh !== "string" || mesh === "") throw new TypeError(`Lightmap: mesh is the glTF mesh's name, got ${JSON.stringify(mesh)}`);
  if (typeof image !== "string" || image === "") throw new TypeError(`Lightmap: image is a picture file (staticFile("lightmaps/x.ktx2")), got ${JSON.stringify(image)}`);
  const r = uv_rect ?? { min: [0, 0], max: [1, 1] };
  const corners = [...(r.min ?? []), ...(r.max ?? [])];
  if (corners.length !== 4 || !corners.every((x) => Number.isFinite(x) && x >= 0 && x <= 1)) throw new RangeError(`Lightmap: uv_rect is { min: [u, v], max: [u, v] } within 0..1, got ${JSON.stringify(uv_rect)}`);
  const [uv_x0, uv_y0, uv_x1, uv_y1] = corners;
  return node3d("Lightmap", { lightmap: image, mesh, bicubic: bicubic_sampling ? 1 : 0, uv_x0, uv_y0, uv_x1, uv_y1 });
}
