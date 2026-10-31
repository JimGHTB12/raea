// Custom properties inherit down the element tree: a class's var() reads the element's own (its inline style, its
// classes, then its parent's and up), not only :root's. A brand kit sets its sizes inline on an ancestor; the
// classes under it, calc() included, take those, and a box outside it keeps :root's.
// proves: css.properties.custom-property, css.types.var, css.types.calc
export const steps = [];
const css = `:root { --box: 40px; --tint: rgb(255, 0, 0); --size: 12px; }
.box { width: var(--box); height: calc(var(--box) * 2); background: var(--tint); }
.big { font-size: calc(var(--size) * 3); color: var(--tint); }
.blue { --tint: rgb(37, 99, 235); }`;
export default function Case() {
  return (
    <div data-testid="root" className="relative h-full w-full bg-white">
      <style>{css}</style>
      <div data-testid="kit" className="absolute" style={{ left: 0, top: 0, ["--box" as string]: "60px", ["--size" as string]: "16px" }}>
        <div data-testid="inner" className="box" />
        <p data-testid="text" className="big" style={{ margin: 0 }}>Brand</p>
        <div data-testid="blue" className="blue">
          <div data-testid="deep" className="box" />
        </div>
      </div>
      <div data-testid="outside" className="box absolute" style={{ left: 300, top: 0 }} />
    </div>
  );
}
