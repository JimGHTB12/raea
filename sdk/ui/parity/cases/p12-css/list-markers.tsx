// Outside list markers: an ol's numbers and a ul's discs sit a gap before their item's text (Chrome's "1. " keeps its
// space) in their ::marker colour, for plain items and for items holding a p as ProseMirror makes them. The items'
// text is drawn transparent and the lists are narrow, so the markers are most of what each list shows.
// proves: css.properties.list-style-position.outside, css.properties.list-style-type, css.selectors.marker, tw.utility.list-*
export const steps = [];
const li = "[&_li::marker]:text-rose-600 [&_li]:text-transparent";
export default function Case() {
  return (
    <div data-testid="root" className="h-full w-full bg-white p-4 text-2xl text-slate-900">
      <ol data-testid="ol" className={`w-20 list-decimal pl-10 ${li}`}>
        <li>I</li>
        <li>
          <p className="my-1">I</p>
        </li>
      </ol>
      <ol data-testid="ol-shown" className="mt-3 w-28 list-decimal pl-10">
        <li>Cut</li>
      </ol>
      <ul data-testid="ul" className={`mt-3 w-20 list-disc pl-10 ${li}`}>
        <li>I</li>
        <li>
          <p className="my-1">I</p>
        </li>
      </ul>
    </div>
  );
}
