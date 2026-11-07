// the shape module tools/shape-wasm.sh builds (wasm/, not in git), aliased by the page's host
declare module "@raea-shape-web" {
  export default function init(input?: unknown): Promise<unknown>;
  export function mesh(expr: string): Float32Array;
}
