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
  pattern: ProceduralSurface["pattern"];
  colors?: [string, string];
  scale?: number;
  rough_a?: number;
  rough_b?: number;
  bump?: number;
  tile?: [number, number];
  gap?: number;
  gap_color?: string;
  dust?: number;
  dirt?: number;
  edges?: number;
  dust_color?: string;
  dirt_color?: string;
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
