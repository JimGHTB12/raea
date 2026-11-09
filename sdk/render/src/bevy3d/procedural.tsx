// raea's procedural surface as a Bevy MaterialExtension (systems/procmat): an ExtendedMaterial's `extension`, its fields
// snake_case as Bevy's are, turned into the old material's `procedural` prop (overlays.ts ProceduralSurface).
import type { ReactNode } from "react";
import type { ProceduralSurface } from "../overlays";

/** raea's extension of StandardMaterial that draws a pattern on the GPU in place of its flat colour (the engine's
 *  Procedural MaterialExtension, systems/procmat): `pattern` (noise, wood, marble, stone, brick, tiles, fabric,
 *  brushed, plaster) in `colors` with roughness `rough_a`/`rough_b`, features `scale` metres across, a `bump`; brick
 *  and tiles in `tile` [w, h] metre cells with `gap` metre joints in `gap_color`. Over it, each 0-1: `dust` on what
 *  faces up, `dirt` in joints, hollows, undersides and the floor line, `edges` worn lighter. The pattern overrides the
 *  base's colour and roughness; its lighting stays. It goes in an ExtendedMaterial's `extension`, alone. */
export interface ProceduralExtensionProps {
  /** what it draws: noise, wood, marble, stone, brick, tiles, fabric, brushed or plaster */
  pattern: ProceduralSurface["pattern"];
  /** the pattern's two colours */
  colors?: [string, string];
  /** how many metres across its features are */
  scale?: number;
  /** the first colour's roughness, 0..1 */
  rough_a?: number;
  /** the second colour's roughness, 0..1 */
  rough_b?: number;
  /** how deep the pattern's relief looks (0 or more) */
  bump?: number;
  /** brick and tiles: a cell's [width, height] in metres */
  tile?: [number, number];
  /** brick and tiles: the joints' width in metres */
  gap?: number;
  /** the joints' colour */
  gap_color?: string;
  /** 0..1 dust on what faces up */
  dust?: number;
  /** 0..1 dirt in joints, hollows, undersides and along the floor line */
  dirt?: number;
  /** 0..1 wear on the edges, lighter */
  edges?: number;
  /** the dust's colour */
  dust_color?: string;
  /** the dirt's colour */
  dirt_color?: string;
  /** which randomness the pattern uses */
  seed?: number;
}

/** raea's procedural surface over a StandardMaterial, as an ExtendedMaterial's extension (systems/procmat):
 *
 *    <ExtendedMaterial base={<StandardMaterial />} extension={<ProceduralExtension pattern="brick" dirt={0.3} />} />
 *
 * @bevy bevy_pbr::mesh_material::MeshMaterial3d<bevy_pbr::extended_material::ExtendedMaterial<bevy_pbr::pbr_material::StandardMaterial, raea_sys_procmat::Procedural>> */
export function ProceduralExtension(_: ProceduralExtensionProps): ReactNode {
  throw new TypeError('ProceduralExtension goes in an ExtendedMaterial: <ExtendedMaterial base={<StandardMaterial />} extension={<ProceduralExtension pattern="wood" />} />');
}

/** the old material's `procedural` from ProceduralExtension's fields */
export function proceduralOf(p: ProceduralExtensionProps): ProceduralSurface {
  const { rough_a, rough_b, gap_color, dust_color, dirt_color, ...rest } = p;
  const named = { roughA: rough_a, roughB: rough_b, gapColor: gap_color, dustColor: dust_color, dirtColor: dirt_color };
  return { ...rest, ...Object.fromEntries(Object.entries(named).filter(([, v]) => v !== undefined)) };
}
