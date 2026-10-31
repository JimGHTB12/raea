// One paragraph of text and inline elements (CSS's inline formatting context): a span breaks mid-line, a padded,
// rounded chip keeps its padding in the line, elements nest, and a centred line of spans is centred as one.
// proves: react-dom.element.span, tw.utility.inline, tw.utility.text-center
export const steps = [];
export default function Case() {
  return (
    <div data-testid="root" className="h-full w-full bg-white p-4 text-slate-900">
      <p data-testid="wrap" className="w-64 text-base">
        A sentence with <span data-testid="long" className="text-rose-600">a coloured phrase long enough to wrap across lines</span> and more after it.
      </p>
      <p data-testid="chips" className="mt-4 w-80 text-sm">
        Run <code data-testid="chip" className="rounded bg-slate-200 px-1.5 py-0.5">npm test</code> then <b data-testid="bold">bold <i data-testid="nested">and italic</i> words</b> to finish.
      </p>
      <p data-testid="centred" className="mt-4 w-96 text-center text-lg">
        <span data-testid="c1">Centred</span> <em data-testid="c2">lines of</em> <strong data-testid="c3">spans that wrap onto a second line</strong>
      </p>
    </div>
  );
}
