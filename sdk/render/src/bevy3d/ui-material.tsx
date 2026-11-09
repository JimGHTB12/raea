// bevy_ui_render's MaterialNode (docs/control.feature FC2a, Bevy's ui/ui_material): a node drawn by a fragment shader
// of the author's, as Bevy's UiMaterial pipeline runs it. The shader file is compiled as written (its `@group(1)`
// bindings renumbered into raea's slots: up to 8 uniforms of any type to 256 bytes, 4 textures and their samplers,
// raea-sys-shader's ui_bind.rs) and reads UiVertexOutput as in Bevy: uv, size, border widths and corner radii. The
// engine draws it into a picture of the node's border box, which the page shows in the node's place in its paint
// order: over what's under the node, under its children. It needs the composition's <Canvas3D> (its engine draws it).
import { createElement, type ReactNode } from "react";
import { Node, type NodeProps } from "./ui";
import { checkVal, type BorderRadius } from "./ui-val";

type Uniform = number | number[] | string;

export interface MaterialNodeProps extends NodeProps {
  /** the fragment shader's file, as UiMaterial::fragment_shader names one: staticFile("shaders/x.wesl") */
  fragment: string;
  /** each `var<uniform>` the shader declares, by name: a number, [x, y], [x, y, z], [x, y, z, w] (a LinearRgba's
   *  to_f32_array()) or "#rrggbb"; a struct's as {field: value} */
  uniforms?: Record<string, Uniform | Record<string, Uniform>>;
  /** each `texture_2d<f32>` it declares, by name: a picture file (staticFile), sampled with Bevy's default sampler */
  textures?: Record<string, string>;
  /** shader defs, as a Material's */
  defs?: Record<string, boolean | number>;
  /** the nodes inside it */
  children?: ReactNode;
}

const SIDES = ["left", "top", "right", "bottom"] as const;
const CORNERS = ["top_left", "top_right", "bottom_right", "bottom_left"] as const;

/** A node its own fragment shader draws (MaterialNode(handle) with a UiMaterial asset in Bevy).
 *  @bevy bevy_ui_render::ui_material::MaterialNode */
export function MaterialNode({ fragment, uniforms = {}, textures = {}, defs, children, ...node }: MaterialNodeProps) {
  const who = "MaterialNode";
  if (typeof fragment !== "string" || !fragment) throw new TypeError(`${who}: fragment is the shader file, staticFile("shaders/x.wesl"), got ${JSON.stringify(fragment)}`);
  // its border widths in px, as the shader reads them (UiVertexOutput.border_widths); a percent is of the parent's
  // width in Bevy, which the engine's own node can't know
  const border = SIDES.map((s) => {
    const v = node.border ? checkVal(who, `border.${s}`, node.border[s]) : "0px";
    if (v !== "0" && !v.endsWith("px")) throw new TypeError(`${who}: border.${s} in px is what a MaterialNode takes for now, got ${v}`);
    return parseFloat(v) || 0;
  });
  // its corners as Vals, resolved by Bevy against the node itself (CornerRadius::resolve), each [x, y]
  const radius = CORNERS.map((c) => corner(who, node.border_radius, c));
  const spec = JSON.stringify({ fragment, uniforms, textures, ...(defs ? { defs } : {}), border, radius });
  const [l, t, r, b] = border;
  // the picture over the node's border box, under its children: absolute, out of their layout
  const picture = createElement("div", { key: "material", "data-raea-ui-material": spec, style: { position: "absolute", left: `${-l}px`, top: `${-t}px`, width: `calc(100% + ${l + r}px)`, height: `calc(100% + ${t + b}px)`, pointerEvents: "none" } });
  return (
    <Node {...node}>
      {picture}
      {children}
    </Node>
  );
}

function corner(who: string, r: BorderRadius | undefined, c: (typeof CORNERS)[number]): [string, string] {
  const v = r?.[c];
  if (v === undefined) return ["0px", "0px"];
  if (typeof v === "object" && v !== null) return [checkVal(who, `border_radius.${c}.x`, v.x), checkVal(who, `border_radius.${c}.y`, v.y)];
  return [checkVal(who, `border_radius.${c}`, v), "auto"];
}
