// void's chat message: a flex row capped at 85% of its column (border-box, as Tailwind's preflight has it), aligned to
// the end, holding one block of 13 px text in void's face stack (-apple-system first). In Chrome on macOS the line is
// SF, wider than the cap, and wraps to two lines. Blitz read -apple-system and BlinkMacSystemFont as unknown names and
// fell to Helvetica Neue, narrower, so the line fit on one and the transcript was too short to scroll.
// proves: css.properties.font-family, css.properties.flex-shrink, css.properties.max-width
export const steps = [];
const face = '-apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, "Helvetica Neue", Arial, sans-serif';
export default function Case() {
  return (
    <div data-testid="root" className="relative h-full w-full bg-white">
      <div data-testid="column" style={{ display: "flex", flexDirection: "column", width: 339, gap: 8, padding: 14, boxSizing: "border-box", background: "#f1f5f9", fontFamily: face }}>
        <div data-testid="message" style={{ display: "flex", alignSelf: "flex-end", maxWidth: "85%", flexShrink: 0, boxSizing: "border-box", padding: "7px 11px", borderRadius: 12, background: "#cbd5e1" }}>
          <div data-testid="text" style={{ display: "block", fontSize: 13, lineHeight: 1.5 }}>line 1 of a transcript long enough to scroll</div>
        </div>
        <div data-testid="short" style={{ display: "flex", alignSelf: "flex-start", maxWidth: "95%", flexShrink: 0, padding: "4px 0" }}>
          <div data-testid="short-text" style={{ display: "block", fontSize: 13, lineHeight: 1.5 }}>ok</div>
        </div>
      </div>
    </div>
  );
}
