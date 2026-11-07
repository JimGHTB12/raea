// raea's parts as React elements: each Part (a raea/3d component's name and its props, what the R3F input makes and
// the source export writes) drawn by that component. One table, so the input, the export and the tests name the same
// components.
import type { ReactNode } from "react";
import {
  AmbientOcclusion, AntiAliasing, Bloom, BufferMesh, DepthOfField, Environment, Fog, Gizmo, Grade, Instances, LensDistortion, Look, RichText, ToneMapping,
} from "raea/3d";
import {
  AmbientLight, DirectionalLight, Group, Mesh, MeshBasicMaterial, MeshPhysicalMaterial, MeshStandardMaterial,
  MeshToonMaterial, OrthographicCamera, PerspectiveCamera, PointLight, RectAreaLight, SpotLight, ThreeShaderMaterial,
} from "@raea/render/three";
import type { Part } from "./input/material";

export type { Part, Props } from "./input/material";

/** every component a Part may name */
// eslint-disable-next-line @typescript-eslint/no-explicit-any
export const COMPONENTS: Record<string, (p: any) => ReactNode> = {
  AmbientLight, AmbientOcclusion, AntiAliasing, Bloom, BufferMesh, DepthOfField, DirectionalLight, Environment, Fog, Gizmo, Grade, Group, Instances,
  LensDistortion, Look, Mesh, MeshBasicMaterial,
  MeshPhysicalMaterial, MeshStandardMaterial, MeshToonMaterial, OrthographicCamera, PerspectiveCamera, PointLight,
  RectAreaLight, RichText, SpotLight, ThreeShaderMaterial, ToneMapping,
};

/** parts as elements, keyed by their place */
export function elementsOf(parts: Part[], at = ""): ReactNode[] {
  return parts.map((p, i) => {
    const C = COMPONENTS[p.type];
    if (!C) throw new TypeError(`raea/r3f: no raea component ${p.type}`);
    const key = `${at}${i}`;
    return p.children ? <C key={key} {...p.props}>{elementsOf(p.children, `${key}.`)}</C> : <C key={key} {...p.props} />;
  });
}
