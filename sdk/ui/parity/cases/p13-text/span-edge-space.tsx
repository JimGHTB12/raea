// A word's space at the end of one span, before the next span, in a centred line: the space stays between them (it
// ends the span, not the row), as in a caption that colours one word ("We're *simulating* them").
// proves: react-dom.element.span, tw.utility.text-center
export const steps = [];
export default function Case() {
  return (
    <div data-testid="root" className="h-full w-full bg-white p-4">
      <div data-testid="card" className="inline-block rounded bg-slate-900 px-8 py-4 text-center text-3xl text-white">
        <div data-testid="lead"><span data-testid="a">We're </span><span data-testid="b" className="text-amber-400">simulating</span><span data-testid="c"> them</span></div>
      </div>
      <div data-testid="wide" className="mt-4 w-96 bg-slate-100 text-center text-xl">
        <span data-testid="d">one </span><b data-testid="e">two </b><span data-testid="f">three</span>
      </div>
    </div>
  );
}
