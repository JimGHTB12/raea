// symbol and use: one symbol (its own viewBox) placed three times at different sizes, and a use of a plain group with
// a fill it inherits (currentColor from the svg's text colour).
// proves: svg.elements.symbol, svg.elements.use, svg.elements.defs
export const steps = [];
export default function Case() {
  return (
    <div data-testid="root" className="h-full w-full bg-white p-4">
      <svg data-testid="svg" width={400} height={160} className="text-rose-600">
        <defs>
          <symbol id="u-star" viewBox="0 0 20 20"><path d="M10 1 L12.6 7 L19 7.5 L14 11.8 L15.6 18.5 L10 15 L4.4 18.5 L6 11.8 L1 7.5 L7.4 7 Z" fill="#f59e0b" /></symbol>
          <g id="u-dot"><circle r={14} fill="currentColor" /></g>
        </defs>
        <use href="#u-star" x={0} y={0} width={40} height={40} />
        <use href="#u-star" x={50} y={0} width={80} height={80} />
        <use href="#u-star" x={140} y={0} width={140} height={140} />
        <use href="#u-dot" x={320} y={40} />
        <use href="#u-dot" x={360} y={100} />
      </svg>
    </div>
  );
}
