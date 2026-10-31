// Characters the line's fonts lack, as desktop Chrome on macOS draws them: the font list, then the standard face
// (Times: π), then the platform's pick for the first character a text item still lacks, which takes every later one it
// has, across spans and spaces of the same font (▌ then → after Helvetica Bold are both Hiragino Sans, though → alone
// is Lucida Grande's). After a monospaced face, Menlo's.
// proves: css.properties.font-family
export const steps = [];
export default function Case() {
  return (
    <div data-testid="root" className="h-full w-full bg-white" style={{ fontSize: 40 }}>
      <div style={{ fontFamily: "Helvetica", fontWeight: 800 }}><span data-testid="h0">▌</span><span data-testid="h1">→</span></div>
      <div style={{ fontFamily: "Helvetica", fontWeight: 800 }}><span data-testid="h2">▌</span> <span data-testid="h3">→</span></div>
      <div style={{ fontFamily: "Helvetica", fontWeight: 800 }}><span data-testid="h4">→</span> <span data-testid="h5">▌</span> <span data-testid="h6">✓</span></div>
      <div style={{ fontFamily: "Parity Mono" }}><span data-testid="m0">π → ✓</span></div>
      <div style={{ fontFamily: "Parity Mono" }}><span data-testid="m1">a ✓ ★ ⌘ ▾ ❯ ∞</span></div>
    </div>
  );
}
