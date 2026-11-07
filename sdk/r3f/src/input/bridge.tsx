// React contexts carried across into React Three Fiber's own root (its reconciler is a second React renderer, and a
// context doesn't cross from one renderer to another): every context provided above the <R3F> is found on its fiber's
// ancestors, read, and provided again inside, as R3F's <Canvas> does with its-fine. So raea's hooks (useCurrentFrame,
// useGameState, a theme) work in the R3F scene's components as they do around it.
import { Component, use, type Context, type ReactNode } from "react";

interface Fiber {
  return: Fiber | null;
  type: unknown;
  memoizedProps: unknown;
}

const CONTEXT = Symbol.for("react.context");
const PROVIDER = Symbol.for("react.provider");

/** the contexts provided above a fiber, outermost first */
function contextsAbove(fiber: Fiber | null): Context<unknown>[] {
  const out: Context<unknown>[] = [];
  const seen = new Set<unknown>();
  for (let f = fiber?.return ?? null; f; f = f.return) {
    const t = f.type as { $$typeof?: symbol; _context?: Context<unknown> } | null;
    if (!t || typeof t !== "object") continue;
    // React 19: <Ctx value> is the context itself; React 18: Ctx.Provider points at it
    const ctx = t.$$typeof === CONTEXT ? (t as unknown as Context<unknown>) : t.$$typeof === PROVIDER ? t._context : undefined;
    if (ctx && !seen.has(ctx)) {
      seen.add(ctx);
      out.unshift(ctx);
    }
  }
  return out;
}

/** hands its child its own fiber, the place the context walk starts from */
export class FiberOf extends Component<{ children: (fiber: Fiber | null) => ReactNode }> {
  render() {
    return this.props.children((this as unknown as { _reactInternals?: Fiber })._reactInternals ?? null);
  }
}

/** a function that wraps a node in every context above `fiber`, each with the value it has here; call it in render */
export function useBridge(fiber: Fiber | null): (node: ReactNode) => ReactNode {
  const ctxs = contextsAbove(fiber);
  const values = ctxs.map((c) => use(c));
  return (node) => ctxs.reduceRight<ReactNode>((inner, C, i) => <C value={values[i]}>{inner}</C>, node);
}
