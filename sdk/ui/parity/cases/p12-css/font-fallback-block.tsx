// A glyph the line's font lacks (U+258C, a caret in P26 reel see/search): drawn from the same fallback face as Chrome's,
// so the line is as wide.
// proves: css.properties.font-family
export const steps = [];
export default function Case() {
  return (
    <div data-testid="root" className="h-full w-full" style={{ position: "relative", background: "#0b111b" }}>
      <div data-testid="line" style={{ position: "absolute", left: 26, top: 10, fontFamily: "Helvetica", fontSize: 46, fontWeight: 800, color: "#fff", whiteSpace: "nowrap" }}>
        {'"fireworks"'}<span data-testid="caret" style={{ color: "#4ff" }}>{"▌"}</span>
      </div>
    </div>
  );
}
