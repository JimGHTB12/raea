// A stylesheet's nested rule and a class inside a class: `.left { & .dot { … } }` and `.card .label { … }` style an
// element under one with the outer class, and nothing outside it.
// proves: css.selectors.nesting, css.selectors.descendant, css.selectors.class
export const steps = [];
const css = `.left { position: absolute; left: 0; top: 0; width: 200px; height: 200px; background: rgb(255, 0, 0);
  & .dot { position: absolute; left: 80px; top: 80px; width: 40px; height: 40px; background: rgb(255, 255, 0); } }
.card .label { color: rgb(37, 99, 235); font-weight: 700; }`;
export default function Case() {
  return (
    <div data-testid="root" className="relative h-full w-full bg-white">
      <style>{css}</style>
      <div data-testid="left" className="left">
        <div data-testid="dot" className="dot" />
      </div>
      <div data-testid="card" className="card absolute" style={{ left: 240, top: 0 }}>
        <p data-testid="in" className="label">Inside the card</p>
      </div>
      <p data-testid="out" className="label absolute" style={{ left: 240, top: 80 }}>Outside it</p>
      <div data-testid="stray" className="dot" />
    </div>
  );
}
