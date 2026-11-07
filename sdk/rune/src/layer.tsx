// A Rune layer in a page (docs/targets.feature T6c, per layer): what's inside is drawn by Rune on Bevy's device, in this
// element's box, and the rest of the page stays Blitz's. The page host splits the op stream there
// (engine/rune/layer, engine/rune/slots); a browser or a host without Rune layers draws it as a plain div.
import type { HTMLAttributes, ReactElement } from "react";

/** The prop that marks a layer, and its value (engine/rune/layer's MARK). */
export const RUNE_LAYER = { "data-raea-renderer": "rune" } as const;

/** Its children drawn by Rune. Give it a size (className or style): the layer is its content box. Their events reach
 *  React as Blitz's do; their boxes aren't in the page's layout snapshot (a getBoundingClientRect inside is zero). */
export function RuneLayer({ children, ...props }: HTMLAttributes<HTMLDivElement>): ReactElement {
  return (
    <div {...props} {...RUNE_LAYER}>
      {children}
    </div>
  );
}
