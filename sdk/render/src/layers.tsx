// A page element inside an entity (docs/sdk.feature S3a, one tree): lowercase JSX in a <Mesh3d> or an <Entity> is a
// page layer in the world (B3 H5's world mode), drawn as a flat <Screen> at the entity; in a <Mesh3d>, layer="surface"
// draws it on the mesh itself, over its UVs (H5's surface mode).
//
//   <Mesh3d transform={Transform.fromXyz(0, 1, 0)}>
//     <Cuboid />
//     <div style={{ width: 320, height: 200 }}>HELLO</div>    a 3.2 m panel at the cube, facing its +Z
//   </Mesh3d>
//   <Mesh3d><Cuboid /><div layer="surface" style={{ width: 256, height: 256 }}>HELLO</div></Mesh3d>    on each face
import { Children, createElement, isValidElement, type ImgHTMLAttributes, type ReactElement, type ReactNode } from "react";
import { Screen, screenContent, screenSpec } from "./screen";

/** Page pixels per metre of a page layer in the world: a 320 px wide element is a 3.2 m panel. */
export const LAYER_PX_PER_M = 100;

/** An entity's children with each page element among them (lowercase JSX: a div, an img) made a page layer in the
 * world (docs/sdk.feature S3a, B3 H5's world mode): a flat <Screen> at the entity, facing its +Z, drawn at the
 * element's own px width and height (its style's, as numbers) and LAYER_PX_PER_M px to the metre. */
export function pageLayers(owner: string, id: string, children: ReactNode): ReactNode {
  return meshLayers(owner, id, children, false).children;
}

/** pageLayers for a mesh: also a page element with layer="surface" (at most one), drawn on the mesh over its UVs at
 * the element's px size: `surface` is the screen's spec for the mesh's row and `part` its page part, beside the mesh.
 * Its background is the page's (white, unless it gives its own). */
export function meshLayers(owner: string, id: string, children: ReactNode, mesh: boolean): { children: ReactNode; surface?: object; part?: ReactNode } {
  let n = 0;
  let surface: object | undefined;
  let part: ReactNode;
  const out = Children.map(children, (child) => {
    if (!isValidElement(child) || typeof child.type !== "string") return child;
    const el = child as ReactElement<{ style?: Record<string, unknown>; layer?: string }>;
    const { layer = "world", ...props } = el.props;
    if (layer !== "world" && !(layer === "surface" && mesh)) throw new TypeError(`${owner}: a page element's layer is "world" (a quad at the entity)${mesh ? ' or "surface" (on the mesh, over its UVs)' : ""}, got ${JSON.stringify(layer)}${layer === "surface" ? "; a surface needs a mesh: put it in a <Mesh3d>" : ""}`);
    const w = el.props.style?.width;
    const h = el.props.style?.height;
    if (!(typeof w === "number" && w > 0 && typeof h === "number" && h > 0)) throw new TypeError(`${owner}: a page element inside it is a layer in the world and needs its size in px: style={{ width: 320, height: 200 }}, got ${JSON.stringify({ width: w, height: h })}`);
    if (layer === "surface") {
      if (surface) throw new TypeError(`${owner}: one page element per mesh has layer="surface"`);
      if (!(Number.isInteger(w) && Number.isInteger(h))) throw new RangeError(`${owner}: a surface's page is whole px, got ${w} x ${h}`);
      const name = `${id}:surface`;
      const src = { resolution: [w, h] as [number, number], crt: false, background: "#ffffff", children: createElement(el.type, props) };
      surface = screenSpec(owner, name, src);
      part = screenContent(name, src);
      return null;
    }
    return (
      <Screen name={`${id}:${n++}`} crt={false} background="transparent" resolution={[w, h]} size={[w / LAYER_PX_PER_M]}>
        {createElement(el.type, props)}
      </Screen>
    );
  });
  return { children: out, surface, part };
}

/** A Bevy texture in an element (B3 H7): `texture` is the id a system lists an Image under in the App's PageTextures (a
 * camera's target, a video's frame), drawn at the element's box GPU to GPU. The other props are the <img>'s, its style
 * placing and sizing it.
 *
 *   <TextureView texture={7} style={{ position: "absolute", left: 100, top: 50, width: 200, height: 150 }} />
 */
export function TextureView({ texture, ...img }: { texture: number } & Omit<ImgHTMLAttributes<HTMLImageElement>, "src">) {
  return <img data-raea-texture={texture} alt="" {...img} />;
}
