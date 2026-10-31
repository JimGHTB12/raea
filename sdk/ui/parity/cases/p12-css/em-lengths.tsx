// Lengths in em, of the element's own font size: a gap between words (a caption's), a margin and a padding.
// proves: css.types.length.em, css.properties.column-gap, css.properties.margin
export const steps = [];
export default function Case() {
  return (
    <div data-testid="root" className="h-full w-full bg-white p-4 text-slate-900">
      <div data-testid="words" className="flex gap-x-[0.5em] bg-slate-100" style={{ fontSize: 32 }}>
        <span data-testid="w1" className="bg-amber-200">one</span>
        <span data-testid="w2" className="bg-amber-200">two</span>
      </div>
      <div data-testid="boxed" className="mt-[1.5em] bg-sky-100" style={{ fontSize: 20, padding: "0.5em", marginLeft: "2em" }}>padded</div>
    </div>
  );
}
