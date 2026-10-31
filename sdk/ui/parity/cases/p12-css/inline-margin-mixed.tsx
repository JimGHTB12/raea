// An inline icon with a right margin before larger text on one line (P26 reel see/search's search bar): the margin
// kept and the two sizes on one baseline, as Chrome lays them out.
// proves: css.properties.margin-right, css.properties.vertical-align
export const steps = [];
export default function Case() {
  return (
    <div data-testid="root" className="h-full w-full" style={{ position: "relative", background: "#0b111b" }}>
      <div data-testid="line" style={{ position: "absolute", left: 26, top: 10, fontFamily: "Helvetica", fontSize: 46, fontWeight: 800, color: "#fff", whiteSpace: "nowrap" }}>
        <span data-testid="icon" style={{ color: "#4ff", fontFamily: "Menlo", fontSize: 34, marginRight: 18 }}>{"Q"}</span>{'"fireworks at night"'}<span style={{ display: "inline-block", width: 14, height: 44, background: "#4ff" }} />
      </div>
    </div>
  );
}
