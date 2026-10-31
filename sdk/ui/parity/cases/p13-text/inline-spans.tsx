// Inline elements inside text: spans with their own colour and a code span, on one line that wraps as one.
// proves: react-dom.element.span, tw.utility.inline
// tier: quick
export const steps = [];
export default function Case() {
  return (
    <div data-testid="root" className="h-full w-full bg-white p-4 text-slate-900">
      <p data-testid="p" className="w-72 text-sm">Some <span data-testid="red" className="text-rose-600">red words</span> and <code data-testid="code" className="bg-slate-200">code()</code> inside a sentence that wraps.</p>
    </div>
  );
}
