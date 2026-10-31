// A centred card in a narrow column (as wide as its absolutely placed region): its width is its text's max-content until that's wider than the column, then
// the column's, and the text wraps inside it (fit-content), with an absolutely placed bar sized to the card.
// proves: css.properties.width, css.properties.align-items
export const steps = [];
export default function Case() {
  return (
    <div data-testid="root" className="relative h-full w-full bg-white p-4 text-slate-900">
      <div data-testid="region" className="absolute" style={{ left: 16, top: 16, width: 360, height: 400 }}>
      <div data-testid="col" className="flex flex-col items-center gap-4 bg-slate-100 text-center text-2xl font-extrabold">
        <div data-testid="short" className="bg-slate-800 px-6 py-2 text-white">Short one</div>
        <div data-testid="plain" className="bg-slate-800 px-10 py-4 text-white">Automating our creatives</div>
        <div data-testid="card" className="relative bg-slate-800 px-10 py-4 text-white">
          <span data-testid="span"><div data-testid="line">Automating <span className="text-amber-400">our</span> creatives</div></span>
        </div>
        <div data-testid="card2" className="relative bg-slate-800 px-10 py-4 text-white">
          <span><div data-testid="line2"><span>Automating our creatives</span></div></span>
          <div data-testid="bar" className="absolute left-6 right-6 top-1/2 h-2 bg-amber-400" />
        </div>
      </div>
      </div>
    </div>
  );
}
