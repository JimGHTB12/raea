// Text in the system's faces (Helvetica, Georgia) at line-height normal: each line as tall as the face's metrics say,
// its glyphs where Chrome puts them in it, a box's top margin under a border not collapsing (a slide's card).
// proves: css.properties.line-height, css.properties.font-family, css.properties.font-style.italic
export const steps = [];
export default function Case() {
  return (
    <div data-testid="root" className="h-full w-full bg-white text-slate-900" style={{ lineHeight: "normal" }}>
      <div data-testid="title" style={{ position: "absolute", left: 16, top: 8, fontFamily: "Helvetica", fontWeight: 700, fontSize: 84, background: "#eef" }}>The menu</div>
      <div data-testid="card" style={{ position: "absolute", left: 16, top: 130, width: 360, height: 220, border: "2px solid #ccc", background: "#fffaf0" }}>
        <div data-testid="name" style={{ margin: 24, fontFamily: "Helvetica", fontWeight: 700, fontSize: 60, background: "#efe" }}>Sourdough</div>
        <div data-testid="line" style={{ marginLeft: 24, fontFamily: "Georgia", fontSize: 36, background: "#fee" }}>48-hour ferment</div>
      </div>
      <div data-testid="serif" style={{ position: "absolute", left: 400, top: 130, fontFamily: "Georgia", fontStyle: "italic", fontSize: 56 }}>since 1987</div>
    </div>
  );
}
