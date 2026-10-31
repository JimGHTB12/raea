// Nudge: a part moved by hand on void's picture (docs/craft-system.md, L). The timeline writes a moved caption back
// to the composition as <Nudge x y> around the element that holds its words; a second move changes x and y.
import { cloneElement, isValidElement, type CSSProperties, type ReactElement, type ReactNode } from "react";
import { AbsoluteFill } from "./timeline";

export interface NudgeProps {
  /** how far to move it, in the composition's pixels */
  x?: number;
  y?: number;
  children?: ReactNode;
}

/** Moves its child by (x, y): an element gets the move in front of its own transform; a component, whose box it
 *  can't reach, moves inside a full-frame layer. */
export function Nudge({ x = 0, y = 0, children }: NudgeProps) {
  if (!Number.isFinite(x) || !Number.isFinite(y)) throw new RangeError(`Nudge: x and y must be finite, got ${x}, ${y}`);
  const move = `translate(${x}px, ${y}px)`;
  if (isValidElement(children) && typeof children.type === "string") {
    const style = ((children.props as { style?: CSSProperties }).style ?? {}) as CSSProperties;
    return cloneElement(children as ReactElement<{ style?: CSSProperties }>, { style: { ...style, transform: style.transform ? `${move} ${style.transform}` : move } });
  }
  return <AbsoluteFill style={{ transform: move }}>{children}</AbsoluteFill>;
}
