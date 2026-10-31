// line-height: normal takes the face's own ascent, descent and line gap, not a fixed multiple.
// proves: css.properties.line-height
export const steps = [];
export default function Case() {
  return (
    <div data-testid="root" className="h-full w-full bg-white p-4 text-slate-900" style={{ lineHeight: "normal" }}>
      <div data-testid="big" className="bg-slate-100" style={{ fontSize: 64 }}>Normal</div>
      <div data-testid="small" className="w-40 bg-slate-100" style={{ fontSize: 14 }}>Some words that wrap onto lines</div>
    </div>
  );
}
