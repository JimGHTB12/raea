// text-pretty and text-balance: Chrome's score line breaker. Pretty pulls a word down to a one-word last line
// (the paragraph's last 4 lines only); balance evens a short paragraph's lines out.
// proves: css.properties.text-wrap-style, tw.utility.text-pretty, tw.utility.text-balance
export const steps = [];
export default function Case() {
  return (
    <div data-testid="root" className="flex h-full w-full flex-col gap-3 bg-white p-4 text-slate-900">
      <p data-testid="greedy" className="w-64 bg-slate-100 text-sm">Pretty moves a word down so that a paragraph never ends on one.</p>
      <p data-testid="pretty" className="w-64 bg-slate-100 text-sm text-pretty">Pretty moves a word down so that a paragraph never ends on one.</p>
      <p data-testid="pretty-long" className="w-64 bg-slate-100 text-sm text-pretty">One two three four five six seven eight nine ten eleven twelve thirteen fourteen fifteen sixteen seventeen eighteen nineteen twenty and one.</p>
      <h2 data-testid="balance" className="w-80 bg-slate-100 text-2xl font-semibold text-balance">A heading that would leave a short last line</h2>
      <p data-testid="balance-center" className="w-72 bg-slate-100 text-center text-base text-balance">Centred and balanced across three lines of ordinary words in a narrow box.</p>
    </div>
  );
}
